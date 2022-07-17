use eframe::egui;
use eframe::egui::containers::panel::Side;
use eframe::egui::{Color32, Id, TextEdit};
use url::Url;

use crate::confirm_button::{Clicked, ConfirmButton};
use crate::views::*;

pub struct Workspace {
    pub name: String,
    delete_clicked: Clicked,
    about: Option<about::About>,
    views: Vec<Box<dyn View>>,
}

impl Workspace {
    pub fn ui(&mut self, parent_id: Id, url: &Url, open: &mut bool, ctx: &egui::Context) {
        egui::SidePanel::new(Side::Right, parent_id.with("workspace_right_panel"))
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(5.);

                ui.add(TextEdit::singleline(&mut self.name).hint_text("Workspace Name"));

                ui.separator();

                if ui.selectable_label(self.about.is_some(), about::DEFAULT_TITLE).clicked() {
                    self.about = match self.about {
                        Some(_) => None,
                        None => Some(about::About::default()),
                    };
                }

                ui.separator();

                if ui.button(log::DEFAULT_TITLE).clicked() {
                    self.views.push(Box::new(log::Log::default()));
                }
                if ui.button(flot::DEFAULT_TITLE).clicked() {
                    self.views.push(Box::new(flot::Flot::default()));
                }
                if ui.button(big_number::DEFAULT_TITLE).clicked() {
                    self.views.push(Box::new(big_number::BigNumber::default()));
                }

                ui.separator();

                ui.add(
                    ConfirmButton::new(
                        &mut self.delete_clicked,
                        "🗑 Delete workpace",
                        "🗑 Click again",
                    )
                    .fill(Color32::LIGHT_RED),
                );

                if self.delete_clicked.confirm_clicked() {
                    *open = false;
                }

                if ui.button("Organize windows").clicked() {
                    ui.ctx().memory().reset_areas();
                    ui.close_menu();
                }
            });

        let mut to_delete = Vec::new();
        egui::CentralPanel::default().show(ctx, |_ui| {
            let mut open = true;
            if let Some(about) = self.about.as_mut() {
                about.show(ctx, parent_id.with("about"), url, &mut open);
                if open == false {
                    self.about = None;
                }
            }

            for (i, view) in self.views.iter_mut().enumerate() {
                let mut open = true;
                view.show(ctx, parent_id.with(i), url, &mut open);
                if open == false {
                    to_delete.push(i);
                }
            }
        });

        let mut removed = 0;
        for i in to_delete {
            self.views.remove(i - removed);
            removed += 1;
        }
    }

    pub fn reset_confirm_delete(&mut self) {
        self.delete_clicked = Clicked::default();
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            name: "Riemann".to_string(),
            delete_clicked: Default::default(),
            about: Some(about::About::default()),
            views: Default::default(),
        }
    }
}
