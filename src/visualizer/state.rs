use crate::common::error_handler::CompileError;
use crate::common::symbol_table::SymbolTable;
use crate::phase1_lexical::token::Token;
use crate::phase2_syntax::grammar::Program;
use crate::phase4_intermediate::three_address_code::Quadruple;
use crate::phase5_optimization::basic_blocks::BasicBlock;
use crate::phase6_target_codegen::runtime_activation::ActivationRecord;

#[derive(Clone, Debug)]
pub struct CompilerSnapshot {
    pub input_path: String,
    pub raw_source: String,
    pub preprocessed_source: String,
    pub tokens: Vec<Token>,
    pub ast: Option<Program>,
    pub symbols: SymbolTable,
    pub sdd_notes: Vec<String>,
    pub tac_unoptimized: Vec<Quadruple>,
    pub tac_optimized: Vec<Quadruple>,
    pub basic_blocks: Vec<BasicBlock>,
    pub activation_records: Vec<ActivationRecord>,
    pub assembly: String,
    pub completed_phase: Option<u8>,
    pub diagnostics: Vec<CompileError>,
}

impl CompilerSnapshot {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            input_path: path.into(),
            raw_source: String::new(),
            preprocessed_source: String::new(),
            tokens: Vec::new(),
            ast: None,
            symbols: SymbolTable::new(),
            sdd_notes: Vec::new(),
            tac_unoptimized: Vec::new(),
            tac_optimized: Vec::new(),
            basic_blocks: Vec::new(),
            activation_records: Vec::new(),
            assembly: String::new(),
            completed_phase: None,
            diagnostics: Vec::new(),
        }
    }

    pub fn is_phase_available(&self, phase: u8) -> bool {
        self.completed_phase
            .map(|completed| phase <= completed)
            .unwrap_or(false)
    }
}
