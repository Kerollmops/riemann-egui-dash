use eframe::egui;

#[derive(Clone, Copy, Debug)]
pub enum Clicked {
    None,
    Once,
    Twice,
}

impl Default for Clicked {
    fn default() -> Self {
        Clicked::None
    }
}

impl Clicked {
    pub fn confirm_clicked(&self) -> bool {
        matches!(self, Clicked::Twice)
    }

    fn click_it(&mut self) {
        *self = match self {
            Clicked::None => Clicked::Once,
            Clicked::Once => Clicked::Twice,
            Clicked::Twice => Clicked::Twice,
        };
    }
}

pub struct ConfirmButton<'a> {
    clicked: &'a mut Clicked,
    text: egui::WidgetText,
    confirm_text: egui::WidgetText,
    fill: Option<egui::Color32>,
}

impl<'a> ConfirmButton<'a> {
    pub fn new(
        clicked: &'a mut Clicked,
        text: impl Into<egui::WidgetText>,
        confirm_text: impl Into<egui::WidgetText>,
    ) -> Self {
        ConfirmButton { clicked, text: text.into(), confirm_text: confirm_text.into(), fill: None }
    }

    pub fn fill(self, color: impl Into<egui::Color32>) -> Self {
        Self { fill: Some(color.into()), ..self }
    }
}

impl egui::Widget for ConfirmButton<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let button = match self.clicked {
            Clicked::None => egui::Button::new(self.text),
            Clicked::Once => egui::Button::new(self.confirm_text),
            Clicked::Twice => egui::Button::new(self.confirm_text),
        };

        let button = match self.fill {
            Some(color) => button.fill(color),
            None => button,
        };

        let response = button.ui(ui);
        if response.clicked() {
            self.clicked.click_it();
        }
        response
    }
}
