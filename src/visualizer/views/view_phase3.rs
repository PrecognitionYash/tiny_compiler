use crate::common::symbol_table::{Scope, SymbolKind, SymbolTable};
use crate::visualizer::state::CompilerSnapshot;
use crate::visualizer::views::graph_canvas::{show_tree, GraphNode, GraphState, NodeKind};
use eframe::egui::{self, Color32, Stroke};

pub fn show(
    ui: &mut egui::Ui,
    snapshot: &CompilerSnapshot,
    selected: &mut Option<usize>,
    graph: &mut GraphState,
) {
    ui.heading("Phase 3 · Semantic analysis + syntax-directed definitions");
    ui.label("The graph shows how names live inside scopes. Click function, scope, or symbol nodes to inspect their attributes and stack slots.");

    ui.horizontal_wrapped(|ui| {
        metric(
            ui,
            "Symbols",
            snapshot.symbols.symbols.len().to_string(),
            Color32::from_rgb(36, 64, 82),
            Color32::from_rgb(125, 211, 252),
        );
        metric(
            ui,
            "Scopes",
            snapshot.symbols.scopes.len().to_string(),
            Color32::from_rgb(39, 67, 57),
            Color32::from_rgb(134, 239, 172),
        );
        metric(
            ui,
            "SDD notes",
            snapshot.sdd_notes.len().to_string(),
            Color32::from_rgb(69, 45, 78),
            Color32::from_rgb(216, 180, 254),
        );
        metric(
            ui,
            "Value type",
            "i64".to_string(),
            Color32::from_rgb(83, 57, 31),
            Color32::from_rgb(251, 191, 36),
        );
    });

    ui.add_space(8.0);
    ui.heading("Semantic scope graph");
    let root = symbol_graph(&snapshot.symbols, &snapshot.sdd_notes);
    show_tree(ui, "semantic-graph", &root, graph);

    ui.add_space(10.0);
    ui.heading("SDD attributes");
    ui.label(
        "These synthesized/inherited attributes summarize what each semantic rule contributes.",
    );
    egui::ScrollArea::horizontal()
        .id_salt("sdd-note-cards")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                for (index, note) in snapshot.sdd_notes.iter().enumerate() {
                    let (fill, edge) = match index % 4 {
                        0 => (
                            Color32::from_rgb(29, 49, 67),
                            Color32::from_rgb(96, 165, 250),
                        ),
                        1 => (
                            Color32::from_rgb(28, 58, 53),
                            Color32::from_rgb(45, 212, 191),
                        ),
                        2 => (
                            Color32::from_rgb(62, 43, 65),
                            Color32::from_rgb(196, 181, 253),
                        ),
                        _ => (
                            Color32::from_rgb(67, 50, 33),
                            Color32::from_rgb(251, 191, 36),
                        ),
                    };
                    egui::Frame::new()
                        .fill(fill)
                        .stroke(Stroke::new(1.0, edge))
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            ui.set_width(290.0);
                            ui.label(
                                egui::RichText::new(format!("Attribute {:02}", index + 1))
                                    .strong()
                                    .color(edge),
                            );
                            ui.add_space(4.0);
                            ui.label(note);
                        });
                    ui.add_space(8.0);
                }
            });
        });

    egui::CollapsingHeader::new(format!(
        "Complete symbol table · {} entries",
        snapshot.symbols.symbols.len()
    ))
    .default_open(true)
    .show(ui, |ui| {
        egui::ScrollArea::both()
            .id_salt("symbol_grid_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("symbol_table_grid")
                    .striped(true)
                    .num_columns(7)
                    .spacing([18.0, 8.0])
                    .show(ui, |ui| {
                        for heading in [
                            "#",
                            "Name",
                            "Type",
                            "Kind",
                            "Scope",
                            "Owner",
                            "Stack offset",
                        ] {
                            ui.strong(heading);
                        }
                        ui.end_row();
                        for (idx, symbol) in snapshot.symbols.symbols.iter().enumerate() {
                            ui.label(idx.to_string());
                            let name_color = match &symbol.kind {
                                SymbolKind::Function => Color32::from_rgb(143, 190, 255),
                                SymbolKind::Parameter => Color32::from_rgb(45, 212, 191),
                                SymbolKind::Variable => Color32::from_rgb(251, 191, 36),
                            };
                            if ui
                                .selectable_label(
                                    *selected == Some(idx),
                                    egui::RichText::new(symbol.name.as_str()).color(name_color),
                                )
                                .clicked()
                            {
                                *selected = Some(idx);
                            }
                            ui.monospace(symbol.ty.as_str());
                            ui.label(symbol.kind.to_string());
                            ui.label(format!(
                                "{} · {}",
                                symbol.scope_id,
                                snapshot.symbols.scope_label(symbol.scope_id)
                            ));
                            ui.label(symbol.owner_function.as_deref().unwrap_or("—"));
                            let offset = if matches!(&symbol.kind, SymbolKind::Function) {
                                "— (no local stack slot)".to_string()
                            } else {
                                symbol
                                    .stack_offset
                                    .map(|n| format!("[rbp - {}]", n))
                                    .unwrap_or_else(|| "allocated in Phase 6".to_string())
                            };
                            ui.monospace(offset);
                            ui.end_row();
                        }
                    });
            });
        if let Some(i) = *selected {
            if let Some(symbol) = snapshot.symbols.symbols.get(i) {
                ui.add_space(8.0);
                egui::Frame::new()
                    .fill(Color32::from_rgb(29, 37, 52))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.heading(format!("Selected: {}", symbol.name));
                        ui.horizontal_wrapped(|ui| {
                            ui.label(format!("Type: {}", symbol.ty));
                            ui.label(format!("Kind: {}", symbol.kind));
                            ui.label(format!("Declared on line {}", symbol.declaration_line));
                            ui.label(format!(
                                "Scope: {}",
                                snapshot.symbols.scope_label(symbol.scope_id)
                            ));
                        });
                        if matches!(&symbol.kind, SymbolKind::Function) {
                            ui.label(format!(
                                "Signature: {}({}) → i64",
                                symbol.name,
                                symbol.params.join(", ")
                            ));
                        } else {
                            ui.monospace(format!(
                                "Activation record: {}",
                                symbol
                                    .stack_offset
                                    .map(|v| format!("[rbp - {v}]"))
                                    .unwrap_or_else(|| "awaiting code generation".to_string())
                            ));
                        }
                    });
            }
        }
    });
}

fn metric(ui: &mut egui::Ui, title: &str, value: String, fill: Color32, accent: Color32) {
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, accent))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).color(Color32::from_rgb(175, 190, 210)));
            ui.label(egui::RichText::new(value).size(22.0).strong().color(accent));
        });
}

fn symbol_graph(table: &SymbolTable, notes: &[String]) -> GraphNode {
    let global_symbols = table
        .symbols
        .iter()
        .filter(|symbol| symbol.scope_id == 0)
        .map(|symbol| {
            let (kind, detail) = match &symbol.kind {
                SymbolKind::Function if symbol.name == "print" => (
                    NodeKind::Function,
                    "built-in · variadic print(i64, ...) · reserved name".to_string(),
                ),
                SymbolKind::Function => (
                    NodeKind::Function,
                    format!(
                        "fn({}) → i64 · line {}",
                        symbol.params.join(", "),
                        symbol.declaration_line
                    ),
                ),
                SymbolKind::Parameter => (NodeKind::Variable, format!("parameter · {}", symbol.ty)),
                SymbolKind::Variable => (NodeKind::Variable, format!("variable · {}", symbol.ty)),
            };
            GraphNode::new(
                format!("symbol:{}:{}", symbol.scope_id, symbol.name),
                symbol.name.clone(),
                detail,
                kind,
            )
        })
        .collect::<Vec<_>>();
    let child_scopes = table
        .scopes
        .iter()
        .filter(|scope| scope.parent == Some(0))
        .map(|scope| scope_node(scope, table))
        .collect::<Vec<_>>();
    let mut children = global_symbols;
    children.extend(child_scopes);
    let attributes = notes
        .iter()
        .enumerate()
        .map(|(index, note)| {
            GraphNode::new(
                format!("sdd-attribute:{index}"),
                format!("Attribute {}", index + 1),
                note.clone(),
                NodeKind::Attribute,
            )
        })
        .collect::<Vec<_>>();
    children.push(
        GraphNode::new(
            "sdd-attributes",
            "SDD attributes",
            format!("{} inherited/synthesized note(s)", attributes.len()),
            NodeKind::Attribute,
        )
        .with_children(attributes, false),
    );
    GraphNode::new(
        "semantic-root",
        "Semantic environment",
        "names · types · scopes · stack layout",
        NodeKind::Root,
    )
    .with_children(children, true)
}

fn scope_node(scope: &Scope, table: &SymbolTable) -> GraphNode {
    let mut children = table
        .symbols
        .iter()
        .filter(|symbol| symbol.scope_id == scope.id)
        .map(|symbol| {
            let (kind, detail) = match &symbol.kind {
                SymbolKind::Function if symbol.name == "print" => (
                    NodeKind::Function,
                    "built-in · variadic print(i64, ...)".to_string(),
                ),
                SymbolKind::Function => (
                    NodeKind::Function,
                    format!(
                        "fn({}) · line {}",
                        symbol.params.join(", "),
                        symbol.declaration_line
                    ),
                ),
                SymbolKind::Parameter => (
                    NodeKind::Variable,
                    format!("parameter · {} · {}", symbol.ty, slot(symbol)),
                ),
                SymbolKind::Variable => (
                    NodeKind::Variable,
                    format!("variable · {} · {}", symbol.ty, slot(symbol)),
                ),
            };
            GraphNode::new(
                format!("symbol:{}:{}", symbol.scope_id, symbol.name),
                symbol.name.clone(),
                detail,
                kind,
            )
        })
        .collect::<Vec<_>>();
    children.extend(
        table
            .scopes
            .iter()
            .filter(|child| child.parent == Some(scope.id))
            .map(|child| scope_node(child, table)),
    );
    let default_expanded = scope.id == 1;
    GraphNode::new(
        format!("scope:{}", scope.id),
        format!("Scope {}", scope.id),
        scope.label.clone(),
        NodeKind::Scope,
    )
    .with_children(children, default_expanded)
}

fn slot(symbol: &crate::common::symbol_table::Symbol) -> String {
    symbol
        .stack_offset
        .map(|offset| format!("[rbp - {offset}]"))
        .unwrap_or_else(|| "stack slot pending".to_string())
}
