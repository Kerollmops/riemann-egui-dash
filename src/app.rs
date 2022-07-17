use eframe::egui::{Color32, Direction, Id, Layout};
use eframe::{egui, App, Frame};
use url::Url;

use crate::workspace::Workspace;

pub struct RiemannDashApp {
    valid_url: Url,
    editable_url: String,
    selected_workspace: usize,
    workspaces: Vec<Workspace>,
}

impl App for RiemannDashApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);

                ui.separator();

                for (i, workspace) in self.workspaces.iter_mut().enumerate() {
                    if ui.selectable_label(self.selected_workspace == i, &workspace.name).clicked()
                    {
                        workspace.reset_confirm_delete();
                        self.selected_workspace = i;
                    }
                }

                if ui.button("+").clicked() {
                    self.workspaces.push(Workspace::default());
                    self.selected_workspace = self.workspaces.len().saturating_sub(1);
                }

                ui.separator();

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
            });
        });

        self.selected_workspace =
            self.selected_workspace.min(self.workspaces.len().saturating_sub(1));
        match self.workspaces.get_mut(self.selected_workspace) {
            Some(workspace) => {
                let mut open = true;
                workspace.ui(Id::new(self.selected_workspace), &self.valid_url, &mut open, ctx);
                if !open {
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
