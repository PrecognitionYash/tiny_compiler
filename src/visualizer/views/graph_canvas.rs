use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use std::collections::HashSet;

const NODE_W: f32 = 220.0;
const NODE_H: f32 = 90.0;
const H_GAP: f32 = 30.0;
const V_GAP: f32 = 118.0;

#[derive(Clone, Copy, Debug)]
pub enum NodeKind {
    Root,
    Function,
    Variable,
    Literal,
    Expression,
    Control,
    Scope,
    Attribute,
}

#[derive(Clone, Debug)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub kind: NodeKind,
    pub children: Vec<GraphNode>,
    pub default_expanded: bool,
}

impl GraphNode {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        detail: impl Into<String>,
        kind: NodeKind,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            detail: detail.into(),
            kind,
            children: Vec::new(),
            default_expanded: false,
        }
    }
    pub fn with_children(mut self, children: Vec<GraphNode>, default_expanded: bool) -> Self {
        self.children = children;
        self.default_expanded = default_expanded;
        self
    }
}

#[derive(Debug, Default)]
pub struct GraphState {
    expanded: HashSet<String>,
    selected: Option<String>,
    initialized: bool,
    pub zoom: f32,
}

impl GraphState {
    pub fn reset(&mut self) {
        self.expanded.clear();
        self.selected = None;
        self.initialized = false;
        self.zoom = 1.0;
    }
}

fn color(kind: NodeKind) -> (Color32, Color32) {
    match kind {
        NodeKind::Root => (
            Color32::from_rgb(48, 45, 79),
            Color32::from_rgb(167, 139, 250),
        ),
        NodeKind::Function => (
            Color32::from_rgb(32, 57, 83),
            Color32::from_rgb(96, 165, 250),
        ),
        NodeKind::Variable => (
            Color32::from_rgb(23, 66, 63),
            Color32::from_rgb(45, 212, 191),
        ),
        NodeKind::Literal => (
            Color32::from_rgb(80, 55, 26),
            Color32::from_rgb(251, 191, 36),
        ),
        NodeKind::Expression => (
            Color32::from_rgb(55, 48, 84),
            Color32::from_rgb(196, 181, 253),
        ),
        NodeKind::Control => (
            Color32::from_rgb(81, 39, 61),
            Color32::from_rgb(244, 114, 182),
        ),
        NodeKind::Scope => (
            Color32::from_rgb(35, 56, 75),
            Color32::from_rgb(125, 211, 252),
        ),
        NodeKind::Attribute => (
            Color32::from_rgb(43, 61, 45),
            Color32::from_rgb(134, 239, 172),
        ),
    }
}

fn expanded(node: &GraphNode, state: &GraphState) -> bool {
    state.expanded.contains(&node.id)
}

fn tree_width(node: &GraphNode, state: &GraphState) -> f32 {
    if node.children.is_empty() || !expanded(node, state) {
        return NODE_W;
    }
    let children_width: f32 = node
        .children
        .iter()
        .map(|child| tree_width(child, state))
        .sum::<f32>()
        + H_GAP * node.children.len().saturating_sub(1) as f32;
    NODE_W.max(children_width)
}

fn max_depth(node: &GraphNode, state: &GraphState, depth: usize) -> usize {
    if node.children.is_empty() || !expanded(node, state) {
        depth
    } else {
        node.children
            .iter()
            .map(|child| max_depth(child, state, depth + 1))
            .max()
            .unwrap_or(depth)
    }
}

#[derive(Clone)]
struct Positioned<'a> {
    node: &'a GraphNode,
    rect: Rect,
}

fn place<'a>(
    node: &'a GraphNode,
    state: &GraphState,
    left: f32,
    depth: usize,
    top: f32,
    zoom: f32,
    output: &mut Vec<Positioned<'a>>,
) {
    let width = tree_width(node, state);
    let node_w = NODE_W * zoom;
    let node_h = NODE_H * zoom;
    let center_x = left + width * zoom * 0.5;
    let y = top + depth as f32 * V_GAP * zoom;
    output.push(Positioned {
        node,
        rect: Rect::from_center_size(
            Pos2::new(center_x, y + node_h * 0.5),
            Vec2::new(node_w, node_h),
        ),
    });
    if node.children.is_empty() || !expanded(node, state) {
        return;
    }
    let widths: Vec<f32> = node
        .children
        .iter()
        .map(|child| tree_width(child, state))
        .collect();
    let total = widths.iter().sum::<f32>() + H_GAP * node.children.len().saturating_sub(1) as f32;
    let mut child_left = left + (width - total) * zoom * 0.5;
    for (child, child_width) in node.children.iter().zip(widths) {
        place(child, state, child_left, depth + 1, top, zoom, output);
        child_left += (child_width + H_GAP) * zoom;
    }
}

fn collect<'a>(node: &'a GraphNode, out: &mut Vec<&'a GraphNode>) {
    out.push(node);
    for child in &node.children {
        collect(child, out);
    }
}

pub fn show_tree(ui: &mut egui::Ui, id: &'static str, root: &GraphNode, state: &mut GraphState) {
    if !state.initialized {
        fn set_defaults(node: &GraphNode, expanded: &mut HashSet<String>) {
            if node.default_expanded {
                expanded.insert(node.id.clone());
            }
            for child in &node.children {
                set_defaults(child, expanded);
            }
        }
        set_defaults(root, &mut state.expanded);
        state.initialized = true;
        if state.zoom <= 0.0 {
            state.zoom = 1.0;
        }
    }

    ui.horizontal_wrapped(|ui| {
        ui.label("Graph controls");
        if ui.button("− Zoom").clicked() {
            state.zoom = (state.zoom / 1.15).clamp(0.45, 1.8);
        }
        if ui.button("Reset view").clicked() {
            state.reset();
        }
        if ui.button("+ Zoom").clicked() {
            state.zoom = (state.zoom * 1.15).clamp(0.45, 1.8);
        }
        if ui.button("Collapse all").clicked() {
            state.expanded.clear();
            state.expanded.insert(root.id.clone());
        }
        if ui.button("Expand all").clicked() {
            fn open(n: &GraphNode, s: &mut HashSet<String>) {
                if !n.children.is_empty() {
                    s.insert(n.id.clone());
                }
                for c in &n.children {
                    open(c, s);
                }
            }
            open(root, &mut state.expanded);
        }
        ui.weak(
            "Click a node to expand/collapse it. Scroll in either direction to pan the canvas.",
        );
    });

    let zoom = state.zoom.clamp(0.45, 1.8);
    let world_w = tree_width(root, state) * zoom + 120.0;
    let available = ui.available_width().max(320.0);
    let canvas_w = world_w.max(available);
    let depth = max_depth(root, state, 0);
    let canvas_h = (depth as f32 * V_GAP + NODE_H + 100.0) * zoom;
    egui::Frame::new()
        .fill(Color32::from_rgb(18, 22, 31))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            egui::ScrollArea::both()
                .id_salt((id, "graph-scroll"))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let (canvas, _) =
                        ui.allocate_exact_size(Vec2::new(canvas_w, canvas_h), Sense::hover());
                    let painter = ui.painter_at(canvas);
                    painter.rect_filled(
                        canvas,
                        egui::CornerRadius::same(8),
                        Color32::from_rgb(18, 22, 31),
                    );
                    let origin_left = canvas.left()
                        + 60.0
                        + (canvas_w - 120.0 - tree_width(root, state) * zoom).max(0.0) * 0.5;
                    let mut nodes = Vec::new();
                    place(
                        root,
                        state,
                        origin_left,
                        0,
                        canvas.top() + 18.0,
                        zoom,
                        &mut nodes,
                    );
                    let by_id: std::collections::HashMap<&str, Rect> =
                        nodes.iter().map(|p| (p.node.id.as_str(), p.rect)).collect();
                    for p in &nodes {
                        if expanded(p.node, state) {
                            for child in &p.node.children {
                                if let Some(target) = by_id.get(child.id.as_str()) {
                                    let from = Pos2::new(p.rect.center().x, p.rect.bottom());
                                    let to = Pos2::new(target.center().x, target.top());
                                    let mid = (from.y + to.y) * 0.5;
                                    let shape = egui::epaint::CubicBezierShape::from_points_stroke(
                                        [from, Pos2::new(from.x, mid), Pos2::new(to.x, mid), to],
                                        false,
                                        Color32::TRANSPARENT,
                                        Stroke::new(1.5, Color32::from_rgb(91, 111, 145)),
                                    );
                                    painter.add(shape);
                                }
                            }
                        }
                    }
                    for p in &nodes {
                        let (fill, edge) = color(p.node.kind);
                        let selected = state.selected.as_deref() == Some(p.node.id.as_str());
                        painter.rect_filled(p.rect, egui::CornerRadius::same(10), fill);
                        painter.rect_stroke(
                            p.rect,
                            egui::CornerRadius::same(10),
                            Stroke::new(
                                if selected { 2.5 } else { 1.3 },
                                if selected { Color32::WHITE } else { edge },
                            ),
                            egui::StrokeKind::Inside,
                        );
                        let title_pos = p.rect.left_top() + Vec2::new(10.0 * zoom, 8.0 * zoom);
                        let title_limit = ((p.rect.width() - 20.0 * zoom) / (7.2 * zoom))
                            .floor()
                            .max(8.0) as usize;
                        let title = if p.node.title.chars().count() > title_limit {
                            format!(
                                "{}…",
                                p.node
                                    .title
                                    .chars()
                                    .take(title_limit.saturating_sub(1))
                                    .collect::<String>()
                            )
                        } else {
                            p.node.title.clone()
                        };
                        painter.text(
                            title_pos,
                            Align2::LEFT_TOP,
                            title,
                            FontId::proportional(12.5 * zoom),
                            Color32::from_rgb(245, 247, 252),
                        );
                        let detail_pos = p.rect.left_top() + Vec2::new(10.0 * zoom, 34.0 * zoom);
                        let detail_preview = if p.node.detail.chars().count() > 44 {
                            format!("{}…", p.node.detail.chars().take(43).collect::<String>())
                        } else {
                            p.node.detail.clone()
                        };
                        let detail_galley = painter.layout(
                            detail_preview,
                            FontId::monospace(9.5 * zoom),
                            Color32::from_rgb(193, 204, 220),
                            (p.rect.width() - 20.0 * zoom).max(40.0),
                        );
                        painter.galley(detail_pos, detail_galley, Color32::from_rgb(193, 204, 220));
                        if !p.node.children.is_empty() {
                            let marker = if expanded(p.node, state) { "−" } else { "+" };
                            painter.text(
                                p.rect.right_top() - Vec2::new(18.0 * zoom, -8.0 * zoom),
                                Align2::RIGHT_TOP,
                                marker,
                                FontId::proportional(16.0 * zoom),
                                edge,
                            );
                        }
                        let response = ui.interact(
                            p.rect,
                            ui.make_persistent_id((id, &p.node.id)),
                            Sense::click(),
                        );
                        if response.clicked() {
                            state.selected = Some(p.node.id.clone());
                            if !p.node.children.is_empty() && !state.expanded.remove(&p.node.id) {
                                state.expanded.insert(p.node.id.clone());
                            }
                        }
                        if response.hovered() {
                            response.on_hover_text(if p.node.children.is_empty() {
                                p.node.detail.clone()
                            } else {
                                format!(
                                    "{}\nClick to {}",
                                    p.node.title,
                                    if expanded(p.node, state) {
                                        "collapse children"
                                    } else {
                                        "expand children"
                                    }
                                )
                            });
                        }
                    }
                });
        });
    if let Some(selected) = &state.selected {
        let mut all = Vec::new();
        collect(root, &mut all);
        if let Some(node) = all.into_iter().find(|node| &node.id == selected) {
            let (fill, edge) = color(node.kind);
            egui::Frame::new()
                .fill(fill)
                .stroke(Stroke::new(1.0, edge))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("Selected node · {}", node.title))
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.label(node.detail.as_str());
                    ui.label(format!("{} child node(s)", node.children.len()));
                });
        }
    }
}
