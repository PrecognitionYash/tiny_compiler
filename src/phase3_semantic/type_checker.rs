use crate::common::error_handler::{CompileError, CompileResult};
use crate::common::symbol_table::{ScopeId, SymbolDefinition, SymbolKind, SymbolTable};
use crate::phase2_syntax::grammar::{Expr, ExprKind, Function, Program, Stmt};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct SemanticOutput {
    pub symbols: SymbolTable,
    pub notes: Vec<String>,
}

// Phase 3 Semantic Analysis: Checking symbol declarations, type consistency, and scopes.
// Verifying variable initilization before use and building scoped symbol table.
struct Analyzer {
    symbols: SymbolTable,
    signatures: HashMap<String, Vec<String>>,
    notes: Vec<String>,
}

impl Analyzer {
    fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            signatures: HashMap::new(),
            notes: Vec::new(),
        }
    }

    fn analyze(mut self, program: &Program) -> CompileResult<SemanticOutput> {
        for function in &program.functions {
            if function.name == "print" {
                return Err(CompileError::new(
                    3,
                    function.line,
                    1,
                    "'print' is a reserved built-in function and cannot be redefined",
                ));
            }
            if self.signatures.contains_key(&function.name) {
                return Err(CompileError::new(
                    3,
                    function.line,
                    1,
                    format!("function '{}' is defined more than once", function.name),
                ));
            }
            let mut seen = HashSet::new();
            for param in &function.params {
                if param == "print" {
                    return Err(CompileError::new(
                        3,
                        function.line,
                        1,
                        "'print' is reserved and cannot be used as a parameter name",
                    ));
                }
                if !seen.insert(param) {
                    return Err(CompileError::new(
                        3,
                        function.line,
                        1,
                        format!("parameter '{}' appears more than once", param),
                    ));
                }
            }
            self.signatures
                .insert(function.name.clone(), function.params.clone());
            self.symbols
                .define(
                    function.name.clone(),
                    "fn",
                    SymbolKind::Function,
                    None,
                    function.params.clone(),
                    function.line,
                )
                .map_err(|message| CompileError::new(3, function.line, 1, message))?;
        }
        self.symbols
            .define(
                "print",
                "builtin(*i64)",
                SymbolKind::Function,
                None,
                vec!["...".to_string()],
                0,
            )
            .map_err(|message| CompileError::new(3, 0, 0, message))?;
        let main = self.signatures.get("main").ok_or_else(|| {
            CompileError::new(3, 0, 0, "program must define entry point 'def main():'")
        })?;
        if !main.is_empty() {
            return Err(CompileError::new(
                3,
                0,
                0,
                "entry point 'main' must not take parameters",
            ));
        }

        for function in &program.functions {
            self.analyze_function(function)?;
        }
        self.notes.push("All language values have type i64; comparison expressions produce 0 (false) or 1 (true).".to_string());
        self.notes.push("Assignments introduce function-local bindings on first assignment. Nested blocks have separate scope records; a binding remains function-local, while definite initialization is checked along control-flow paths.".to_string());
        self.notes.push("User-function calls are checked against the complete function-signature table, so forward calls and recursion are supported; print(...) is a reserved variadic built-in.".to_string());
        Ok(SemanticOutput {
            symbols: self.symbols,
            notes: self.notes,
        })
    }

    fn analyze_function(&mut self, function: &Function) -> CompileResult<()> {
        let scope_id = self
            .symbols
            .enter_scope(format!("function {}", function.name));
        let mut initialized: HashSet<(ScopeId, String)> = HashSet::new();
        for param in &function.params {
            self.symbols
                .define_in(
                    scope_id,
                    SymbolDefinition {
                        name: param.clone(),
                        ty: "i64".to_string(),
                        kind: SymbolKind::Parameter,
                        owner_function: Some(function.name.clone()),
                        params: Vec::new(),
                        declaration_line: function.line,
                    },
                )
                .map_err(|message| CompileError::new(3, function.line, 1, message))?;
            initialized.insert((scope_id, param.clone()));
        }
        self.notes.push(format!("Function '{}': inherited environment = parameters and visible function-local bindings; synthesized return type = i64.", function.name));
        self.check_statements(&function.body, scope_id, &mut initialized, &function.name)?;
        self.symbols.exit_scope();
        Ok(())
    }

    fn check_statements(
        &mut self,
        statements: &[Stmt],
        function_scope: ScopeId,
        initialized: &mut HashSet<(ScopeId, String)>,
        function_name: &str,
    ) -> CompileResult<()> {
        for stmt in statements {
            match stmt {
                Stmt::Assign { name, value, line } => {
                    self.check_expr(value, function_scope, initialized)?;
                    if self.symbols.lookup_in_scope(function_scope, name).is_none() {
                        if self
                            .symbols
                            .lookup_global(name)
                            .map(|s| s.kind == SymbolKind::Function)
                            .unwrap_or(false)
                        {
                            return Err(CompileError::new(
                                3,
                                *line,
                                1,
                                format!("'{}' is a function name and cannot be assigned", name),
                            ));
                        }
                        self.symbols
                            .define_in(
                                function_scope,
                                SymbolDefinition {
                                    name: name.clone(),
                                    ty: "i64".to_string(),
                                    kind: SymbolKind::Variable,
                                    owner_function: Some(function_name.to_string()),
                                    params: Vec::new(),
                                    declaration_line: *line,
                                },
                            )
                            .map_err(|message| CompileError::new(3, *line, 1, message))?;
                    }
                    initialized.insert((function_scope, name.clone()));
                }
                Stmt::Return { value, .. } => {
                    self.check_expr(value, function_scope, initialized)?
                }
                Stmt::Expr { value, .. } => self.check_expr(value, function_scope, initialized)?,
                Stmt::If {
                    condition,
                    then_body,
                    else_body,
                    line,
                } => {
                    self.check_expr(condition, function_scope, initialized)?;
                    let before = initialized.clone();
                    self.symbols.enter_scope(format!("if@{}", line));
                    let mut then_state = before.clone();
                    self.check_statements(
                        then_body,
                        function_scope,
                        &mut then_state,
                        function_name,
                    )?;
                    self.symbols.exit_scope();
                    if else_body.is_empty() {
                        *initialized = before;
                    } else {
                        self.symbols.enter_scope(format!("else@{}", line));
                        let mut else_state = before;
                        self.check_statements(
                            else_body,
                            function_scope,
                            &mut else_state,
                            function_name,
                        )?;
                        self.symbols.exit_scope();
                        *initialized = then_state.intersection(&else_state).cloned().collect();
                    }
                    self.notes.push(format!("If at line {}: condition is evaluated as i64 truthiness (nonzero = true); definite initialization joins both paths by intersection.", line));
                }
                Stmt::For {
                    variable,
                    start,
                    end,
                    body,
                    line,
                } => {
                    self.check_expr(start, function_scope, initialized)?;
                    self.check_expr(end, function_scope, initialized)?;
                    if self
                        .symbols
                        .lookup_in_scope(function_scope, variable)
                        .is_none()
                    {
                        if self
                            .symbols
                            .lookup_global(variable)
                            .map(|s| s.kind == SymbolKind::Function)
                            .unwrap_or(false)
                        {
                            return Err(CompileError::new(
                                3,
                                *line,
                                1,
                                format!(
                                    "'{}' is a function name and cannot be a loop variable",
                                    variable
                                ),
                            ));
                        }
                        self.symbols
                            .define_in(
                                function_scope,
                                SymbolDefinition {
                                    name: variable.clone(),
                                    ty: "i64".to_string(),
                                    kind: SymbolKind::Variable,
                                    owner_function: Some(function_name.to_string()),
                                    params: Vec::new(),
                                    declaration_line: *line,
                                },
                            )
                            .map_err(|message| CompileError::new(3, *line, 1, message))?;
                    }
                    let before = initialized.clone();
                    let mut loop_state = before.clone();
                    loop_state.insert((function_scope, variable.clone()));
                    self.symbols
                        .enter_scope(format!("for {}@{}", variable, line));
                    self.check_statements(body, function_scope, &mut loop_state, function_name)?;
                    self.symbols.exit_scope();
                    *initialized = before;
                    initialized.insert((function_scope, variable.clone()));
                    self.notes.push(format!("Range loop at line {}: start and exclusive end are evaluated once; loop variable '{}' is initialized before the first test.", line, variable));
                }
            }
        }
        Ok(())
    }

    fn check_expr(
        &self,
        expr: &Expr,
        function_scope: ScopeId,
        initialized: &HashSet<(ScopeId, String)>,
    ) -> CompileResult<()> {
        match &expr.kind {
            ExprKind::Integer(_) => Ok(()),
            ExprKind::Variable(name) => match self.symbols.lookup(name) {
                Some(symbol) if symbol.kind == SymbolKind::Function => Err(CompileError::new(
                    3,
                    expr.line,
                    1,
                    format!("function '{}' cannot be used as an integer value", name),
                )),
                Some(_) if initialized.contains(&(function_scope, name.clone())) => Ok(()),
                Some(_) => Err(CompileError::new(
                    3,
                    expr.line,
                    1,
                    format!("variable '{}' may be read before it is initialized", name),
                )),
                None => Err(CompileError::new(
                    3,
                    expr.line,
                    1,
                    format!("undefined variable '{}'", name),
                )),
            },
            ExprKind::Call { name, args } => {
                if name == "print" {
                    for arg in args {
                        self.check_expr(arg, function_scope, initialized)?;
                    }
                    return Ok(());
                }
                let expected = self.signatures.get(name).ok_or_else(|| {
                    CompileError::new(
                        3,
                        expr.line,
                        1,
                        format!("call to undefined function '{}'", name),
                    )
                })?;
                if expected.len() != args.len() {
                    return Err(CompileError::new(
                        3,
                        expr.line,
                        1,
                        format!(
                            "function '{}' expects {} argument(s), received {}",
                            name,
                            expected.len(),
                            args.len()
                        ),
                    ));
                }
                for arg in args {
                    self.check_expr(arg, function_scope, initialized)?;
                }
                Ok(())
            }
            ExprKind::Unary { operand, .. } => {
                self.check_expr(operand, function_scope, initialized)
            }
            ExprKind::Binary { left, right, .. } => {
                self.check_expr(left, function_scope, initialized)?;
                self.check_expr(right, function_scope, initialized)
            }
        }
    }
}

pub fn check(program: &Program) -> CompileResult<SemanticOutput> {
    Analyzer::new().analyze(program)
}
