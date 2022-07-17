use eframe::egui;
use eframe::egui::TextStyle;
use ewebsock::{WsEvent, WsMessage};
use url::Url;

use super::View;
use crate::event::{Event, EventReceiver};
use crate::{base_url, websocket_url};

pub const DEFAULT_TITLE: &str = "🔢 Big Number";

pub struct BigNumber {
    query: String,
    title: String,
    current_metric: Option<f32>,
    current_state_ok: Option<bool>,
    event_receiver: Option<EventReceiver>,
}

impl View for BigNumber {
    fn title(&self) -> String {
        DEFAULT_TITLE.to_string()
    }

    fn show(&mut self, ctx: &egui::Context, id: egui::Id, url: &Url, open: &mut bool) {
        egui::Window::new(self.title()).id(id).open(open).show(ctx, |ui| self.ui(ui, url));
    }

    fn ui(&mut self, ui: &mut egui::Ui, url: &Url) {
        if let Some(event_receiver) = &self.event_receiver {
            while let Some(event) = event_receiver.try_recv() {
                if let WsEvent::Message(WsMessage::Text(text)) = event {
                    if let Ok(Event { metric, state, .. }) = serde_json::from_str::<Event>(&text) {
                        self.current_metric = metric;
                        self.current_state_ok = state.map(|s| s == "ok");
                    }
                }
            }
        }

        // TODO(kerollmops) create a simple function for that
        ui.collapsing("Settings", |ui| {
            ui.group(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label("title:");
                    ui.add(egui::TextEdit::singleline(&mut self.title));

                    ui.label("query string:");
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut self.query)
                            .font(TextStyle::Monospace) // for cursor height
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
                        let wakeup = move || ();
                        let url = websocket_url(url, true, &self.query);
                        if let Ok(event_receiver) = EventReceiver::connect(url, wakeup) {
                            self.current_metric = None;
                            self.event_receiver = Some(event_receiver);
                        }
                    }
                });
            });
        });

        ui.heading(&self.title);

        match self.current_metric {
            Some(num) => ui.heading(format!("{:.02?}", num)),
            None => ui.heading("-.--"),
        };
    }
}

impl Default for BigNumber {
    fn default() -> Self {
        Self {
            query: Default::default(),
            title: Default::default(),
            current_metric: Default::default(),
            current_state_ok: Default::default(),
            event_receiver: Default::default(),
        }
    }
}
