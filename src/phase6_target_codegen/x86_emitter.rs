use super::runtime_activation::{allocate, ActivationRecord};
use crate::common::error_handler::{CompileError, CompileResult};
use crate::common::symbol_table::SymbolTable;
use crate::phase4_intermediate::three_address_code::Quadruple;

fn is_macos() -> bool {
    cfg!(target_os = "macos")
}
fn is_windows() -> bool {
    cfg!(target_os = "windows")
}
fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

fn runtime_symbol(name: &str) -> String {
    if is_macos() {
        format!("_{}", name)
    } else {
        name.to_string()
    }
}

fn function_symbol(name: &str) -> String {
    let internal_name = if name == "main" {
        name.to_string()
    } else {
        format!("tcomp_fn_{}", name)
    };
    if is_macos() {
        format!("_{}", internal_name)
    } else {
        internal_name
    }
}

fn local_label(name: &str) -> String {
    if is_macos() {
        name.strip_prefix('.').unwrap_or(name).to_string()
    } else {
        name.to_string()
    }
}

fn format_label(index: usize) -> String {
    let base = format!(".L_tcomp_fmt_{}", index);
    local_label(&base)
}

fn slot(record: &ActivationRecord, name: &str, line: usize) -> CompileResult<i32> {
    record.offset_of(name).ok_or_else(|| {
        CompileError::new(
            6,
            line,
            1,
            format!(
                "internal code-generation error: no stack slot allocated for '{}' in function '{}'",
                name, record.function
            ),
        )
    })
}

fn load_value(
    out: &mut String,
    value: &str,
    register: &str,
    record: &ActivationRecord,
    line: usize,
) -> CompileResult<()> {
    if let Ok(number) = value.parse::<i64>() {
        out.push_str(&format!("    mov {}, {}\n", register, number));
    } else {
        let offset = slot(record, value, line)?;
        out.push_str(&format!(
            "    mov {}, QWORD PTR [rbp - {}]\n",
            register, offset
        ));
    }
    Ok(())
}

fn store_value(
    out: &mut String,
    value: &str,
    register: &str,
    record: &ActivationRecord,
    line: usize,
) -> CompileResult<()> {
    let offset = slot(record, value, line)?;
    out.push_str(&format!(
        "    mov QWORD PTR [rbp - {}], {}\n",
        offset, register
    ));
    Ok(())
}

fn emit_prologue(out: &mut String, function: &str, record: &ActivationRecord) {
    let symbol = function_symbol(function);
    out.push_str(&format!("\n.globl {}\n", symbol));
    if is_linux() {
        out.push_str(&format!(".type {}, @function\n", symbol));
    }
    out.push_str(&format!("{}:\n    push rbp\n    mov rbp, rsp\n", symbol));
    if record.frame_size > 0 {
        out.push_str(&format!("    sub rsp, {}\n", record.frame_size));
    }
}

fn emit_epilogue(out: &mut String) {
    out.push_str("    mov rsp, rbp\n    pop rbp\n    ret\n");
}

fn emit_binary(out: &mut String, q: &Quadruple, record: &ActivationRecord) -> CompileResult<()> {
    load_value(out, &q.arg1, "rax", record, q.line)?;
    load_value(out, &q.arg2, "rcx", record, q.line)?;
    match q.op.as_str() {
        "ADD" => out.push_str("    add rax, rcx\n"),
        "SUB" => out.push_str("    sub rax, rcx\n"),
        "MUL" => out.push_str("    imul rax, rcx\n"),
        "DIV" => out.push_str("    cqo\n    idiv rcx\n"),
        "AND" => out.push_str("    and rax, rcx\n"),
        "OR" => out.push_str("    or rax, rcx\n"),
        "XOR" => out.push_str("    xor rax, rcx\n"),
        "SHL" => out.push_str("    and rcx, 63\n    shl rax, cl\n"),
        "SHR" => out.push_str("    and rcx, 63\n    sar rax, cl\n"),
        "CMP_EQ" | "CMP_NE" | "CMP_LT" | "CMP_LE" | "CMP_GT" | "CMP_GE" => {
            out.push_str("    cmp rax, rcx\n");
            let set = match q.op.as_str() {
                "CMP_EQ" => "sete",
                "CMP_NE" => "setne",
                "CMP_LT" => "setl",
                "CMP_LE" => "setle",
                "CMP_GT" => "setg",
                _ => "setge",
            };
            out.push_str(&format!("    {} al\n    movzx rax, al\n", set));
        }
        _ => {
            return Err(CompileError::new(
                6,
                q.line,
                1,
                format!("unsupported binary TAC operation '{}'", q.op),
            ))
        }
    }
    store_value(out, &q.result, "rax", record, q.line)
}

fn align16(bytes: usize) -> usize {
    (bytes + 15) & !15
}

fn emit_user_call(out: &mut String, q: &Quadruple, record: &ActivationRecord) -> CompileResult<()> {
    let args: Vec<&str> = if q.arg2.is_empty() {
        Vec::new()
    } else {
        q.arg2.split(',').collect()
    };
    if is_windows() {
        // Microsoft x64 calling convension: 4 register arguments (RCX, RDX, R8, R9)
        // and 32 bytes shadow space for caller. Handled for windows compatibilty!
        let regs = ["rcx", "rdx", "r8", "r9"];
        let stack_args = args.get(regs.len()..).unwrap_or(&[]);
        let alloc = align16(32 + stack_args.len() * 8);
        out.push_str(&format!("    sub rsp, {}\n", alloc));
        for (index, value) in stack_args.iter().enumerate() {
            load_value(out, value, "rax", record, q.line)?;
            out.push_str(&format!(
                "    mov QWORD PTR [rsp + {}], rax\n",
                32 + index * 8
            ));
        }
        for (value, reg) in args.iter().take(regs.len()).zip(regs.iter()) {
            load_value(out, value, reg, record, q.line)?;
        }
        out.push_str(&format!("    call {}\n", function_symbol(&q.arg1)));
        out.push_str(&format!("    add rsp, {}\n", alloc));
    } else {
        // System V AMD64 ABI (Linux and Mac): 6 interger registers (RDI, RSI, RDX, RCX, R8, R9);
        // stack must remain 16-byte aligned before call instruction or segfault will occur.
        let regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        let stack_args = args.get(regs.len()..).unwrap_or(&[]);
        let padding = if stack_args.len() % 2 == 1 {
            8usize
        } else {
            0usize
        };
        if padding > 0 {
            out.push_str(&format!("    sub rsp, {}\n", padding));
        }
        for value in stack_args.iter().rev() {
            load_value(out, value, "rax", record, q.line)?;
            out.push_str("    push rax\n");
        }
        for (value, reg) in args.iter().take(regs.len()).zip(regs.iter()) {
            load_value(out, value, reg, record, q.line)?;
        }
        out.push_str(&format!("    call {}\n", function_symbol(&q.arg1)));
        let cleanup = stack_args.len() * 8 + padding;
        if cleanup > 0 {
            out.push_str(&format!("    add rsp, {}\n", cleanup));
        }
    }
    if !q.result.is_empty() {
        store_value(out, &q.result, "rax", record, q.line)?;
    }
    Ok(())
}

fn emit_print(
    out: &mut String,
    q: &Quadruple,
    record: &ActivationRecord,
    label: &str,
) -> CompileResult<String> {
    let args: Vec<&str> = if q.arg2.is_empty() {
        Vec::new()
    } else {
        q.arg2.split(',').collect()
    };
    let format = if args.is_empty() {
        "\\n".to_string()
    } else {
        format!("{}\\n", vec!["%lld"; args.len()].join(" "))
    };

    if is_windows() {
        // Windows x64: printf(format, first, second, third, ...) uses RCX/RDX/R8/R9 regesters.
        let value_regs = ["rdx", "r8", "r9"];
        let stack_args = args.get(value_regs.len()..).unwrap_or(&[]);
        let alloc = align16(32 + stack_args.len() * 8);
        out.push_str(&format!("    sub rsp, {}\n", alloc));
        out.push_str(&format!("    lea rcx, [rip + {}]\n", label));
        for (index, value) in stack_args.iter().enumerate() {
            load_value(out, value, "rax", record, q.line)?;
            out.push_str(&format!(
                "    mov QWORD PTR [rsp + {}], rax\n",
                32 + index * 8
            ));
        }
        for (value, reg) in args.iter().take(value_regs.len()).zip(value_regs.iter()) {
            load_value(out, value, reg, record, q.line)?;
        }
        out.push_str(&format!(
            "    xor eax, eax\n    call {}\n",
            runtime_symbol("printf")
        ));
        out.push_str(&format!("    add rsp, {}\n", alloc));
    } else {
        // System V AMD64: printf format string goes to RDI, and values go into RSI, RDX, RCX, R8, R9.
        let value_regs = ["rsi", "rdx", "rcx", "r8", "r9"];
        let stack_args = args.get(value_regs.len()..).unwrap_or(&[]);
        let alloc = align16(stack_args.len() * 8);
        if alloc > 0 {
            out.push_str(&format!("    sub rsp, {}\n", alloc));
        }
        out.push_str(&format!("    lea rdi, [rip + {}]\n", label));
        for (index, value) in stack_args.iter().enumerate() {
            load_value(out, value, "rax", record, q.line)?;
            out.push_str(&format!("    mov QWORD PTR [rsp + {}], rax\n", index * 8));
        }
        for (value, reg) in args.iter().take(value_regs.len()).zip(value_regs.iter()) {
            load_value(out, value, reg, record, q.line)?;
        }
        out.push_str(&format!(
            "    xor eax, eax\n    call {}\n",
            runtime_symbol("printf")
        ));
        if alloc > 0 {
            out.push_str(&format!("    add rsp, {}\n", alloc));
        }
    }
    Ok(format)
}

fn emit_quad(
    out: &mut String,
    q: &Quadruple,
    record: &ActivationRecord,
    index: usize,
    formats: &mut Vec<(String, String)>,
) -> CompileResult<()> {
    match q.op.as_str() {
        "FUNC" => emit_prologue(out, &q.result, record),
        "FUNC_END" => {}
        "LABEL" => out.push_str(&format!("{}:\n", local_label(&q.result))),
        "PARAM" => {
            let index = q
                .arg1
                .parse::<usize>()
                .map_err(|_| CompileError::new(6, q.line, 1, "invalid PARAM index in TAC"))?;
            let regs: &[&str] = if is_windows() {
                &["rcx", "rdx", "r8", "r9"]
            } else {
                &["rdi", "rsi", "rdx", "rcx", "r8", "r9"]
            };
            if let Some(reg) = regs.get(index) {
                store_value(out, &q.result, reg, record, q.line)?;
            } else {
                let (first_stack_index, base_offset) = if is_windows() {
                    (4usize, 48usize)
                } else {
                    (6usize, 16usize)
                };
                let stack_offset = index
                    .checked_sub(first_stack_index)
                    .and_then(|n| n.checked_mul(8))
                    .and_then(|n| n.checked_add(base_offset))
                    .and_then(|n| i32::try_from(n).ok())
                    .ok_or_else(|| {
                        CompileError::new(
                            6,
                            q.line,
                            1,
                            "function parameter stack offset exceeds backend limits",
                        )
                    })?;
                out.push_str(&format!(
                    "    mov rax, QWORD PTR [rbp + {}]\n",
                    stack_offset
                ));
                store_value(out, &q.result, "rax", record, q.line)?;
            }
        }
        "MOV" => {
            load_value(out, &q.arg1, "rax", record, q.line)?;
            store_value(out, &q.result, "rax", record, q.line)?;
        }
        "NEG" => {
            load_value(out, &q.arg1, "rax", record, q.line)?;
            out.push_str("    neg rax\n");
            store_value(out, &q.result, "rax", record, q.line)?;
        }
        "ADD" | "SUB" | "MUL" | "DIV" | "AND" | "OR" | "XOR" | "SHL" | "SHR" | "CMP_EQ"
        | "CMP_NE" | "CMP_LT" | "CMP_LE" | "CMP_GT" | "CMP_GE" => emit_binary(out, q, record)?,
        "BR_ZERO" => {
            load_value(out, &q.arg1, "rax", record, q.line)?;
            out.push_str(&format!(
                "    cmp rax, 0\n    je {}\n",
                local_label(&q.result)
            ));
        }
        "BR_LT" | "BR_LE" | "BR_GT" | "BR_GE" | "BR_EQ" | "BR_NE" => {
            load_value(out, &q.arg1, "rax", record, q.line)?;
            load_value(out, &q.arg2, "rcx", record, q.line)?;
            let jump = match q.op.as_str() {
                "BR_LT" => "jl",
                "BR_LE" => "jle",
                "BR_GT" => "jg",
                "BR_GE" => "jge",
                "BR_EQ" => "je",
                _ => "jne",
            };
            out.push_str(&format!(
                "    cmp rax, rcx\n    {} {}\n",
                jump,
                local_label(&q.result)
            ));
        }
        "GOTO" => out.push_str(&format!("    jmp {}\n", local_label(&q.result))),
        "RET" => {
            if !q.arg1.is_empty() {
                load_value(out, &q.arg1, "rax", record, q.line)?;
            } else {
                out.push_str("    xor eax, eax\n");
            }
            emit_epilogue(out);
        }
        "CALL" => emit_user_call(out, q, record)?,
        "PRINT" => {
            let label = format_label(index);
            let format = emit_print(out, q, record, &label)?;
            formats.push((label, format));
        }
        other => {
            return Err(CompileError::new(
                6,
                q.line,
                1,
                format!("unsupported TAC operation '{}'", other),
            ))
        }
    }
    Ok(())
}

pub fn emit(
    code: &[Quadruple],
    symbols: &mut SymbolTable,
) -> CompileResult<(String, Vec<ActivationRecord>)> {
    if !is_linux() && !is_macos() && !is_windows() {
        return Err(CompileError::new(
            6,
            0,
            0,
            "this backend targets x86-64 Linux, macOS, and Windows GNU-compatible toolchains",
        ));
    }
    let records = allocate(code, symbols);
    let mut assembly = String::new();
    assembly.push_str(".intel_syntax noprefix\n");
    if is_macos() {
        assembly.push_str(".section __TEXT,__text,regular,pure_instructions\n");
    } else {
        assembly.push_str(".text\n");
    }

    let mut formats = Vec::new();
    let mut active_record: Option<&ActivationRecord> = None;
    for (index, q) in code.iter().enumerate() {
        if q.op == "FUNC" {
            active_record = records.iter().find(|record| record.function == q.result);
        }
        let record = active_record.ok_or_else(|| {
            CompileError::new(6, q.line, 1, "TAC operation appeared outside a function")
        })?;
        emit_quad(&mut assembly, q, record, index, &mut formats)?;
        if q.op == "FUNC_END" {
            active_record = None;
        }
    }
    if !formats.is_empty() {
        if is_macos() {
            assembly.push_str("\n.section __TEXT,__cstring,cstring_literals\n");
        } else if is_windows() {
            assembly.push_str("\n.section .rdata,\"dr\"\n");
        } else {
            assembly.push_str("\n.section .rodata\n");
        }
        for (label, literal) in formats {
            assembly.push_str(&format!("{}:\n    .asciz \"{}\"\n", label, literal));
        }
    }
    if is_linux() {
        assembly.push_str("\n.section .note.GNU-stack,\"\",@progbits\n");
    }
    Ok((assembly, records))
}
