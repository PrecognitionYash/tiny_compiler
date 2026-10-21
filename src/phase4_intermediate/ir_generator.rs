use super::three_address_code::Quadruple;
use crate::common::error_handler::CompileResult;
use crate::phase2_syntax::grammar::{BinaryOp, Expr, ExprKind, Function, Program, Stmt, UnaryOp};

// Phase 4: Intermediate Code Generation (Three-Address Code TAC).
// Generates quadruples: (op, arg1, arg2, result) with temporary regesters ($t1, $t2...) and jump labels.
#[derive(Default)]
struct Generator {
    code: Vec<Quadruple>,
    temp_counter: usize,
    label_counter: usize,
}

impl Generator {
    fn temp(&mut self) -> String {
        self.temp_counter += 1;
        format!("$t{}", self.temp_counter)
    }
    fn label(&mut self, prefix: &str) -> String {
        self.label_counter += 1;
        format!(".L_tcomp_{}_{}", prefix, self.label_counter)
    }
    fn emit(&mut self, op: &str, arg1: &str, arg2: &str, result: &str, line: usize) {
        self.code.push(Quadruple::new(op, arg1, arg2, result, line));
    }

    fn generate(mut self, program: &Program) -> CompileResult<Vec<Quadruple>> {
        for function in &program.functions {
            self.function(function)?;
        }
        Ok(self.code)
    }

    fn function(&mut self, function: &Function) -> CompileResult<()> {
        self.emit("FUNC", "", "", &function.name, function.line);
        for (index, name) in function.params.iter().enumerate() {
            self.emit("PARAM", &index.to_string(), "", name, function.line);
        }
        self.statements(&function.body)?;
        if !matches!(function.body.last(), Some(Stmt::Return { .. })) {
            self.emit("RET", "0", "", "", function.line);
        }
        self.emit("FUNC_END", "", "", &function.name, function.line);
        Ok(())
    }

    fn statements(&mut self, statements: &[Stmt]) -> CompileResult<()> {
        for stmt in statements {
            self.statement(stmt)?;
        }
        Ok(())
    }

    fn statement(&mut self, stmt: &Stmt) -> CompileResult<()> {
        match stmt {
            Stmt::Assign { name, value, line } => {
                let source = self.expression(value)?;
                self.emit("MOV", &source, "", name, *line);
            }
            Stmt::Return { value, line } => {
                let source = self.expression(value)?;
                self.emit("RET", &source, "", "", *line);
            }
            Stmt::Expr { value, .. } => {
                let _ = self.expression(value)?;
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                line,
            } => {
                let cond = self.expression(condition)?;
                let else_label = self.label("L_if_else");
                let end_label = self.label("L_if_end");
                self.emit("BR_ZERO", &cond, "", &else_label, *line);
                self.statements(then_body)?;
                if !else_body.is_empty() {
                    self.emit("GOTO", "", "", &end_label, *line);
                    self.emit("LABEL", "", "", &else_label, *line);
                    self.statements(else_body)?;
                    self.emit("LABEL", "", "", &end_label, *line);
                } else {
                    self.emit("LABEL", "", "", &else_label, *line);
                }
            }
            Stmt::For {
                variable,
                start,
                end,
                body,
                line,
            } => {
                let start_value = self.expression(start)?;
                let end_value = self.expression(end)?;
                self.emit("MOV", &start_value, "", variable, *line);
                let head = self.label("L_for_head");
                let exit = self.label("L_for_exit");
                self.emit("LABEL", "", "", &head, *line);
                self.emit("BR_GE", variable, &end_value, &exit, *line);
                self.statements(body)?;
                self.emit("ADD", variable, "1", variable, *line);
                self.emit("GOTO", "", "", &head, *line);
                self.emit("LABEL", "", "", &exit, *line);
            }
        }
        Ok(())
    }

    fn expression(&mut self, expr: &Expr) -> CompileResult<String> {
        match &expr.kind {
            ExprKind::Integer(value) => Ok(value.to_string()),
            ExprKind::Variable(name) => Ok(name.clone()),
            ExprKind::Call { name, args } => {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.expression(arg)?);
                }
                if name == "print" {
                    self.emit(
                        "PRINT",
                        &values.len().to_string(),
                        &values.join(","),
                        "",
                        expr.line,
                    );
                    // In our language all experssions are i64. Print call inside a bigger
                    // expression returns 0 after printing to console as side effect.
                    Ok("0".to_string())
                } else {
                    let result = self.temp();
                    self.emit("CALL", name, &values.join(","), &result, expr.line);
                    Ok(result)
                }
            }
            ExprKind::Unary { op, operand } => {
                let source = self.expression(operand)?;
                let result = self.temp();
                match op {
                    UnaryOp::Negate => self.emit("NEG", &source, "", &result, expr.line),
                }
                Ok(result)
            }
            ExprKind::Binary { op, left, right } => {
                let a = self.expression(left)?;
                let b = self.expression(right)?;
                let result = self.temp();
                let opcode = match op {
                    BinaryOp::Add => "ADD",
                    BinaryOp::Subtract => "SUB",
                    BinaryOp::Multiply => "MUL",
                    BinaryOp::Divide => "DIV",
                    BinaryOp::BitAnd => "AND",
                    BinaryOp::BitOr => "OR",
                    BinaryOp::BitXor => "XOR",
                    BinaryOp::ShiftLeft => "SHL",
                    BinaryOp::ShiftRight => "SHR",
                    BinaryOp::Equal => "CMP_EQ",
                    BinaryOp::NotEqual => "CMP_NE",
                    BinaryOp::Less => "CMP_LT",
                    BinaryOp::LessEqual => "CMP_LE",
                    BinaryOp::Greater => "CMP_GT",
                    BinaryOp::GreaterEqual => "CMP_GE",
                };
                self.emit(opcode, &a, &b, &result, expr.line);
                Ok(result)
            }
        }
    }
}

pub fn generate(program: &Program) -> CompileResult<Vec<Quadruple>> {
    Generator::default().generate(program)
}
