use crate::visualizer::state::CompilerSnapshot;
use crate::visualizer::views::code_view::{show_comparison, CodeKind, CodePane};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, snapshot: &CompilerSnapshot, search: &mut String) {
    ui.heading("Phase 6 · Source → x86-64 assembly");
    ui.label("Read the source and its generated assembly together. Both panes use a shared scroll position; syntax, directives, labels, and instructions are colour-coded. Assembly follows the native host ABI selected at build time.");
    ui.horizontal(|ui| {
        ui.label("Assembly filter:");
        ui.text_edit_singleline(search);
        if ui.button("Clear").clicked() {
            search.clear();
        }
        ui.label(format!(
            "{} assembly line(s)",
            snapshot.assembly.lines().count()
        ));
    });
    if !snapshot.activation_records.is_empty() {
        egui::CollapsingHeader::new("Activation records · stack frame map")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for record in &snapshot.activation_records {
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(32, 49, 67))
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(&record.function)
                                        .strong()
                                        .color(egui::Color32::from_rgb(143, 190, 255)),
                                );
                                ui.label(format!(
                                    "{} bytes · {} slot(s)",
                                    record.frame_size, record.slot_count
                                ));
                                let mut slots: Vec<(&String, &i32)> = record.slots.iter().collect();
                                slots.sort_by_key(|(_, offset)| **offset);
                                for (name, offset) in slots {
                                    ui.monospace(format!(
                                        "{:<14} [rbp - {}]",
                                        name.strip_prefix('$').unwrap_or(name.as_str()),
                                        offset
                                    ));
                                }
                            });
                    }
                });
            });
    }
    let assembly = if search.trim().is_empty() {
        snapshot.assembly.clone()
    } else {
        snapshot
            .assembly
            .lines()
            .filter(|line| line.to_lowercase().contains(&search.to_lowercase()))
            .collect::<Vec<_>>()
            .join("\n")
    };
    show_comparison(
        ui,
        "phase6-source-assembly",
        CodePane {
            title: "Source · .t",
            text: &snapshot.raw_source,
            kind: CodeKind::Source,
        },
        CodePane {
            title: "Target · .s",
            text: &assembly,
            kind: CodeKind::Assembly,
        },
    );
}
