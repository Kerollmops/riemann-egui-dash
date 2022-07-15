use eframe::egui::{Button, Color32, Direction, Id, Layout, TextEdit};
use eframe::{egui, App, Frame};
use url::Url;

use crate::views::Log;

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
    widgets: Vec<Log>,
}

impl Workspace {
    fn ui(&mut self, url: &Url, ctx: &egui::Context) -> bool {
        let mut is_open = true;

        egui::SidePanel::right("workspace_right_side_panel").show(ctx, |ui| {
            ui.separator();
            ui.add(TextEdit::singleline(&mut self.name).hint_text("Workspace Name"));
            ui.separator();
            if ui.button("📃 Scrolling List").clicked() {
                self.widgets.push(Log::default());
            }
            ui.separator();
            if ui.add(Button::new("🗑 Delete workpace").fill(Color32::LIGHT_RED)).clicked() {
                is_open = false;
            }
            if ui.button("Organize windows").clicked() {
                ui.ctx().memory().reset_areas();
                ui.close_menu();
            }
        });

        let mut to_delete = Vec::new();
        egui::CentralPanel::default().show(ctx, |_ui| {
            for (i, widget) in self.widgets.iter_mut().enumerate() {
                let mut opened = true;
                egui::Window::new("<widget type name>")
                    .id(Id::new(i))
                    .open(&mut opened)
                    .show(ctx, |ui| widget.ui(url, ui));
                if opened == false {
                    to_delete.push(i);
                }
            }
        });

        let mut removed = 0;
        for i in to_delete {
            self.widgets.remove(i - removed);
            removed += 1;
        }

        is_open
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self { name: "Riemann".to_string(), widgets: Default::default() }
    }
}
