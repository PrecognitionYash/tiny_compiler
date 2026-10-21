use crate::common::symbol_table::{SymbolKind, SymbolTable};
use crate::phase2_syntax::grammar::{Expr, ExprKind, Program, Stmt};

pub fn evaluate(program: &Program, symbols: &SymbolTable) -> Vec<String> {
    let mut notes = vec![format!(
        "Program attribute: {} function definition(s), all function results synthesize type i64.",
        program.functions.len()
    )];
    for function in &program.functions {
        let locals = symbols
            .symbols
            .iter()
            .filter(|s| {
                s.owner_function.as_deref() == Some(function.name.as_str())
                    && s.kind != SymbolKind::Function
            })
            .count();
        notes.push(format!("{}: {} parameter(s), {} statement(s) at function top level, {} local/parameter symbol(s).", function.name, function.params.len(), function.body.len(), locals));
        for stmt in &function.body {
            summarize_stmt(stmt, &mut notes);
        }
    }
    notes.push("SDD convention used by this teaching compiler: variable/expression type is synthesized as i64; function signatures and visible bindings are inherited from the symbol environment; control-flow statements synthesize the intersection of definitely initialized names.".to_string());
    notes
}

fn summarize_stmt(stmt: &Stmt, notes: &mut Vec<String>) {
    match stmt {
        Stmt::Assign { name, value, line } => notes.push(format!("line {}: assignment to '{}' evaluates an RHS ({}) first, then synthesizes an i64 binding/value.", line, name, expr_kind_name(value))),
        Stmt::Return { value, line } => notes.push(format!("line {}: return expression ({}) has synthesized type i64.", line, expr_kind_name(value))),
        Stmt::If { condition, then_body, else_body, line } => {
            notes.push(format!("line {}: if-condition attribute is truthy iff its i64 value is nonzero; then={} statement(s), else={} statement(s).", line, then_body.len(), else_body.len()));
            let _ = condition;
            for child in then_body { summarize_stmt(child, notes); }
            for child in else_body { summarize_stmt(child, notes); }
        }
        Stmt::For { variable, start, end, body, line } => {
            notes.push(format!("line {}: range loop binds '{}' as i64 and iterates while current < exclusive_end.", line, variable));
            let _ = (start, end);
            for child in body { summarize_stmt(child, notes); }
        }
        Stmt::Expr { value, line } => notes.push(format!("line {}: expression statement has synthesized type {}.", line, expr_kind_name(value))),
    }
}

fn expr_kind_name(expr: &Expr) -> &'static str {
    match &expr.kind {
        ExprKind::Call { .. } => "i64 call result",
        ExprKind::Binary { .. } => "i64 expression",
        ExprKind::Unary { .. } => "i64 expression",
        ExprKind::Integer(_) => "i64 literal",
        ExprKind::Variable(_) => "i64 variable",
    }
}
