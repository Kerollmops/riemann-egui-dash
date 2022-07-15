use std::collections::BTreeMap;

use eframe::egui;
use eframe::egui::plot::{Legend, Line, Plot, Value, Values};
use eframe::egui::TextStyle;
use ewebsock::{WsEvent, WsMessage};
use url::Url;

use super::View;
use crate::event::{Event, EventReceiver};
use crate::{base_url, websocket_url};

pub struct Flot {
    query: String,
    limit: f32,
    events: Vec<Event>,
    event_receiver: Option<EventReceiver>,
}

impl View for Flot {
    fn title(&self) -> String {
        String::from("📈 Flot Graph")
    }

    fn show(&mut self, ctx: &egui::Context, id: egui::Id, url: &Url, open: &mut bool) {
        egui::Window::new(self.title()).id(id).open(open).show(ctx, |ui| self.ui(ui, url));
    }

    fn ui(&mut self, ui: &mut egui::Ui, url: &Url) {
        if let Some(event_receiver) = &self.event_receiver {
            while let Some(event) = event_receiver.try_recv() {
                if let WsEvent::Message(WsMessage::Text(text)) = event {
                    if let Ok(event) = serde_json::from_str::<Event>(&text) {
                        self.events.push(event);
                    }
                }
            }
        }

        if self.events.len() > self.limit as usize {
            let diff = self.events.len() - self.limit as usize;
            self.events.drain(0..diff);
        }

        // TODO(kerollmops) create a simple function for that
        ui.collapsing("Settings", |ui| {
            ui.group(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.limit, 10.0..=10_000.0)
                            .integer()
                            .text("message limit"),
                    );

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
                        let ctx = ui.ctx().clone();
                        let wakeup = move || ctx.request_repaint(); // wake up UI thread on new message
                        let url = websocket_url(url, true, &self.query);
                        match EventReceiver::connect(url, wakeup) {
                            Ok(event_receiver) => {
                                self.events.clear();
                                self.event_receiver = Some(event_receiver);
                            }
                            Err(_) => (),
                        }
                    }
                });
            });
        });

        Plot::new("lines").legend(Legend::default()).show(ui, |plot_ui| {
            let mut lines = BTreeMap::new();
            for event in &self.events {
                if let Some(((service, metric), time)) =
                    event.service.as_ref().zip(event.metric).zip(event.time)
                {
                    let time = time.unix_timestamp_nanos() / 1_000_000; // millis
                    let point = Value { x: time as f64, y: metric as f64 };
                    lines.entry(service).or_insert_with(Vec::new).push(point);
                }
            }

            for (service, points) in lines {
                plot_ui.line(Line::new(Values::from_values(points)).name(service));
            }
        });
    }
}

impl Default for Flot {
    fn default() -> Self {
        Self {
            query: Default::default(),
            limit: 1000.0,
            events: Default::default(),
            event_receiver: Default::default(),
        }
    }
}
