use std::collections::HashMap;

use eframe::egui::{Color32, Direction, Id, Layout, RichText};
use eframe::{egui, App, Frame};
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use url::Url;

pub struct RiemannDashApp {
    valid_url: Url,
    editable_url: String,
    selected_workspace: usize,
    workspaces: Vec<Workspace>,
}

impl App for RiemannDashApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label("WS URL:");
                let valid_editable_url = Url::parse(&self.editable_url).is_ok();
                let lost_focus = ui
                    .with_layout(Layout::left_to_right(), |ui| {
                        if !valid_editable_url {
                            ui.style_mut().visuals.extreme_bg_color = Color32::LIGHT_RED;
                        }
                        ui.text_edit_singleline(&mut self.editable_url).lost_focus()
                    })
                    .inner;

                if lost_focus || ui.input().key_pressed(egui::Key::Enter) {
                    match Url::parse(&self.editable_url) {
                        Ok(url) => self.valid_url = url,
                        Err(e) => eprintln!("{}", e),
                    }
                }
                ui.separator();

                for (i, workspace) in self.workspaces.iter().enumerate() {
                    if self.selected_workspace == i {
                        ui.add(egui::Button::new(&workspace.name).fill(egui::Color32::LIGHT_BLUE));
                    } else if ui.button(&workspace.name).clicked() {
                        self.selected_workspace = i;
                    }
                }

                if ui.button("+").clicked() {
                    self.workspaces.push(Workspace::default());
                    self.selected_workspace = self.workspaces.len().saturating_sub(1);
                }
            });
        });

        self.selected_workspace =
            self.selected_workspace.min(self.workspaces.len().saturating_sub(1));
        match self.workspaces.get_mut(self.selected_workspace) {
            Some(workspace) => {
                if !workspace.ui(&self.valid_url, ctx) {
                    self.workspaces.remove(self.selected_workspace);
                }
            }
            None => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(Direction::TopDown),
                        |ui| ui.label("There is no workspace, create one with the (+) button."),
                    );
                });
            }
        }
    }
}

impl Default for RiemannDashApp {
    fn default() -> Self {
        let url = Url::parse("ws://163.172.181.220:5556").unwrap(); // ws://localhost:5556
        Self {
            editable_url: url.as_str().to_string(),
            valid_url: url,
            selected_workspace: 0,
            workspaces: vec![Workspace::default()],
        }
    }
}

struct Workspace {
    name: String,
    widgets: Vec<Widget>,
}

impl Workspace {
    fn ui(&mut self, url: &Url, ctx: &egui::Context) -> bool {
        let mut is_open = true;

        egui::SidePanel::right("workspace_right_side_panel").show(ctx, |ui| {
            ui.label("Workspace name:");
            ui.text_edit_singleline(&mut self.name);
            ui.separator();
            let delete_button = egui::Button::new("delete workpace").fill(egui::Color32::LIGHT_RED);
            if ui.add(delete_button).clicked() {
                is_open = false;
            }
            ui.separator();
            if ui.button("☟ scrolling list").clicked() {
                self.widgets.push(Widget::default());
            }
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            for (i, widget) in self.widgets.iter_mut().enumerate() {
                egui::Window::new("<widget type name>").id(Id::new(i)).show(ctx, |ui| {
                    widget.ui(url, ui);
                });
            }
        });

        is_open
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self { name: "Riemann".to_string(), widgets: Default::default() }
    }
}

#[derive(Default)]
struct Widget {
    query: String,
    events: Vec<WsEvent>,
    event_receiver: Option<EventReceiver>,
}

impl Widget {
    fn ui(&mut self, url: &Url, ui: &mut egui::Ui) {
        if let Some(event_receiver) = &self.event_receiver {
            while let Some(event) = event_receiver.try_recv() {
                self.events.push(event);
            }
        }

        ui.collapsing("Query string", |ui| {
            ui.group(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut self.query)
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .desired_rows(3)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY),
                    );

                    if (response.lost_focus() && !self.query.is_empty())
                        || self
                            .event_receiver
                            .as_ref()
                            .map_or(false, |er| base_url(er.url().clone()) != base_url(url.clone()))
                    {
                        let ctx = ui.ctx().clone();
                        let wakeup = move || ctx.request_repaint(); // wake up UI thread on new message
                        let url = websocket_url(url, true, &self.query);
                        match EventReceiver::connect(url, wakeup) {
                            Ok(event_receiver) => self.event_receiver = Some(event_receiver),
                            Err(_) => (),
                        }
                    }
                });
            });
        });

        egui::ScrollArea::vertical().stick_to_bottom().show(ui, |ui| {
            for event in &self.events {
                let text = match event {
                    WsEvent::Message(WsMessage::Text(text)) => {
                        match serde_json::from_str::<Event>(text) {
                            Ok(event) => RichText::new(format!("{:?}", event)),
                            Err(e) => RichText::new(format!("{:?} {}", event, e))
                                .color(Color32::LIGHT_RED),
                        }
                    }
                    otherwise => {
                        RichText::new(format!("{:?}", otherwise)).color(Color32::LIGHT_RED)
                    }
                };
                ui.label(text);
            }
        });
    }
}

struct EventReceiver {
    url: Url,
    // don't drop the sender or the connection will be closed
    _sender: WsSender,
    receiver: WsReceiver,
}

impl EventReceiver {
    pub fn connect(url: Url, wakeup: impl Fn() + Send + Sync + 'static) -> ewebsock::Result<Self> {
        match ewebsock::connect_with_wakeup(url.as_str(), wakeup) {
            Ok((sender, receiver)) => Ok(EventReceiver { url, _sender: sender, receiver }),
            Err(e) => Err(e),
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn try_recv(&self) -> Option<WsEvent> {
        self.receiver.try_recv()
    }
}

#[derive(Debug, Deserialize)]
pub struct Event {
    time: Option<String>,
    state: Option<String>,
    service: Option<String>,
    host: Option<String>,
    description: Option<String>,
    #[serde(deserialize_with = "deserialize_collections")]
    tags: Vec<String>,
    ttl: Option<f32>,
    time_micros: Option<i64>,
    metric: Option<f32>,
    #[serde(flatten)]
    attributes: HashMap<String, Value>,
}

fn websocket_url(url: &Url, subscribe: bool, query: &str) -> Url {
    use url::form_urlencoded::Serializer;

    let query = Serializer::new(String::new())
        .append_pair("subscribe", &subscribe.to_string())
        .append_pair("query", query)
        .finish();
    let mut url = url.clone().join("index/").unwrap();
    url.set_query(Some(&query));
    url
}

fn base_url(mut url: Url) -> Result<Url, url::ParseError> {
    match url.path_segments_mut() {
        Ok(mut path) => path.clear(),
        Err(_) => return Err(url::ParseError::RelativeUrlWithCannotBeABaseBase),
    };
    url.set_query(None);
    Ok(url)
}

fn deserialize_collections<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    // Deserialize null to empty Vec
    Deserialize::deserialize(deserializer).or(Ok(vec![]))
}
