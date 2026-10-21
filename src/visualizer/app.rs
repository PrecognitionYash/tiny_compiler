use crate::common::error_handler::phase_name;
use crate::visualizer::state::CompilerSnapshot;
use crate::visualizer::views::graph_canvas::GraphState;
use crate::visualizer::views::{
    view_phase0, view_phase1, view_phase2, view_phase3, view_phase4, view_phase5, view_phase6,
};
use eframe::egui;

pub struct CompilerApp {
    pub snapshot: CompilerSnapshot,
    start_phase: u8,
    end_phase: u8,
    active_phase: u8,
    token_search: String,
    tac_search: String,
    assembly_search: String,
    selected_token: Option<usize>,
    selected_symbol: Option<usize>,
    selected_quad: Option<usize>,
    selected_block: Option<usize>,
    ast_graph: GraphState,
    semantic_graph: GraphState,
    cfg_zoom: f32,
}

impl CompilerApp {
    pub fn new(snapshot: CompilerSnapshot, range: (u8, u8)) -> Self {
        let start = range.0.min(6);
        let end = range.1.clamp(start, 6);
        Self {
            snapshot,
            start_phase: start,
            end_phase: end,
            active_phase: start,
            token_search: String::new(),
            tac_search: String::new(),
            assembly_search: String::new(),
            selected_token: None,
            selected_symbol: None,
            selected_quad: None,
            selected_block: None,
            ast_graph: GraphState::default(),
            semantic_graph: GraphState::default(),
            cfg_zoom: 1.0,
        }
    }

    fn tab_title(phase: u8) -> &'static str {
        match phase {
            0 => "0 · Preprocess",
            1 => "1 · Tokens",
            2 => "2 · AST",
            3 => "3 · Semantics",
            4 => "4 · TAC",
            5 => "5 · CFG",
            6 => "6 · Assembly",
            _ => "Phase",
        }
    }

    fn phase_summary(&self, phase: u8) -> String {
        if self.snapshot.is_phase_available(phase) {
            format!("Phase {} complete", phase)
        } else if self.snapshot.diagnostics.iter().any(|d| d.phase == phase) {
            format!("Phase {} stopped with an error", phase)
        } else {
            "Not reached".to_string()
        }
    }
}

impl eframe::App for CompilerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // GUI part: eframe/egui draws the complier visualizer using App::ui and nested panels.
        // using dark theme so it looks cool during project presentation viva!
        egui::Panel::top("top_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("tcomp");
                ui.separator();
                ui.label(egui::RichText::new("Compiler Explorer").strong());
                ui.separator();
                ui.label(self.snapshot.input_path.as_str());
            });
            ui.horizontal_wrapped(|ui| {
                for phase in self.start_phase..=self.end_phase {
                    let selected = self.active_phase == phase;
                    let status = if self.snapshot.is_phase_available(phase) {
                        "✓"
                    } else {
                        "·"
                    };
                    if ui
                        .selectable_label(
                            selected,
                            format!("{} {}", status, Self::tab_title(phase)),
                        )
                        .clicked()
                    {
                        self.active_phase = phase;
                    }
                }
            });
        });

        egui::Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!(
                    "Viewing phases {}–{}",
                    self.start_phase, self.end_phase
                ));
                ui.separator();
                ui.label(self.phase_summary(self.active_phase));
                ui.separator();
                ui.label(
                    "Tip: select rows, expand AST nodes, and click CFG blocks to inspect details.",
                );
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if !self.snapshot.diagnostics.is_empty() {
                egui::CollapsingHeader::new(format!("Diagnostics · {}", self.snapshot.diagnostics.len()))
                    .default_open(true)
                    .show(ui, |ui| {
                        for diagnostic in &self.snapshot.diagnostics {
                            ui.colored_label(egui::Color32::LIGHT_RED, diagnostic.to_string());
                        }
                    });
            }

            if !self.snapshot.is_phase_available(self.active_phase) {
                ui.add_space(28.0);
                ui.heading(format!("Phase {} · {}", self.active_phase, phase_name(self.active_phase)));
                ui.label("This phase did not complete. The diagnostic above identifies the first blocking issue; fix it and run tcomp again to populate downstream artifacts.");
                return;
            }

            let active_phase = self.active_phase;
            egui::ScrollArea::vertical()
                .id_salt(("phase_scroll", active_phase))
                .show(ui, |ui| {
                    match active_phase {
                        0 => view_phase0::show(ui, &self.snapshot),
                        1 => view_phase1::show(ui, &self.snapshot, &mut self.token_search, &mut self.selected_token),
                        2 => view_phase2::show(ui, &self.snapshot, &mut self.ast_graph),
                        3 => view_phase3::show(ui, &self.snapshot, &mut self.selected_symbol, &mut self.semantic_graph),
                        4 => view_phase4::show(ui, &self.snapshot, &mut self.tac_search, &mut self.selected_quad),
                        5 => view_phase5::show(ui, &self.snapshot, &mut self.selected_block, &mut self.cfg_zoom),
                        6 => view_phase6::show(ui, &self.snapshot, &mut self.assembly_search),
                        _ => { ui.label("Unknown phase"); }
                    };
                });
        });
    }
}
