use std::fmt;

pub type ScopeId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Parameter,
    Variable,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Parameter => write!(f, "parameter"),
            Self::Variable => write!(f, "variable"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SymbolDefinition {
    pub name: String,
    pub ty: String,
    pub kind: SymbolKind,
    pub owner_function: Option<String>,
    pub params: Vec<String>,
    pub declaration_line: usize,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    pub ty: String,
    pub scope_id: ScopeId,
    pub stack_offset: Option<i32>,
    pub kind: SymbolKind,
    pub owner_function: Option<String>,
    pub params: Vec<String>,
    pub declaration_line: usize,
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub scopes: Vec<Scope>,
    scope_stack: Vec<ScopeId>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            scopes: vec![Scope {
                id: 0,
                parent: None,
                label: "global".to_string(),
            }],
            scope_stack: vec![0],
        }
    }

    pub fn current_scope(&self) -> ScopeId {
        *self.scope_stack.last().unwrap_or(&0)
    }

    pub fn enter_scope(&mut self, label: impl Into<String>) -> ScopeId {
        let id = self.scopes.len();
        let parent = Some(self.current_scope());
        self.scopes.push(Scope {
            id,
            parent,
            label: label.into(),
        });
        self.scope_stack.push(id);
        id
    }

    pub fn exit_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    pub fn define(
        &mut self,
        name: impl Into<String>,
        ty: impl Into<String>,
        kind: SymbolKind,
        owner_function: Option<String>,
        params: Vec<String>,
        declaration_line: usize,
    ) -> Result<usize, String> {
        self.define_in(
            self.current_scope(),
            SymbolDefinition {
                name: name.into(),
                ty: ty.into(),
                kind,
                owner_function,
                params,
                declaration_line,
            },
        )
    }

    pub fn define_in(
        &mut self,
        scope_id: ScopeId,
        definition: SymbolDefinition,
    ) -> Result<usize, String> {
        if let Some(existing) = self
            .symbols
            .iter()
            .find(|s| s.scope_id == scope_id && s.name == definition.name)
        {
            return Err(format!(
                "'{}' is already declared in scope '{}'",
                definition.name,
                self.scope_label(existing.scope_id)
            ));
        }
        let index = self.symbols.len();
        self.symbols.push(Symbol {
            name: definition.name,
            ty: definition.ty,
            scope_id,
            stack_offset: None,
            kind: definition.kind,
            owner_function: definition.owner_function,
            params: definition.params,
            declaration_line: definition.declaration_line,
        });
        Ok(index)
    }

    pub fn lookup_in_scope(&self, scope_id: ScopeId, name: &str) -> Option<&Symbol> {
        self.symbols
            .iter()
            .find(|s| s.scope_id == scope_id && s.name == name)
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut cursor = Some(self.current_scope());
        while let Some(scope_id) = cursor {
            if let Some(symbol) = self.lookup_in_scope(scope_id, name) {
                return Some(symbol);
            }
            cursor = self.scopes.get(scope_id).and_then(|scope| scope.parent);
        }
        None
    }

    pub fn lookup_global(&self, name: &str) -> Option<&Symbol> {
        self.lookup_in_scope(0, name)
    }

    pub fn scope_label(&self, scope_id: ScopeId) -> &str {
        self.scopes
            .get(scope_id)
            .map(|s| s.label.as_str())
            .unwrap_or("unknown")
    }

    pub fn assign_stack_offset(&mut self, owner: &str, name: &str, offset: i32) {
        if let Some(symbol) = self.symbols.iter_mut().find(|s| {
            s.owner_function.as_deref() == Some(owner)
                && s.name == name
                && s.kind != SymbolKind::Function
        }) {
            symbol.stack_offset = Some(offset);
        }
    }

    pub fn clear_stack_offsets_for(&mut self, owner: &str) {
        for symbol in &mut self.symbols {
            if symbol.owner_function.as_deref() == Some(owner) {
                symbol.stack_offset = None;
            }
        }
    }
}
