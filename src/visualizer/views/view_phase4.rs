use crate::phase4_intermediate::three_address_code::Quadruple;
use crate::visualizer::state::CompilerSnapshot;
use eframe::egui::{self, Color32, Stroke, Vec2};

pub fn show(
    ui: &mut egui::Ui,
    snapshot: &CompilerSnapshot,
    search: &mut String,
    selected: &mut Option<usize>,
) {
    ui.heading("Phase 4 · Three-address code");
    ui.label("Each row is a quadruple: operation, two operands, and a result/target. Colours group arithmetic, data movement, calls, and control-flow instructions.");
    ui.horizontal(|ui| {
        ui.label("Filter TAC:");
        ui.text_edit_singleline(search);
        if ui.button("Clear filter").clicked() {
            search.clear();
        }
        ui.separator();
        ui.label(format!(
            "{} optimized · {} original",
            snapshot.tac_optimized.len(),
            snapshot.tac_unoptimized.len()
        ));
    });
    ui.add_space(8.0);
    if snapshot.tac_optimized.is_empty() {
        ui.weak("TAC is unavailable because the intermediate representation did not complete.");
        return;
    }

    egui::Frame::new()
        .fill(Color32::from_rgb(18, 23, 33))
        .stroke(Stroke::new(1.0, Color32::from_rgb(49, 62, 82)))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            let width = ui.available_width().max(700.0);
            let cols = [42.0, 102.0, 112.0, 112.0, 132.0];
            let linear_width = (width - cols.iter().sum::<f32>() - 24.0).max(180.0);
            ui.horizontal(|ui| {
                for (label, col_width) in [
                    ("#", cols[0]),
                    ("Operation", cols[1]),
                    ("Arg 1", cols[2]),
                    ("Arg 2", cols[3]),
                    ("Result / target", cols[4]),
                    ("Linear TAC", linear_width),
                ] {
                    ui.add_sized(
                        Vec2::new(col_width, 25.0),
                        egui::Label::new(
                            egui::RichText::new(label)
                                .strong()
                                .color(Color32::from_rgb(165, 190, 222)),
                        ),
                    );
                }
            });
            ui.separator();
            for (idx, quad) in snapshot.tac_optimized.iter().enumerate() {
                let full = quad.to_string();
                if !search.is_empty()
                    && !format!("{} {}", quad.op, full)
                        .to_lowercase()
                        .contains(&search.to_lowercase())
                {
                    continue;
                }
                let is_selected = *selected == Some(idx);
                let row_fill = if is_selected {
                    Color32::from_rgb(36, 57, 81)
                } else if idx % 2 == 0 {
                    Color32::from_rgb(25, 32, 44)
                } else {
                    Color32::from_rgb(21, 28, 39)
                };
                egui::Frame::new()
                    .fill(row_fill)
                    .stroke(Stroke::new(
                        if is_selected { 1.4 } else { 0.0 },
                        Color32::from_rgb(95, 167, 225),
                    ))
                    .corner_radius(egui::CornerRadius::same(4))
                    .inner_margin(egui::Margin {
                        left: 5,
                        right: 5,
                        top: 3,
                        bottom: 3,
                    })
                    .show(ui, |ui| {
                        ui.set_min_width(width);
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                Vec2::new(cols[0], 22.0),
                                egui::Label::new(
                                    egui::RichText::new(idx.to_string())
                                        .color(Color32::GRAY)
                                        .monospace(),
                                ),
                            );
                            let (fill, ink) = op_color(&quad.op);
                            egui::Frame::new()
                                .fill(fill)
                                .corner_radius(egui::CornerRadius::same(4))
                                .inner_margin(egui::Margin {
                                    left: 5,
                                    right: 5,
                                    top: 2,
                                    bottom: 2,
                                })
                                .show(ui, |ui| {
                                    ui.add_sized(
                                        Vec2::new(cols[1] - 10.0, 20.0),
                                        egui::Label::new(
                                            egui::RichText::new(quad.op.as_str())
                                                .strong()
                                                .color(ink)
                                                .monospace(),
                                        ),
                                    );
                                });
                            ui.add_sized(
                                Vec2::new(cols[2], 22.0),
                                egui::Label::new(
                                    egui::RichText::new(show_value(&quad.arg1))
                                        .monospace()
                                        .color(Color32::from_rgb(222, 229, 240)),
                                ),
                            );
                            ui.add_sized(
                                Vec2::new(cols[3], 22.0),
                                egui::Label::new(
                                    egui::RichText::new(show_value(&quad.arg2))
                                        .monospace()
                                        .color(Color32::from_rgb(222, 229, 240)),
                                ),
                            );
                            ui.add_sized(
                                Vec2::new(cols[4], 22.0),
                                egui::Label::new(
                                    egui::RichText::new(show_value(&quad.result))
                                        .monospace()
                                        .color(Color32::from_rgb(104, 224, 198)),
                                ),
                            );
                            if ui
                                .add_sized(
                                    Vec2::new(linear_width, 22.0),
                                    egui::Button::new(egui::RichText::new(full).monospace())
                                        .selected(is_selected),
                                )
                                .clicked()
                            {
                                *selected = Some(idx);
                            }
                        });
                    });
            }
        });

    if let Some(index) = *selected {
        if let Some(quad) = snapshot.tac_optimized.get(index) {
            ui.add_space(8.0);
            egui::Frame::new()
                .fill(Color32::from_rgb(30, 43, 60))
                .stroke(Stroke::new(1.0, Color32::from_rgb(96, 165, 250)))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.heading(format!("Instruction {} · {}", index, quad.op));
                    show_quad_details(ui, quad);
                });
        }
    }
    egui::CollapsingHeader::new("Before vs. after optimization").default_open(false).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            metric(ui, "Before", snapshot.tac_unoptimized.len().to_string(), Color32::from_rgb(76, 51, 37), Color32::from_rgb(251, 191, 36));
            metric(ui, "After", snapshot.tac_optimized.len().to_string(), Color32::from_rgb(31, 63, 57), Color32::from_rgb(45, 212, 191));
        });
        ui.label("Constant facts are propagated only within straight-line regions and are cleared at branches, labels, calls, returns, and function boundaries.");
    });
}

fn op_color(op: &str) -> (Color32, Color32) {
    if matches!(op, "MOV" | "PARAM") {
        (
            Color32::from_rgb(30, 58, 77),
            Color32::from_rgb(125, 211, 252),
        )
    } else if matches!(
        op,
        "ADD" | "SUB" | "MUL" | "DIV" | "NEG" | "AND" | "OR" | "XOR" | "SHL" | "SHR"
    ) {
        (
            Color32::from_rgb(69, 49, 29),
            Color32::from_rgb(251, 191, 36),
        )
    } else if op == "CALL" || op == "PRINT" {
        (
            Color32::from_rgb(42, 61, 49),
            Color32::from_rgb(134, 239, 172),
        )
    } else if op.starts_with("BR_") || matches!(op, "GOTO" | "LABEL") {
        (
            Color32::from_rgb(73, 40, 61),
            Color32::from_rgb(244, 114, 182),
        )
    } else {
        (
            Color32::from_rgb(48, 45, 79),
            Color32::from_rgb(196, 181, 253),
        )
    }
}

fn metric(ui: &mut egui::Ui, label: &str, value: String, fill: Color32, accent: Color32) {
    egui::Frame::new()
        .fill(fill)
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).color(Color32::LIGHT_GRAY));
            ui.label(egui::RichText::new(value).size(18.0).strong().color(accent));
        });
}

fn show_value(value: &str) -> String {
    if value.is_empty() {
        "—".to_string()
    } else {
        value.strip_prefix('$').unwrap_or(value).to_string()
    }
}

fn show_quad_details(ui: &mut egui::Ui, q: &Quadruple) {
    egui::Grid::new("quad_details")
        .num_columns(2)
        .spacing([14.0, 7.0])
        .show(ui, |ui| {
            ui.label("Operation");
            ui.monospace(q.op.as_str());
            ui.end_row();
            ui.label("Argument 1");
            ui.monospace(show_value(&q.arg1));
            ui.end_row();
            ui.label("Argument 2");
            ui.monospace(show_value(&q.arg2));
            ui.end_row();
            ui.label("Result / target");
            ui.monospace(show_value(&q.result));
            ui.end_row();
            ui.label("Source line");
            ui.label(q.line.to_string());
            ui.end_row();
        });
    match q.op.as_str() {
        "CALL" => {
            ui.label("CALL invokes a user-defined function and stores its integer return value.");
        }
        "PRINT" => {
            ui.label("PRINT calls the platform C runtime to display the values separated by spaces, followed by a newline.");
        }
        _ if q.op.starts_with("BR_") => {
            ui.label("A conditional branch transfers control to its target only when its condition holds.");
        }
        _ => {}
    }
}
