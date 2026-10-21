use crate::visualizer::state::CompilerSnapshot;
use crate::visualizer::views::code_view::{show_comparison, CodeKind, CodePane};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, snapshot: &CompilerSnapshot) {
    ui.heading("Phase 0 · Preprocessor");
    ui.label("Both panes share one scroll position, so indentation and corresponding lines stay aligned. Keywords, operators, integers, comments, and the built-in print() call use a consistent colour palette. Long lines scroll horizontally rather than wrapping and breaking alignment.");
    ui.horizontal_wrapped(|ui| {
        legend(ui, "Keywords", egui::Color32::from_rgb(199, 176, 255));
        legend(
            ui,
            "Integer literals",
            egui::Color32::from_rgb(250, 204, 110),
        );
        legend(ui, "print()", egui::Color32::from_rgb(102, 220, 224));
        legend(ui, "Comments", egui::Color32::from_rgb(117, 143, 132));
    });
    ui.add_space(8.0);
    show_comparison(
        ui,
        "phase0-source-compare",
        CodePane {
            title: "Original source · .t",
            text: &snapshot.raw_source,
            kind: CodeKind::Source,
        },
        CodePane {
            title: "Preprocessed source · .ti",
            text: &snapshot.preprocessed_source,
            kind: CodeKind::Source,
        },
    );
}

fn legend(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, color);
        ui.label(label);
    });
}
