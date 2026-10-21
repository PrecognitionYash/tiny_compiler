use crate::phase2_syntax::grammar::{Expr, ExprKind, Function, Program, Stmt, UnaryOp};
use crate::visualizer::state::CompilerSnapshot;
use crate::visualizer::views::graph_canvas::{show_tree, GraphNode, GraphState, NodeKind};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, snapshot: &CompilerSnapshot, state: &mut GraphState) {
    ui.heading("Phase 2 · Abstract syntax tree");
    ui.label("A node is a language construct. Follow the lines from the program to functions, statements, variables, and expressions. Click +/− nodes to expand or collapse; use the scrollbars and zoom controls to explore larger trees.");
    ui.horizontal_wrapped(|ui| {
        legend(ui, "Program / scope", Color32::from_rgb(167, 139, 250));
        legend(ui, "Function", Color32::from_rgb(96, 165, 250));
        legend(ui, "Variable", Color32::from_rgb(45, 212, 191));
        legend(ui, "Literal", Color32::from_rgb(251, 191, 36));
        legend(ui, "Control flow", Color32::from_rgb(244, 114, 182));
    });
    ui.add_space(8.0);
    if let Some(program) = &snapshot.ast {
        let root = program_node(program);
        show_tree(ui, "ast", &root, state);
    } else {
        ui.weak("AST is unavailable because parsing did not complete.");
    }
}

use eframe::egui::Color32;
fn legend(ui: &mut egui::Ui, title: &str, color: Color32) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, color);
        ui.label(title);
    });
}

fn program_node(program: &Program) -> GraphNode {
    let children = program.functions.iter().map(function_node).collect();
    GraphNode::new(
        "program",
        "Program",
        format!("{} function(s)", program.functions.len()),
        NodeKind::Root,
    )
    .with_children(children, true)
}

fn function_node(function: &Function) -> GraphNode {
    let mut children: Vec<GraphNode> = function
        .params
        .iter()
        .enumerate()
        .map(|(index, name)| {
            GraphNode::new(
                format!("fn:{}:param:{}", function.name, index),
                format!("Parameter · {name}"),
                "binding · i64",
                NodeKind::Variable,
            )
        })
        .collect();
    children.extend(
        function
            .body
            .iter()
            .enumerate()
            .map(|(index, stmt)| stmt_node(stmt, &format!("fn:{}:stmt:{}", function.name, index))),
    );
    GraphNode::new(
        format!("fn:{}", function.name),
        format!("def {}({})", function.name, function.params.join(", ")),
        format!("function · line {}", function.line),
        NodeKind::Function,
    )
    .with_children(children, true)
}

fn stmt_node(stmt: &Stmt, id: &str) -> GraphNode {
    match stmt {
        Stmt::Assign { name, value, line } => GraphNode::new(
            id,
            "Assignment",
            format!("line {line} · binds an i64 value"),
            NodeKind::Control,
        )
        .with_children(
            vec![
                GraphNode::new(
                    format!("{id}:target"),
                    format!("Variable · {name}"),
                    format!("assignment target · i64 · line {line}"),
                    NodeKind::Variable,
                ),
                expr_node(value, &format!("{id}:rhs")),
            ],
            true,
        ),
        Stmt::Return { value, line } => GraphNode::new(
            id,
            "Return",
            format!("line {line} · returns i64"),
            NodeKind::Control,
        )
        .with_children(vec![expr_node(value, &format!("{id}:value"))], true),
        Stmt::Expr { value, line } => GraphNode::new(
            id,
            "Expression statement",
            format!("line {line}"),
            NodeKind::Control,
        )
        .with_children(vec![expr_node(value, &format!("{id}:value"))], true),
        Stmt::If {
            condition,
            then_body,
            else_body,
            line,
        } => {
            let mut children = vec![expr_node(condition, &format!("{id}:condition"))];
            children.push(
                GraphNode::new(
                    format!("{id}:then"),
                    "Then branch",
                    format!("{} statement(s)", then_body.len()),
                    NodeKind::Control,
                )
                .with_children(
                    then_body
                        .iter()
                        .enumerate()
                        .map(|(i, s)| stmt_node(s, &format!("{id}:then:{i}")))
                        .collect(),
                    false,
                ),
            );
            if !else_body.is_empty() {
                children.push(
                    GraphNode::new(
                        format!("{id}:else"),
                        "Else branch",
                        format!("{} statement(s)", else_body.len()),
                        NodeKind::Control,
                    )
                    .with_children(
                        else_body
                            .iter()
                            .enumerate()
                            .map(|(i, s)| stmt_node(s, &format!("{id}:else:{i}")))
                            .collect(),
                        false,
                    ),
                );
            }
            GraphNode::new(
                id,
                "If / Else",
                format!("line {line} · nonzero = true"),
                NodeKind::Control,
            )
            .with_children(children, true)
        }
        Stmt::For {
            variable,
            start,
            end,
            body,
            line,
        } => {
            let children = vec![
                GraphNode::new(
                    format!("{id}:loop-var"),
                    format!("Loop variable · {variable}"),
                    "i64 · updated each iteration",
                    NodeKind::Variable,
                ),
                expr_node(start, &format!("{id}:start")),
                expr_node(end, &format!("{id}:end")),
                GraphNode::new(
                    format!("{id}:body"),
                    "Loop body",
                    format!("{} statement(s)", body.len()),
                    NodeKind::Control,
                )
                .with_children(
                    body.iter()
                        .enumerate()
                        .map(|(i, s)| stmt_node(s, &format!("{id}:body:{i}")))
                        .collect(),
                    false,
                ),
            ];
            GraphNode::new(
                id,
                "For · range",
                format!("line {line} · end is exclusive"),
                NodeKind::Control,
            )
            .with_children(children, true)
        }
    }
}

fn expr_node(expr: &Expr, id: &str) -> GraphNode {
    match &expr.kind {
        ExprKind::Integer(value) => GraphNode::new(
            id,
            format!("{value}"),
            format!("integer literal · i64 · line {}", expr.line),
            NodeKind::Literal,
        ),
        ExprKind::Variable(name) => GraphNode::new(
            id,
            format!("Variable · {name}"),
            format!("read · i64 · line {}", expr.line),
            NodeKind::Variable,
        ),
        ExprKind::Call { name, args } => GraphNode::new(
            id,
            format!("Call · {name}()"),
            format!("{} argument(s) · line {}", args.len(), expr.line),
            NodeKind::Function,
        )
        .with_children(
            args.iter()
                .enumerate()
                .map(|(i, arg)| expr_node(arg, &format!("{id}:arg:{i}")))
                .collect(),
            false,
        ),
        ExprKind::Unary { op, operand } => {
            let title = match op {
                UnaryOp::Negate => "Unary · negate",
            };
            GraphNode::new(
                id,
                title,
                format!("line {}", expr.line),
                NodeKind::Expression,
            )
            .with_children(vec![expr_node(operand, &format!("{id}:operand"))], false)
        }
        ExprKind::Binary { op, left, right } => GraphNode::new(
            id,
            format!("Binary · {}", op.text()),
            format!("line {} · precedence resolved", expr.line),
            NodeKind::Expression,
        )
        .with_children(
            vec![
                expr_node(left, &format!("{id}:left")),
                expr_node(right, &format!("{id}:right")),
            ],
            false,
        ),
    }
}
