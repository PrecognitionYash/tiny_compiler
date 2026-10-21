use crate::phase5_optimization::basic_blocks::BasicBlock;
use crate::visualizer::state::CompilerSnapshot;
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use std::collections::HashMap;

pub fn show(
    ui: &mut egui::Ui,
    snapshot: &CompilerSnapshot,
    selected: &mut Option<usize>,
    zoom: &mut f32,
) {
    ui.heading("Phase 5 · Optimization + control-flow graph");
    ui.label("Basic blocks are arranged in a single readable flow lane. Straight-line edges stay central; branch and loop edges get their own side lanes to avoid crossing through nodes.");
    ui.horizontal_wrapped(|ui| {
        metric(
            ui,
            "Basic blocks",
            snapshot.basic_blocks.len().to_string(),
            Color32::from_rgb(36, 57, 81),
            Color32::from_rgb(125, 211, 252),
        );
        metric(
            ui,
            "Edges",
            snapshot
                .basic_blocks
                .iter()
                .map(|b| b.successors.len())
                .sum::<usize>()
                .to_string(),
            Color32::from_rgb(42, 61, 49),
            Color32::from_rgb(134, 239, 172),
        );
        if ui.button("− Zoom").clicked() {
            *zoom = (*zoom / 1.15).clamp(0.55, 1.6);
        }
        if ui.button("Reset view").clicked() {
            *zoom = 1.0;
        }
        if ui.button("+ Zoom").clicked() {
            *zoom = (*zoom * 1.15).clamp(0.55, 1.6);
        }
    });
    ui.horizontal_wrapped(|ui| {
        edge_legend(ui, "Fall-through", Color32::from_rgb(102, 190, 165));
        edge_legend(ui, "Forward branch", Color32::from_rgb(244, 114, 182));
        edge_legend(ui, "Loop / back edge", Color32::from_rgb(251, 191, 36));
    });
    ui.label("Scroll horizontally to inspect branch lanes and vertically to follow long functions. Click a block to inspect its instructions and predecessor/successor links.");
    ui.separator();
    if snapshot.basic_blocks.is_empty() {
        ui.weak("Control-flow graph is unavailable because the intermediate representation did not complete.");
        return;
    }
    draw_graph(ui, &snapshot.basic_blocks, selected, *zoom);
    if let Some(index) = *selected {
        if let Some(block) = snapshot.basic_blocks.iter().find(|b| b.id == index) {
            ui.add_space(8.0);
            egui::Frame::new()
                .fill(Color32::from_rgb(29, 39, 55))
                .stroke(Stroke::new(1.0, Color32::from_rgb(96, 165, 250)))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.heading(format!("B{} · {}", block.id, block.function));
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!(
                            "TAC indexes: {}..{}",
                            block.start_index, block.end_index
                        ));
                        ui.label(format!("Predecessors: {}", ids(&block.predecessors)));
                        ui.label(format!("Successors: {}", ids(&block.successors)));
                    });
                    egui::Grid::new("selected_block_quads")
                        .striped(true)
                        .num_columns(2)
                        .spacing([14.0, 7.0])
                        .show(ui, |ui| {
                            ui.strong("Local index");
                            ui.strong("Quadruple");
                            ui.end_row();
                            for (i, q) in block.quads.iter().enumerate() {
                                ui.label(i.to_string());
                                ui.monospace(q.to_string());
                                ui.end_row();
                            }
                        });
                });
        }
    }
}

fn edge_legend(ui: &mut egui::Ui, label: &str, color: Color32) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 10.0), Sense::hover());
        ui.painter().line_segment(
            [
                Pos2::new(rect.left(), rect.center().y),
                Pos2::new(rect.right(), rect.center().y),
            ],
            Stroke::new(2.0, color),
        );
        ui.label(label);
    });
}

fn metric(ui: &mut egui::Ui, title: &str, value: String, fill: Color32, accent: Color32) {
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, accent))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).color(Color32::LIGHT_GRAY));
            ui.label(egui::RichText::new(value).size(18.0).strong().color(accent));
        });
}

fn ids(values: &[usize]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values
            .iter()
            .map(|v| format!("B{}", v))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn draw_graph(ui: &mut egui::Ui, blocks: &[BasicBlock], selected: &mut Option<usize>, zoom: f32) {
    let node_w = 350.0 * zoom;
    let node_h = 112.0 * zoom;
    let row_gap = 82.0 * zoom;
    let lane_gap = 27.0 * zoom;
    let mut forward_edges = Vec::new();
    let mut backward_edges = Vec::new();
    let mut straight_edges = Vec::new();
    for block in blocks {
        for target in &block.successors {
            if *target == block.id + 1 {
                straight_edges.push((block.id, *target));
            } else if *target > block.id {
                forward_edges.push((block.id, *target));
            } else {
                backward_edges.push((block.id, *target));
            }
        }
    }
    let left_space = 80.0 * zoom + backward_edges.len() as f32 * lane_gap;
    let right_space = 80.0 * zoom + forward_edges.len() as f32 * lane_gap;
    let group_width = left_space + node_w + right_space;
    let width = ui.available_width().max(group_width + 80.0 * zoom);
    let height = 40.0 * zoom + blocks.len() as f32 * (node_h + row_gap) - row_gap + 40.0 * zoom;
    egui::Frame::new()
        .fill(Color32::from_rgb(18, 22, 31))
        .stroke(Stroke::new(1.0, Color32::from_rgb(49, 62, 82)))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            egui::ScrollArea::both()
                .id_salt("cfg-canvas-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let (canvas, _) =
                        ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
                    let painter = ui.painter_at(canvas);
                    painter.rect_filled(
                        canvas,
                        egui::CornerRadius::same(8),
                        Color32::from_rgb(18, 22, 31),
                    );
                    let graph_width = group_width;
                    let start_x = canvas.left() + (width - graph_width).max(0.0) * 0.5;
                    let center_x = start_x + left_space + node_w * 0.5;
                    let mut positions: HashMap<usize, Rect> = HashMap::new();
                    for (index, block) in blocks.iter().enumerate() {
                        let y = canvas.top() + 20.0 * zoom + index as f32 * (node_h + row_gap);
                        positions.insert(
                            block.id,
                            Rect::from_center_size(
                                Pos2::new(center_x, y + node_h * 0.5),
                                Vec2::new(node_w, node_h),
                            ),
                        );
                    }
                    // Every non-linear edge gets a dedicated outer lane to avoid clumsy line overlap
                    // through basic block cards. Took 2 hours to fix this graph routing logic!
                    for (lane, edge) in backward_edges.iter().enumerate() {
                        let (from_id, to_id) = *edge;
                        if let (Some(from_rect), Some(to_rect)) =
                            (positions.get(&from_id), positions.get(&to_id))
                        {
                            let from_port = edge_port(&backward_edges, edge, from_id, true);
                            let to_port = edge_port(&backward_edges, edge, to_id, false);
                            let lane_x =
                                center_x - node_w * 0.5 - 12.0 * zoom - lane as f32 * lane_gap;
                            draw_routed_edge(
                                &painter,
                                *from_rect,
                                *to_rect,
                                EdgeRoute {
                                    lane_x,
                                    forward: false,
                                    from_port,
                                    to_port,
                                    zoom,
                                },
                            );
                        }
                    }
                    for (lane, edge) in forward_edges.iter().enumerate() {
                        let (from_id, to_id) = *edge;
                        if let (Some(from_rect), Some(to_rect)) =
                            (positions.get(&from_id), positions.get(&to_id))
                        {
                            let from_port = edge_port(&forward_edges, edge, from_id, true);
                            let to_port = edge_port(&forward_edges, edge, to_id, false);
                            let lane_x =
                                center_x + node_w * 0.5 + 12.0 * zoom + lane as f32 * lane_gap;
                            draw_routed_edge(
                                &painter,
                                *from_rect,
                                *to_rect,
                                EdgeRoute {
                                    lane_x,
                                    forward: true,
                                    from_port,
                                    to_port,
                                    zoom,
                                },
                            );
                        }
                    }
                    for (from_id, to_id) in straight_edges {
                        if let (Some(from_rect), Some(to_rect)) =
                            (positions.get(&from_id), positions.get(&to_id))
                        {
                            let start = Pos2::new(from_rect.center().x, from_rect.bottom());
                            let end = Pos2::new(to_rect.center().x, to_rect.top());
                            painter.line_segment(
                                [start, end],
                                Stroke::new(1.7, Color32::from_rgb(102, 190, 165)),
                            );
                            arrow_head(
                                &painter,
                                end,
                                Vec2::new(0.0, 1.0),
                                Color32::from_rgb(102, 190, 165),
                            );
                        }
                    }
                    for block in blocks {
                        let Some(rect) = positions.get(&block.id).copied() else {
                            continue;
                        };
                        let chosen = *selected == Some(block.id);
                        let fill = if chosen {
                            Color32::from_rgb(37, 67, 94)
                        } else {
                            Color32::from_rgb(29, 38, 53)
                        };
                        let edge = if chosen {
                            Color32::from_rgb(147, 197, 253)
                        } else {
                            Color32::from_rgb(83, 110, 145)
                        };
                        painter.rect_filled(rect, egui::CornerRadius::same(9), fill);
                        painter.rect_stroke(
                            rect,
                            egui::CornerRadius::same(9),
                            Stroke::new(if chosen { 2.3 } else { 1.2 }, edge),
                            egui::StrokeKind::Inside,
                        );
                        painter.text(
                            rect.left_top() + Vec2::new(12.0 * zoom, 9.0 * zoom),
                            Align2::LEFT_TOP,
                            format!("B{} · {}", block.id, block.function),
                            FontId::proportional(14.0 * zoom),
                            Color32::from_rgb(236, 243, 252),
                        );
                        for (i, q) in block.quads.iter().take(4).enumerate() {
                            painter.text(
                                rect.left_top()
                                    + Vec2::new(12.0 * zoom, (32.0 + i as f32 * 17.0) * zoom),
                                Align2::LEFT_TOP,
                                compact_quad(q),
                                FontId::monospace(10.5 * zoom),
                                opcode_color(&q.op),
                            );
                        }
                        if block.quads.len() > 4 {
                            painter.text(
                                rect.left_bottom() - Vec2::new(-12.0 * zoom, 6.0 * zoom),
                                Align2::LEFT_BOTTOM,
                                format!("+{} more instruction(s)", block.quads.len() - 4),
                                FontId::proportional(10.0 * zoom),
                                Color32::LIGHT_GRAY,
                            );
                        }
                        let response = ui.interact(
                            rect,
                            ui.make_persistent_id(("cfg-block", block.id)),
                            Sense::click(),
                        );
                        if response.clicked() {
                            *selected = Some(block.id);
                        }
                        if response.hovered() {
                            response.on_hover_text(format!(
                                "B{} · {} instruction(s)\nPredecessors: {}\nSuccessors: {}",
                                block.id,
                                block.quads.len(),
                                ids(&block.predecessors),
                                ids(&block.successors)
                            ));
                        }
                    }
                });
        });
}

fn edge_port(
    edges: &[(usize, usize)],
    edge: &(usize, usize),
    block_id: usize,
    source: bool,
) -> (usize, usize) {
    let adjacent: Vec<&(usize, usize)> = edges
        .iter()
        .filter(|candidate| {
            if source {
                candidate.0 == block_id
            } else {
                candidate.1 == block_id
            }
        })
        .collect();
    let rank = adjacent
        .iter()
        .position(|candidate| **candidate == *edge)
        .unwrap_or(0);
    (rank, adjacent.len().max(1))
}

fn port_y(rect: Rect, port: (usize, usize), zoom: f32) -> f32 {
    let (rank, count) = port;
    let centered = rank as f32 - (count.saturating_sub(1) as f32 * 0.5);
    let max_offset = rect.height() * 0.34;
    rect.center().y + (centered * 15.0 * zoom).clamp(-max_offset, max_offset)
}

#[derive(Clone, Copy)]
struct EdgeRoute {
    lane_x: f32,
    forward: bool,
    from_port: (usize, usize),
    to_port: (usize, usize),
    zoom: f32,
}

fn draw_routed_edge(painter: &egui::Painter, from: Rect, to: Rect, route: EdgeRoute) {
    let start_y = port_y(from, route.from_port, route.zoom);
    let end_y = port_y(to, route.to_port, route.zoom);
    let (start, end) = if route.forward {
        (
            Pos2::new(from.right(), start_y),
            Pos2::new(to.right(), end_y),
        )
    } else {
        (Pos2::new(from.left(), start_y), Pos2::new(to.left(), end_y))
    };
    let color = if route.forward {
        Color32::from_rgb(244, 114, 182)
    } else {
        Color32::from_rgb(251, 191, 36)
    };
    let shape = egui::epaint::CubicBezierShape::from_points_stroke(
        [
            start,
            Pos2::new(route.lane_x, start.y),
            Pos2::new(route.lane_x, end.y),
            end,
        ],
        false,
        Color32::TRANSPARENT,
        Stroke::new(1.6, color),
    );
    painter.add(shape);
    let dir = if route.forward {
        Vec2::new(-1.0, 0.0)
    } else {
        Vec2::new(1.0, 0.0)
    };
    arrow_head(painter, end, dir, color);
}

fn arrow_head(painter: &egui::Painter, tip: Pos2, direction: Vec2, color: Color32) {
    let dir = direction.normalized();
    let side = Vec2::new(-dir.y, dir.x);
    painter.line_segment([tip, tip - dir * 8.0 + side * 4.0], Stroke::new(1.5, color));
    painter.line_segment([tip, tip - dir * 8.0 - side * 4.0], Stroke::new(1.5, color));
}

fn opcode_color(op: &str) -> Color32 {
    if matches!(
        op,
        "ADD" | "SUB" | "MUL" | "DIV" | "AND" | "OR" | "XOR" | "SHL" | "SHR"
    ) {
        Color32::from_rgb(251, 191, 36)
    } else if matches!(op, "CALL" | "PRINT") {
        Color32::from_rgb(134, 239, 172)
    } else if op.starts_with("BR_") || matches!(op, "GOTO" | "LABEL") {
        Color32::from_rgb(244, 114, 182)
    } else {
        Color32::from_rgb(190, 212, 238)
    }
}

fn compact_quad(q: &crate::phase4_intermediate::three_address_code::Quadruple) -> String {
    let text = q.to_string();
    if text.chars().count() > 44 {
        format!("{}…", text.chars().take(41).collect::<String>())
    } else {
        text
    }
}
