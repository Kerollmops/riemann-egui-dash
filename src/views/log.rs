use eframe::egui;
use eframe::egui::{Color32, RichText, TextStyle};
use egui_extras::{Size, TableBuilder};
use ewebsock::{WsEvent, WsMessage};
use url::Url;

use crate::event::{Event, EventReceiver};
use crate::{base_url, websocket_url};

pub struct Log {
    query: String,
    limit: f32,
    events: Vec<Event>,
    event_receiver: Option<EventReceiver>,
}

impl Log {
    pub fn ui(&mut self, url: &Url, ui: &mut egui::Ui) {
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

        egui::ScrollArea::vertical().stick_to_bottom().show(ui, |ui| {
            TableBuilder::new(ui)
                .resizable(true)
                .column(Size::initial(120.0).at_least(60.0)) // host
                .column(Size::initial(120.0).at_least(60.0)) // service
                .column(Size::initial(50.0).at_least(20.0)) // state
                .column(Size::initial(60.0).at_least(20.0)) // metric
                .column(Size::remainder().at_least(60.0)) // description
                .header(25.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Host");
                    });
                    header.col(|ui| {
                        ui.heading("Service");
                    });
                    header.col(|ui| {
                        ui.heading("State");
                    });
                    header.col(|ui| {
                        ui.heading("Metric");
                    });
                    header.col(|ui| {
                        ui.heading("Description");
                    });
                })
                .body(|body| {
                    body.rows(25.0, self.events.len(), |index, mut row| {
                        let event = &self.events[index];
                        row.col(|ui| {
                            if let Some(host) = &event.host {
                                ui.label(host);
                            }
                        });
                        row.col(|ui| {
                            if let Some(service) = &event.service {
                                ui.label(service);
                            }
                        });
                        row.col(|ui| {
                            if let Some(state) = &event.state {
                                if state == "ok" {
                                    ui.label(state);
                                } else {
                                    ui.label(RichText::new(state).color(Color32::LIGHT_RED));
                                }
                            }
                        });
                        row.col(|ui| {
                            if let Some(metric) = event.metric {
                                ui.label(format!("{:.02?}", metric));
                            }
                        });
                        row.col(|ui| {
                            if let Some(description) = &event.description {
                                ui.label(description);
                            }
                        });
                    })
                });
        });
    }
}

impl Default for Log {
    fn default() -> Self {
        Self {
            query: Default::default(),
            limit: 1000.0,
            events: Default::default(),
            event_receiver: Default::default(),
        }
    }
}
