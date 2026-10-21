use crate::phase4_intermediate::three_address_code::Quadruple;
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub id: usize,
    pub function: String,
    pub start_index: usize,
    pub end_index: usize,
    pub quads: Vec<Quadruple>,
    pub successors: Vec<usize>,
    pub predecessors: Vec<usize>,
}

fn target_of(q: &Quadruple) -> Option<&str> {
    match q.op.as_str() {
        "GOTO" | "BR_ZERO" | "BR_LT" | "BR_LE" | "BR_GT" | "BR_GE" | "BR_EQ" | "BR_NE" => {
            Some(q.result.as_str())
        }
        _ => None,
    }
}

pub fn build_basic_blocks(code: &[Quadruple]) -> Vec<BasicBlock> {
    if code.is_empty() {
        return Vec::new();
    }
    let mut leaders = BTreeSet::new();
    leaders.insert(0usize);
    for (index, q) in code.iter().enumerate() {
        if q.op == "LABEL" || q.op == "FUNC" {
            leaders.insert(index);
        }
        if q.is_terminator() && index + 1 < code.len() {
            leaders.insert(index + 1);
        }
    }
    let points: Vec<usize> = leaders.into_iter().collect();
    let mut blocks: Vec<BasicBlock> = Vec::new();
    for (i, start) in points.iter().enumerate() {
        let end = points.get(i + 1).copied().unwrap_or(code.len());
        let segment = &code[*start..end];
        let function = segment
            .iter()
            .rev()
            .find_map(|q| {
                if q.op == "FUNC" || q.op == "FUNC_END" {
                    Some(q.result.clone())
                } else {
                    None
                }
            })
            .or_else(|| blocks.last().map(|b: &BasicBlock| b.function.clone()))
            .unwrap_or_default();
        blocks.push(BasicBlock {
            id: blocks.len(),
            function,
            start_index: *start,
            end_index: end,
            quads: segment.to_vec(),
            successors: Vec::new(),
            predecessors: Vec::new(),
        });
    }

    let mut label_to_block: HashMap<String, usize> = HashMap::new();
    for block in &blocks {
        for q in &block.quads {
            if q.op == "LABEL" {
                label_to_block.insert(q.result.clone(), block.id);
            }
        }
    }
    let count = blocks.len();
    for idx in 0..count {
        let last = blocks[idx].quads.last().cloned();
        let function = blocks[idx].function.clone();
        let next_same_function = if idx + 1 < count && blocks[idx + 1].function == function {
            Some(idx + 1)
        } else {
            None
        };
        let mut successors = Vec::new();
        if let Some(q) = last {
            match q.op.as_str() {
                "GOTO" => {
                    if let Some(target) = label_to_block.get(&q.result).copied() {
                        if blocks[target].function == function {
                            successors.push(target);
                        }
                    }
                }
                "BR_ZERO" | "BR_LT" | "BR_LE" | "BR_GT" | "BR_GE" | "BR_EQ" | "BR_NE" => {
                    if let Some(target) = target_of(&q)
                        .and_then(|name| label_to_block.get(name))
                        .copied()
                    {
                        if blocks[target].function == function {
                            successors.push(target);
                        }
                    }
                    if let Some(next) = next_same_function {
                        successors.push(next);
                    }
                }
                "RET" | "FUNC_END" => {}
                _ => {
                    if let Some(next) = next_same_function {
                        successors.push(next);
                    }
                }
            }
        }
        successors.sort_unstable();
        successors.dedup();
        blocks[idx].successors = successors;
    }
    let edges: Vec<(usize, usize)> = blocks
        .iter()
        .flat_map(|b| b.successors.iter().map(move |s| (b.id, *s)))
        .collect();
    for (from, to) in edges {
        if !blocks[to].predecessors.contains(&from) {
            blocks[to].predecessors.push(from);
        }
    }
    blocks
}
