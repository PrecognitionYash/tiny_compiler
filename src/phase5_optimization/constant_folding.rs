use crate::phase4_intermediate::three_address_code::Quadruple;
use std::collections::HashMap;

fn literal(value: &str) -> Option<i64> {
    value.parse::<i64>().ok()
}

fn evaluate(op: &str, a: i64, b: i64) -> Option<i64> {
    match op {
        "ADD" => a.checked_add(b),
        "SUB" => a.checked_sub(b),
        "MUL" => a.checked_mul(b),
        "DIV" if b != 0 => a.checked_div(b),
        "AND" => Some(a & b),
        "OR" => Some(a | b),
        "XOR" => Some(a ^ b),
        "SHL" if (0..64).contains(&b) => a.checked_shl(b as u32),
        "SHR" if (0..64).contains(&b) => Some(a >> (b as u32)),
        "CMP_EQ" => Some(if a == b { 1 } else { 0 }),
        "CMP_NE" => Some(if a != b { 1 } else { 0 }),
        "CMP_LT" => Some(if a < b { 1 } else { 0 }),
        "CMP_LE" => Some(if a <= b { 1 } else { 0 }),
        "CMP_GT" => Some(if a > b { 1 } else { 0 }),
        "CMP_GE" => Some(if a >= b { 1 } else { 0 }),
        _ => None,
    }
}

// Phase 5 Optimization: Local constant propogation and folding.
// We are clearing constants table at control flow boundries, function boundries,
// and calls so wrong values dont leak across differnt execution paths.
pub fn optimize(code: &[Quadruple]) -> Vec<Quadruple> {
    let mut output = Vec::with_capacity(code.len());
    let mut constants: HashMap<String, i64> = HashMap::new();

    for original in code {
        let mut q = original.clone();
        if matches!(
            q.op.as_str(),
            "LABEL"
                | "FUNC"
                | "FUNC_END"
                | "CALL"
                | "PARAM"
                | "GOTO"
                | "BR_ZERO"
                | "BR_LT"
                | "BR_LE"
                | "BR_GT"
                | "BR_GE"
                | "BR_EQ"
                | "BR_NE"
                | "RET"
        ) {
            constants.clear();
        }
        if let Some(value) = constants.get(&q.arg1).copied() {
            q.arg1 = value.to_string();
        }
        if let Some(value) = constants.get(&q.arg2).copied() {
            q.arg2 = value.to_string();
        }

        if q.op == "PRINT" {
            if !q.arg2.is_empty() {
                q.arg2 = q
                    .arg2
                    .split(',')
                    .map(|value| {
                        constants
                            .get(value)
                            .copied()
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| value.to_string())
                    })
                    .collect::<Vec<_>>()
                    .join(",");
            }
            constants.clear();
            output.push(q);
            continue;
        }

        if q.op == "MOV" {
            constants.remove(&q.result);
            if let Some(value) = literal(&q.arg1) {
                constants.insert(q.result.clone(), value);
            }
            output.push(q);
            continue;
        }
        if q.op == "NEG" {
            if let Some(value) = literal(&q.arg1).and_then(|value| value.checked_neg()) {
                q.op = "MOV".to_string();
                q.arg1 = value.to_string();
                q.arg2.clear();
                constants.insert(q.result.clone(), value);
                output.push(q);
                continue;
            }
        }
        if let (Some(a), Some(b)) = (literal(&q.arg1), literal(&q.arg2)) {
            if let Some(value) = evaluate(&q.op, a, b) {
                q.op = "MOV".to_string();
                q.arg1 = value.to_string();
                q.arg2.clear();
                constants.insert(q.result.clone(), value);
                output.push(q);
                continue;
            }
        }
        if !q.result.is_empty()
            && !matches!(
                q.op.as_str(),
                "LABEL"
                    | "FUNC"
                    | "FUNC_END"
                    | "GOTO"
                    | "BR_ZERO"
                    | "BR_LT"
                    | "BR_LE"
                    | "BR_GT"
                    | "BR_GE"
                    | "BR_EQ"
                    | "BR_NE"
                    | "RET"
            )
        {
            constants.remove(&q.result);
        }
        output.push(q);
    }
    output
}
