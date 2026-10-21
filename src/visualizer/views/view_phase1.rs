use crate::visualizer::state::CompilerSnapshot;
use eframe::egui;

pub fn show(
    ui: &mut egui::Ui,
    snapshot: &CompilerSnapshot,
    search: &mut String,
    selected: &mut Option<usize>,
) {
    ui.heading("Phase 1 · Lexical analysis");
    ui.label(format!(
        "{} token(s), including indentation and end-of-file markers.",
        snapshot.tokens.len()
    ));
    ui.horizontal(|ui| {
        ui.label("Filter tokens:");
        ui.text_edit_singleline(search);
        if ui.button("Clear").clicked() {
            search.clear();
        }
    });
    ui.separator();
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(18, 23, 33))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(49, 62, 82)))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::ScrollArea::both()
                .id_salt("token_table_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("token_grid")
                        .striped(true)
                        .num_columns(5)
                        .spacing([18.0, 8.0])
                        .show(ui, |ui| {
                            ui.strong("Index");
                            ui.strong("Line:Col");
                            ui.strong("TokenKind");
                            ui.strong("Lexeme");
                            ui.strong("Inspect");
                            ui.end_row();
                            for (index, token) in snapshot.tokens.iter().enumerate() {
                                let description =
                                    format!("{} {} {}", token.kind, token.lexeme, token.line);
                                if !search.is_empty()
                                    && !description.to_lowercase().contains(&search.to_lowercase())
                                {
                                    continue;
                                }
                                ui.label(index.to_string());
                                ui.label(format!("{}:{}", token.line, token.column));
                                ui.monospace(token.kind.to_string());
                                ui.monospace(format!("{:?}", token.lexeme));
                                if ui
                                    .selectable_label(*selected == Some(index), "Select")
                                    .clicked()
                                {
                                    *selected = Some(index);
                                }
                                ui.end_row();
                            }
                        });
                });
        });
    if let Some(i) = *selected {
        if let Some(token) = snapshot.tokens.get(i) {
            ui.separator();
            ui.heading("Selected token");
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Index: {}", i));
                ui.label(format!("Kind: {}", token.kind));
                ui.label(format!("Lexeme: {:?}", token.lexeme));
                ui.label(format!(
                    "Source position: line {}, column {}",
                    token.line, token.column
                ));
            });
            ui.label("INDENT and DEDENT are emitted by comparing each nonblank line's indentation width with the indentation stack.");
        }
    }
}
