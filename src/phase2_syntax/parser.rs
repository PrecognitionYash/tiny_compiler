use super::grammar::{BinaryOp, Expr, ExprKind, Function, Program, Stmt, UnaryOp};
use crate::common::error_handler::{CompileError, CompileResult};
use crate::phase1_lexical::token::{Token, TokenKind};

pub struct Parser<'a> {
    tokens: &'a [Token],
    cursor: usize,
}

// Phase 2: Recursive Descent Parser for building Abstract Syntax Tree (AST).
// Implimenting operator predence climbing grammar rules from Dragon Book.
impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn current(&self) -> &'a Token {
        self.tokens
            .get(self.cursor)
            .unwrap_or_else(|| self.tokens.last().expect("lexer always emits EOF"))
    }
    fn previous(&self) -> &'a Token {
        self.tokens
            .get(self.cursor.saturating_sub(1))
            .unwrap_or(self.current())
    }
    fn check(&self, kind: &TokenKind) -> bool {
        &self.current().kind == kind
    }
    fn advance(&mut self) -> &'a Token {
        let token = self.current();
        if token.kind != TokenKind::Eof {
            self.cursor += 1;
        }
        token
    }
    fn matches(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, kind: TokenKind, expectation: &str) -> CompileResult<&'a Token> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            let token = self.current();
            Err(CompileError::new(
                2,
                token.line,
                token.column,
                format!(
                    "expected {}, found '{}' ({})",
                    expectation, token.lexeme, token.kind
                ),
            ))
        }
    }

    pub fn parse(mut self) -> CompileResult<Program> {
        let mut functions = Vec::new();
        while !self.check(&TokenKind::Eof) {
            while self.matches(&TokenKind::Newline) {}
            if self.check(&TokenKind::Eof) {
                break;
            }
            if !self.check(&TokenKind::Def) {
                let token = self.current();
                return Err(CompileError::new(
                    2,
                    token.line,
                    token.column,
                    "only function definitions are allowed at top level",
                ));
            }
            functions.push(self.parse_function()?);
        }
        if functions.is_empty() {
            return Err(CompileError::new(
                2,
                1,
                1,
                "program must define at least one function",
            ));
        }
        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> CompileResult<Function> {
        let def_token = self.expect(TokenKind::Def, "'def'")?;
        let name_token = self.expect(TokenKind::Identifier, "function name")?;
        let name = name_token.lexeme.clone();
        self.expect(TokenKind::LParen, "'('")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                params.push(
                    self.expect(TokenKind::Identifier, "parameter name")?
                        .lexeme
                        .clone(),
                );
                if !self.matches(&TokenKind::Comma) {
                    break;
                }
                if self.check(&TokenKind::RParen) {
                    let token = self.current();
                    return Err(CompileError::new(
                        2,
                        token.line,
                        token.column,
                        "trailing commas are not supported in parameter lists",
                    ));
                }
            }
        }
        self.expect(TokenKind::RParen, "')'")?;
        self.expect(TokenKind::Colon, "':' after function signature")?;
        let body = self.parse_suite()?;
        Ok(Function {
            name,
            params,
            body,
            line: def_token.line,
        })
    }

    fn parse_suite(&mut self) -> CompileResult<Vec<Stmt>> {
        self.expect(TokenKind::Newline, "newline after ':'")?;
        self.expect(TokenKind::Indent, "an indented block")?;
        let mut statements = Vec::new();
        while !self.check(&TokenKind::Dedent) && !self.check(&TokenKind::Eof) {
            if self.matches(&TokenKind::Newline) {
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenKind::Dedent, "end of indented block")?;
        if statements.is_empty() {
            return Err(CompileError::new(
                2,
                self.previous().line,
                self.previous().column,
                "empty blocks are not allowed",
            ));
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> CompileResult<Stmt> {
        let line = self.current().line;
        if self.matches(&TokenKind::Return) {
            let value = self.parse_expression()?;
            self.expect(TokenKind::Newline, "newline after return expression")?;
            return Ok(Stmt::Return { value, line });
        }
        if self.matches(&TokenKind::If) {
            let condition = self.parse_expression()?;
            self.expect(TokenKind::Colon, "':' after if condition")?;
            let then_body = self.parse_suite()?;
            let else_body = if self.matches(&TokenKind::Else) {
                self.expect(TokenKind::Colon, "':' after else")?;
                self.parse_suite()?
            } else {
                Vec::new()
            };
            return Ok(Stmt::If {
                condition,
                then_body,
                else_body,
                line,
            });
        }
        if self.matches(&TokenKind::For) {
            let variable = self
                .expect(TokenKind::Identifier, "loop variable")?
                .lexeme
                .clone();
            self.expect(TokenKind::In, "'in'")?;
            let start = self.parse_expression()?;
            self.expect(TokenKind::Range, "'..' in range loop")?;
            let end = self.parse_expression()?;
            self.expect(TokenKind::Colon, "':' after range")?;
            let body = self.parse_suite()?;
            return Ok(Stmt::For {
                variable,
                start,
                end,
                body,
                line,
            });
        }
        if self.check(&TokenKind::Identifier)
            && self.tokens.get(self.cursor + 1).map(|t| &t.kind) == Some(&TokenKind::Assign)
        {
            let name = self.advance().lexeme.clone();
            self.advance();
            let value = self.parse_expression()?;
            self.expect(TokenKind::Newline, "newline after assignment")?;
            return Ok(Stmt::Assign { name, value, line });
        }
        let value = self.parse_expression()?;
        self.expect(TokenKind::Newline, "newline after expression")?;
        Ok(Stmt::Expr { value, line })
    }

    fn parse_expression(&mut self) -> CompileResult<Expr> {
        self.parse_bit_or()
    }

    fn parse_equality(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_relational()?;
        loop {
            let op = match self.current().kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::NotEqual => BinaryOp::NotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_relational()?;
            let line = expr.line;
            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                line,
            };
        }
        Ok(expr)
    }

    fn parse_relational(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_shift()?;
        loop {
            let op = match self.current().kind {
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_shift()?;
            let line = expr.line;
            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                line,
            };
        }
        Ok(expr)
    }

    fn parse_bit_or(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_bit_xor()?;
        while self.matches(&TokenKind::Pipe) {
            let right = self.parse_bit_xor()?;
            let line = expr.line;
            expr = Expr {
                kind: ExprKind::Binary {
                    op: BinaryOp::BitOr,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                line,
            };
        }
        Ok(expr)
    }

    fn parse_bit_xor(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_bit_and()?;
        while self.matches(&TokenKind::Caret) {
            let right = self.parse_bit_and()?;
            let line = expr.line;
            expr = Expr {
                kind: ExprKind::Binary {
                    op: BinaryOp::BitXor,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                line,
            };
        }
        Ok(expr)
    }

    fn parse_bit_and(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_equality()?;
        while self.matches(&TokenKind::Amp) {
            let right = self.parse_equality()?;
            let line = expr.line;
            expr = Expr {
                kind: ExprKind::Binary {
                    op: BinaryOp::BitAnd,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                line,
            };
        }
        Ok(expr)
    }

    fn parse_shift(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_additive()?;
        loop {
            let op = if self.matches(&TokenKind::ShiftLeft) {
                Some(BinaryOp::ShiftLeft)
            } else if self.matches(&TokenKind::ShiftRight) {
                Some(BinaryOp::ShiftRight)
            } else {
                None
            };
            let Some(op) = op else {
                break;
            };
            let right = self.parse_additive()?;
            let line = expr.line;
            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                line,
            };
        }
        Ok(expr)
    }

    fn parse_additive(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            let op = if self.matches(&TokenKind::Plus) {
                Some(BinaryOp::Add)
            } else if self.matches(&TokenKind::Minus) {
                Some(BinaryOp::Subtract)
            } else {
                None
            };
            let Some(op) = op else {
                break;
            };
            let right = self.parse_multiplicative()?;
            let line = expr.line;
            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                line,
            };
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = if self.matches(&TokenKind::Star) {
                Some(BinaryOp::Multiply)
            } else if self.matches(&TokenKind::Slash) {
                Some(BinaryOp::Divide)
            } else {
                None
            };
            let Some(op) = op else {
                break;
            };
            let right = self.parse_unary()?;
            let line = expr.line;
            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                line,
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> CompileResult<Expr> {
        if self.matches(&TokenKind::Minus) {
            let line = self.previous().line;
            if self.check(&TokenKind::Integer) && self.current().lexeme == "9223372036854775808" {
                self.advance();
                return Ok(Expr {
                    kind: ExprKind::Integer(i64::MIN),
                    line,
                });
            }
            let operand = self.parse_unary()?;
            return Ok(Expr {
                kind: ExprKind::Unary {
                    op: UnaryOp::Negate,
                    operand: Box::new(operand),
                },
                line,
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> CompileResult<Expr> {
        let token = self.current().clone();
        if self.matches(&TokenKind::Integer) {
            let value = token.lexeme.parse::<i64>().map_err(|_| {
                CompileError::new(2, token.line, token.column, "invalid signed 64-bit integer")
            })?;
            return Ok(Expr {
                kind: ExprKind::Integer(value),
                line: token.line,
            });
        }
        if self.matches(&TokenKind::Identifier) {
            let name = token.lexeme;
            if self.matches(&TokenKind::LParen) {
                let mut args = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.matches(&TokenKind::Comma) {
                            break;
                        }
                        if self.check(&TokenKind::RParen) {
                            let current = self.current();
                            return Err(CompileError::new(
                                2,
                                current.line,
                                current.column,
                                "trailing commas are not supported in argument lists",
                            ));
                        }
                    }
                }
                self.expect(TokenKind::RParen, "')' after arguments")?;
                return Ok(Expr {
                    kind: ExprKind::Call { name, args },
                    line: token.line,
                });
            }
            return Ok(Expr {
                kind: ExprKind::Variable(name),
                line: token.line,
            });
        }
        if self.matches(&TokenKind::LParen) {
            let expr = self.parse_expression()?;
            self.expect(TokenKind::RParen, "')'")?;
            return Ok(expr);
        }
        Err(CompileError::new(
            2,
            token.line,
            token.column,
            format!(
                "expected expression, found '{}' ({})",
                token.lexeme, token.kind
            ),
        ))
    }
}

pub fn parse(tokens: &[Token]) -> CompileResult<Program> {
    Parser::new(tokens).parse()
}
