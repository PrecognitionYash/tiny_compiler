use crate::common::symbol_table::{SymbolKind, SymbolTable};
use crate::phase4_intermediate::three_address_code::Quadruple;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ActivationRecord {
    pub function: String,
    pub frame_size: i32,
    pub slots: HashMap<String, i32>,
    pub slot_count: usize,
}

impl ActivationRecord {
    pub fn offset_of(&self, name: &str) -> Option<i32> {
        self.slots.get(name).copied()
    }
}

fn is_value_result(op: &str) -> bool {
    matches!(
        op,
        "MOV"
            | "ADD"
            | "SUB"
            | "MUL"
            | "DIV"
            | "AND"
            | "OR"
            | "XOR"
            | "SHL"
            | "SHR"
            | "NEG"
            | "CMP_EQ"
            | "CMP_NE"
            | "CMP_LT"
            | "CMP_LE"
            | "CMP_GT"
            | "CMP_GE"
            | "CALL"
    )
}

fn collect_temp_operand(value: &str, slots: &mut HashMap<String, i32>, next_offset: &mut i32) {
    if value.starts_with('$') && !slots.contains_key(value) {
        *next_offset += 8;
        slots.insert(value.to_string(), *next_offset);
    }
}

pub fn allocate(code: &[Quadruple], symbols: &mut SymbolTable) -> Vec<ActivationRecord> {
    let mut records = Vec::new();
    let mut index = 0usize;
    while index < code.len() {
        if code[index].op != "FUNC" {
            index += 1;
            continue;
        }
        let function = code[index].result.clone();
        let start = index;
        index += 1;
        while index < code.len() && code[index].op != "FUNC_END" {
            index += 1;
        }
        let end = (if index < code.len() { index + 1 } else { index }).min(code.len());
        let function_code = &code[start..end];

        symbols.clear_stack_offsets_for(&function);
        let mut slots: HashMap<String, i32> = HashMap::new();
        let mut next_offset = 0i32;
        let local_symbols: Vec<(String, SymbolKind)> = symbols
            .symbols
            .iter()
            .filter(|symbol| {
                symbol.owner_function.as_deref() == Some(function.as_str())
                    && symbol.kind != SymbolKind::Function
            })
            .map(|symbol| (symbol.name.clone(), symbol.kind.clone()))
            .collect();
        for (name, _kind) in local_symbols {
            if !slots.contains_key(&name) {
                next_offset += 8;
                slots.insert(name.clone(), next_offset);
                symbols.assign_stack_offset(&function, &name, next_offset);
            }
        }

        for q in function_code {
            if q.op == "PARAM" {
                collect_temp_operand(&q.result, &mut slots, &mut next_offset);
            }
            if matches!(
                q.op.as_str(),
                "MOV"
                    | "ADD"
                    | "SUB"
                    | "MUL"
                    | "DIV"
                    | "AND"
                    | "OR"
                    | "XOR"
                    | "SHL"
                    | "SHR"
                    | "NEG"
                    | "CMP_EQ"
                    | "CMP_NE"
                    | "CMP_LT"
                    | "CMP_LE"
                    | "CMP_GT"
                    | "CMP_GE"
                    | "CALL"
                    | "PRINT"
            ) {
                if q.op == "CALL" || q.op == "PRINT" {
                    if !q.arg2.is_empty() {
                        for arg in q.arg2.split(',') {
                            collect_temp_operand(arg, &mut slots, &mut next_offset);
                        }
                    }
                } else {
                    collect_temp_operand(&q.arg1, &mut slots, &mut next_offset);
                    collect_temp_operand(&q.arg2, &mut slots, &mut next_offset);
                }
                if is_value_result(&q.op) && !q.result.is_empty() {
                    collect_temp_operand(&q.result, &mut slots, &mut next_offset);
                }
            } else if q.is_branch() {
                collect_temp_operand(&q.arg1, &mut slots, &mut next_offset);
                collect_temp_operand(&q.arg2, &mut slots, &mut next_offset);
            } else if q.op == "RET" {
                collect_temp_operand(&q.arg1, &mut slots, &mut next_offset);
            }
        }
        let frame_size = if next_offset == 0 {
            0
        } else {
            ((next_offset + 15) / 16) * 16
        };
        records.push(ActivationRecord {
            function,
            frame_size,
            slot_count: slots.len(),
            slots,
        });
        index = end;
    }
    records
}
