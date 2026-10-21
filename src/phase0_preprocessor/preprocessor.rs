use super::comment_stripper::strip_comments;
use crate::common::error_handler::{CompileError, CompileResult};
use std::fs;
use std::path::{Path, PathBuf};

pub fn preprocess(source: &str) -> String {
    strip_comments(source)
}

pub fn output_path(input: &Path) -> PathBuf {
    let mut path = input.to_path_buf();
    path.set_extension("ipp");
    path
}

pub fn preprocess_file(input: &Path, output: Option<&Path>) -> CompileResult<PathBuf> {
    if input.extension().and_then(|e| e.to_str()) != Some("t") {
        return Err(CompileError::new(
            0,
            0,
            0,
            "source files must use the .t extension",
        ));
    }
    let source = fs::read_to_string(input).map_err(|e| {
        CompileError::new(0, 0, 0, format!("cannot read '{}': {}", input.display(), e))
    })?;
    let cleaned = preprocess(&source);
    let path = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| output_path(input));
    fs::write(&path, cleaned).map_err(|e| {
        CompileError::new(0, 0, 0, format!("cannot write '{}': {}", path.display(), e))
    })?;
    Ok(path)
}
