use eframe::egui;
use url::Url;

mod log;

pub use self::log::Log;

pub trait View {
    fn title(&self) -> String;
    fn show(&mut self, ctx: &egui::Context, id: egui::Id, url: &Url, open: &mut bool);
    fn ui(&mut self, ui: &mut egui::Ui, url: &Url);
}
