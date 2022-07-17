use eframe::egui;
use url::Url;

use super::View;

pub const DEFAULT_TITLE: &str = "About this";

#[derive(Default)]
pub struct About;

impl View for About {
    fn title(&self) -> String {
        DEFAULT_TITLE.to_string()
    }

    fn show(&mut self, ctx: &egui::Context, id: egui::Id, url: &Url, open: &mut bool) {
        egui::Window::new(self.title())
            .id(id)
            .default_width(380.0)
            .open(open)
            .show(ctx, |ui| self.ui(ui, url));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _url: &Url) {
        use egui::special_emojis::{OS_APPLE, OS_LINUX, OS_WINDOWS};

        ui.style_mut().spacing.interact_size.y = 0.0; // hack to make `horizontal_wrapped` work better with text.

        ui.heading("Riemann egui Dashboard");

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("It uses websockets and is based on ");
            ui.hyperlink_to("egui", "https://github.com/emilk/egui");
            ui.label(format!(
                " to let you compose fully interactive, low-latency visualizations of Riemann's index. \
                egui runs both on the web and natively on {}{}{}.",
                OS_APPLE, OS_LINUX, OS_WINDOWS,
            ));
        });

        ui.add_space(12.0);

        ui.label(
            "It includes a basic window manager. \
                Each workspace (top left) is an arrangement of views. \
                Each view is a single query, visualized in a specific way.",
        );

        ui.add_space(12.0);

        ui.heading("Queries");

        ui.label(
            "Clients can query the index for particular events using a simple query language. \
            Dashboard views are each powered by a single query. \
            Queries can be applied on the index of past events, but can also tap into the stream of events going to the index in real-time."
        );

        ui.add_space(8.0);

        let mut code = r#"# Simple equality
state = "ok"

# Wildcards
(service =~ "disk%") or
(state != "critical" and host =~ "%.trioptimum.com")

# Standard operator precedence applies
metric_f > 2.0 and not host = nil

# Anything with a tag "product"
tagged "product"

# All states
true"#;
        ui.add(
            egui::TextEdit::multiline(&mut code)
                .font(egui::TextStyle::Monospace) // for cursor height
                .code_editor()
                .desired_rows(1)
                .desired_width(f32::INFINITY)
                .lock_focus(true),
        );

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("Query messages return a list of matching events. See ");
            ui.hyperlink_to(
                "the query tests",
                "https://github.com/riemann/riemann/blob/master/test/riemann/query_test.clj",
            );
            ui.label(" for tons of examples, or read ");
            ui.hyperlink_to(
                "the full grammar",
                "https://github.com/riemann/riemann/blob/master/resources/query.g4",
            );
            ui.label(".");
        });

        ui.add_space(12.0);

        ui.heading("Links");
        links(ui);
    }
}

fn links(ui: &mut egui::Ui) {
    use egui::special_emojis::GITHUB;
    ui.hyperlink_to(format!("{} egui on GitHub", GITHUB), "https://github.com/emilk/egui");
    ui.hyperlink_to("Riemann's website", "https://riemann.io");
}
