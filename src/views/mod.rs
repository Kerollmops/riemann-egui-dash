use eframe::egui;
use url::Url;

pub mod about;
pub mod big_number;
pub mod flot;
pub mod log;

pub trait View {
    fn title(&self) -> String;
    fn show(&mut self, ctx: &egui::Context, id: egui::Id, url: &Url, open: &mut bool);
    fn ui(&mut self, ui: &mut egui::Ui, url: &Url);
}
