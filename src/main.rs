mod common;
mod phase0_preprocessor;
mod phase1_lexical;
mod phase2_syntax;
mod phase3_semantic;
mod phase4_intermediate;
mod phase5_optimization;
mod phase6_target_codegen;
mod visualizer;

use common::error_handler::CompileError;
use phase0_preprocessor::preprocessor;
use phase1_lexical::dfa_lexer;
use phase2_syntax::parser;
use phase3_semantic::{sdd_evaluator, type_checker};
use phase4_intermediate::ir_generator;
use phase5_optimization::{basic_blocks, constant_folding};
use phase6_target_codegen::x86_emitter;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use visualizer::app::CompilerApp;
use visualizer::state::CompilerSnapshot;

#[derive(Clone, Debug)]
enum Operation {
    Compile,
    Preprocess,
    Assembly,
    Visualize((u8, u8)),
}

#[derive(Debug)]
struct Cli {
    input: PathBuf,
    output: Option<PathBuf>,
    operation: Operation,
}

fn usage() -> &'static str {
    "Usage:\n  tcomp <file.t> [-o <executable>]\n  tcomp <file.t> -E [-o <file.ti>]\n  tcomp <file.t> -S [-o <file.s>]\n  tcomp <file.t> -v [start..end]\n\nPhases: 0 preprocessor, 1 lexer, 2 parser, 3 semantic analysis, 4 TAC, 5 optimization/CFG, 6 x86-64 code generation."
}

fn parse_range(text: &str) -> Result<(u8, u8), String> {
    let Some((start, end)) = text.split_once("..") else {
        return Err(format!(
            "invalid phase range '{}'; expected start..end",
            text
        ));
    };
    let start = start
        .parse::<u8>()
        .map_err(|_| format!("invalid starting phase in '{}'; expected 0..6", text))?;
    let end = end
        .parse::<u8>()
        .map_err(|_| format!("invalid ending phase in '{}'; expected 0..6", text))?;
    if start > 6 || end > 6 || start > end {
        return Err(format!(
            "invalid phase range '{}'; require 0 <= start <= end <= 6",
            text
        ));
    }
    Ok((start, end))
}

fn parse_cli() -> Result<Option<Cli>, String> {
    let mut args = env::args().skip(1).peekable();
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut operation = Operation::Compile;
    let mut operation_set = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "-o" => {
                let value = args
                    .next()
                    .ok_or_else(|| "-o requires an output path".to_string())?;
                output = Some(PathBuf::from(value));
            }
            "-E" => {
                if operation_set {
                    return Err("choose only one of -E, -S, and -v".to_string());
                }
                operation = Operation::Preprocess;
                operation_set = true;
            }
            "-S" => {
                if operation_set {
                    return Err("choose only one of -E, -S, and -v".to_string());
                }
                operation = Operation::Assembly;
                operation_set = true;
            }
            "-v" => {
                if operation_set {
                    return Err("choose only one of -E, -S, and -v".to_string());
                }
                let range = match args.peek() {
                    Some(next) if next.contains("..") && !next.starts_with('-') => {
                        parse_range(next)?
                    }
                    _ => (0, 6),
                };
                if matches!(args.peek(), Some(next) if next.contains("..") && !next.starts_with('-'))
                {
                    args.next();
                }
                operation = Operation::Visualize(range);
                operation_set = true;
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown option '{}';\n{}", arg, usage()))
            }
            _ => {
                if input.is_some() {
                    return Err(format!("unexpected extra input '{}';\n{}", arg, usage()));
                }
                input = Some(PathBuf::from(arg));
            }
        }
    }
    let input = input.ok_or_else(|| usage().to_string())?;
    if input.extension().and_then(|ext| ext.to_str()) != Some("t") {
        return Err(format!(
            "input '{}' must have the .t extension",
            input.display()
        ));
    }
    Ok(Some(Cli {
        input,
        output,
        operation,
    }))
}

// Building complier snapshot for all 7 phases sequentially.
// If error occurs in any phase, we record diagnostic and stop for GUI / terminal display.
fn build_snapshot(path: &Path) -> CompilerSnapshot {
    let mut snapshot = CompilerSnapshot::new(path.display().to_string());
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            snapshot.diagnostics.push(CompileError::io(format!(
                "cannot read '{}': {}",
                path.display(),
                error
            )));
            return snapshot;
        }
    };
    snapshot.raw_source = source.clone();
    snapshot.preprocessed_source = preprocessor::preprocess(&source);
    snapshot.completed_phase = Some(0);

    snapshot.tokens = match dfa_lexer::tokenize(&snapshot.preprocessed_source) {
        Ok(tokens) => tokens,
        Err(error) => {
            snapshot.diagnostics.push(error);
            return snapshot;
        }
    };
    snapshot.completed_phase = Some(1);

    snapshot.ast = match parser::parse(&snapshot.tokens) {
        Ok(ast) => Some(ast),
        Err(error) => {
            snapshot.diagnostics.push(error);
            return snapshot;
        }
    };
    snapshot.completed_phase = Some(2);

    let ast = snapshot
        .ast
        .as_ref()
        .expect("AST was just constructed")
        .clone();
    let semantic = match type_checker::check(&ast) {
        Ok(output) => output,
        Err(error) => {
            snapshot.diagnostics.push(error);
            return snapshot;
        }
    };
    snapshot.symbols = semantic.symbols;
    snapshot.sdd_notes = semantic.notes;
    snapshot
        .sdd_notes
        .extend(sdd_evaluator::evaluate(&ast, &snapshot.symbols));
    snapshot.completed_phase = Some(3);

    snapshot.tac_unoptimized = match ir_generator::generate(&ast) {
        Ok(code) => code,
        Err(error) => {
            snapshot.diagnostics.push(error);
            return snapshot;
        }
    };
    snapshot.completed_phase = Some(4);

    snapshot.tac_optimized = constant_folding::optimize(&snapshot.tac_unoptimized);
    snapshot.basic_blocks = basic_blocks::build_basic_blocks(&snapshot.tac_optimized);
    snapshot.completed_phase = Some(5);

    match x86_emitter::emit(&snapshot.tac_optimized, &mut snapshot.symbols) {
        Ok((assembly, records)) => {
            snapshot.assembly = assembly;
            snapshot.activation_records = records;
            snapshot.completed_phase = Some(6);
        }
        Err(error) => snapshot.diagnostics.push(error),
    }
    snapshot
}

fn print_diagnostics(snapshot: &CompilerSnapshot) {
    for diagnostic in &snapshot.diagnostics {
        eprintln!("tcomp: {}", diagnostic);
    }
}

fn require_clean(snapshot: &CompilerSnapshot) -> Result<(), String> {
    if snapshot.diagnostics.is_empty() {
        Ok(())
    } else {
        print_diagnostics(snapshot);
        Err("compilation failed".to_string())
    }
}

fn with_default_extension(input: &Path, extension: &str) -> PathBuf {
    let mut path = input.to_path_buf();
    path.set_extension(extension);
    path
}

fn native_executable_path(path: PathBuf) -> PathBuf {
    if cfg!(target_os = "windows") && path.extension().is_none() {
        let mut executable = path;
        executable.set_extension("exe");
        executable
    } else {
        path
    }
}

fn compile_to_executable(input: &Path, output: &Path, assembly: &str) -> Result<(), String> {
    let asm_path = with_default_extension(input, "tcomp-tmp.s");
    std::fs::write(&asm_path, assembly).map_err(|error| {
        format!(
            "cannot write temporary assembly '{}': {}",
            asm_path.display(),
            error
        )
    })?;

    let configured = env::var("CC").ok().filter(|value| !value.trim().is_empty());
    let candidates: Vec<String> = if let Some(cc) = configured {
        let mut values = vec![cc];
        if cfg!(target_os = "windows") {
            values.extend(["gcc".to_string(), "clang".to_string(), "cc".to_string()]);
        } else {
            values.extend(["cc".to_string(), "clang".to_string(), "gcc".to_string()]);
        }
        values
    } else if cfg!(target_os = "windows") {
        vec!["gcc".to_string(), "clang".to_string(), "cc".to_string()]
    } else {
        vec!["cc".to_string(), "clang".to_string(), "gcc".to_string()]
    };

    let mut tried = Vec::new();
    let mut last_missing = None;
    for compiler in candidates {
        if tried.contains(&compiler) {
            continue;
        }
        tried.push(compiler.clone());
        let mut command = Command::new(&compiler);
        if cfg!(target_os = "macos") {
            command.args(["-arch", "x86_64"]);
        }
        let result = command.arg(&asm_path).arg("-o").arg(output).output();
        match result {
            Ok(result) if result.status.success() => {
                let _ = std::fs::remove_file(&asm_path);
                return Ok(());
            }
            Ok(result) => {
                let _ = std::fs::remove_file(&asm_path);
                return Err(format!("native toolchain '{}' failed while assembling/linking:\n{}{}\n\nWindows note: install a 64-bit MinGW-w64 GCC toolchain and run tcomp from its terminal, or set CC to the GCC executable. MSVC's cl.exe is not compatible with this GNU-style assembly backend.",
                    compiler, String::from_utf8_lossy(&result.stdout), String::from_utf8_lossy(&result.stderr)));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                last_missing = Some(error);
            }
            Err(error) => {
                let _ = std::fs::remove_file(&asm_path);
                return Err(format!(
                    "could not invoke native toolchain '{}': {}",
                    compiler, error
                ));
            }
        }
    }
    let _ = std::fs::remove_file(&asm_path);
    Err(format!("could not find a native C toolchain (tried {}). Install a compiler and ensure it is on PATH. On Windows, install MinGW-w64 GCC; on macOS, install the Xcode Command Line Tools; on Linux, install GCC or Clang. Last lookup error: {}",
        tried.join(", "), last_missing.map(|e| e.to_string()).unwrap_or_else(|| "no compiler candidate succeeded".to_string())))
}

fn run() -> Result<(), String> {
    let Some(cli) = parse_cli()? else {
        println!("{}", usage());
        return Ok(());
    };
    match cli.operation {
        Operation::Preprocess => {
            let output = cli.output.as_deref();
            let path =
                preprocessor::preprocess_file(&cli.input, output).map_err(|e| e.to_string())?;
            println!("Preprocessed source written to {}", path.display());
        }
        Operation::Assembly => {
            let snapshot = build_snapshot(&cli.input);
            require_clean(&snapshot)?;
            let output = cli
                .output
                .unwrap_or_else(|| with_default_extension(&cli.input, "s"));
            std::fs::write(&output, &snapshot.assembly).map_err(|error| {
                format!("cannot write assembly '{}': {}", output.display(), error)
            })?;
            println!("Assembly written to {}", output.display());
        }
        Operation::Compile => {
            let snapshot = build_snapshot(&cli.input);
            require_clean(&snapshot)?;
            let output = native_executable_path(cli.output.unwrap_or_else(|| {
                if cfg!(target_os = "windows") {
                    PathBuf::from("a.out.exe")
                } else {
                    PathBuf::from("a.out")
                }
            }));
            compile_to_executable(&cli.input, &output, &snapshot.assembly)?;
            println!("Built executable {}", output.display());
        }
        Operation::Visualize(range) => {
            let snapshot = build_snapshot(&cli.input);
            let options = eframe::NativeOptions {
                viewport: eframe::egui::ViewportBuilder::default()
                    .with_inner_size([1380.0, 900.0])
                    .with_min_inner_size([980.0, 640.0]),
                ..Default::default()
            };
            eframe::run_native(
                "tcomp · Compiler Explorer",
                options,
                Box::new(move |creation_context| {
                    creation_context
                        .egui_ctx
                        .set_visuals(eframe::egui::Visuals::dark());
                    Ok(Box::new(CompilerApp::new(snapshot, range)))
                }),
            )
            .map_err(|error| format!("could not launch visualizer: {}", error))?;
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        if error != "compilation failed" {
            eprintln!("tcomp: {}", error);
        }
        process::exit(1);
    }
}
