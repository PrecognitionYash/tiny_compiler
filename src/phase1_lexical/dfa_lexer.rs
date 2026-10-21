use super::token::{Token, TokenKind};
use crate::common::error_handler::{CompileError, CompileResult};

pub struct DfaLexer;

// Phase 1 Lexer: Tokenizer with indentation stack (like Python).
// Hand-written DFA style state machine for scanning lexemes.
impl DfaLexer {
    pub fn tokenize(source: &str) -> CompileResult<Vec<Token>> {
        let mut tokens = Vec::new();
        // indentation stack: keeps track of spaces for nesting blocks
        let mut indentation: Vec<usize> = vec![0];
        let mut last_line = 1usize;

        for (line_index, raw_line) in source.split('\n').enumerate() {
            let line_number = line_index + 1;
            last_line = line_number;
            let chars: Vec<char> = raw_line.chars().collect();
            let mut cursor = 0usize;
            let mut indent_width = 0usize;
            // count spaces and tabs to detect indent change
            while cursor < chars.len() {
                match chars[cursor] {
                    ' ' => {
                        indent_width += 1;
                        cursor += 1;
                    }
                    '\t' => {
                        indent_width = ((indent_width / 4) + 1) * 4;
                        cursor += 1;
                    }
                    _ => break,
                }
            }
            if cursor >= chars.len() {
                continue;
            }

            let current_indent = *indentation.last().unwrap_or(&0);
            if indent_width > current_indent {
                indentation.push(indent_width);
                tokens.push(Token::new(TokenKind::Indent, "<INDENT>", line_number, 1));
            } else if indent_width < current_indent {
                while indentation.last().copied().unwrap_or(0) > indent_width {
                    indentation.pop();
                    tokens.push(Token::new(
                        TokenKind::Dedent,
                        "<DEDENT>",
                        line_number,
                        indent_width + 1,
                    ));
                }
                if indentation.last().copied().unwrap_or(0) != indent_width {
                    return Err(CompileError::new(
                        1,
                        line_number,
                        1,
                        format!(
                            "indentation width {} does not match an earlier block level",
                            indent_width
                        ),
                    ));
                }
            }

            while cursor < chars.len() {
                let ch = chars[cursor];
                if ch.is_whitespace() {
                    cursor += 1;
                    continue;
                }
                let column = cursor + 1;
                if ch.is_ascii_alphabetic() || ch == '_' {
                    let start = cursor;
                    cursor += 1;
                    while cursor < chars.len()
                        && (chars[cursor].is_ascii_alphanumeric() || chars[cursor] == '_')
                    {
                        cursor += 1;
                    }
                    let word: String = chars[start..cursor].iter().collect();
                    let kind = match word.as_str() {
                        "def" => TokenKind::Def,
                        "return" => TokenKind::Return,
                        "if" => TokenKind::If,
                        "else" => TokenKind::Else,
                        "for" => TokenKind::For,
                        "in" => TokenKind::In,
                        _ => TokenKind::Identifier,
                    };
                    tokens.push(Token::new(kind, word, line_number, column));
                    continue;
                }
                if ch.is_ascii_digit() {
                    let start = cursor;
                    cursor += 1;
                    while cursor < chars.len() && chars[cursor].is_ascii_digit() {
                        cursor += 1;
                    }
                    let number: String = chars[start..cursor].iter().collect();
                    tokens.push(Token::new(TokenKind::Integer, number, line_number, column));
                    continue;
                }

                let two = if cursor + 1 < chars.len() {
                    Some((ch, chars[cursor + 1]))
                } else {
                    None
                };
                let pair_kind = match two {
                    Some(('.', '.')) => Some(TokenKind::Range),
                    Some(('<', '<')) => Some(TokenKind::ShiftLeft),
                    Some(('>', '>')) => Some(TokenKind::ShiftRight),
                    Some(('=', '=')) => Some(TokenKind::EqualEqual),
                    Some(('!', '=')) => Some(TokenKind::NotEqual),
                    Some(('<', '=')) => Some(TokenKind::LessEqual),
                    Some(('>', '=')) => Some(TokenKind::GreaterEqual),
                    _ => None,
                };
                if let Some(kind) = pair_kind {
                    let lexeme: String = chars[cursor..cursor + 2].iter().collect();
                    tokens.push(Token::new(kind, lexeme, line_number, column));
                    cursor += 2;
                    continue;
                }
                let kind = match ch {
                    ':' => TokenKind::Colon,
                    ',' => TokenKind::Comma,
                    '(' => TokenKind::LParen,
                    ')' => TokenKind::RParen,
                    '=' => TokenKind::Assign,
                    '+' => TokenKind::Plus,
                    '-' => TokenKind::Minus,
                    '*' => TokenKind::Star,
                    '/' => TokenKind::Slash,
                    '&' => TokenKind::Amp,
                    '|' => TokenKind::Pipe,
                    '^' => TokenKind::Caret,
                    '<' => TokenKind::Less,
                    '>' => TokenKind::Greater,
                    _ => {
                        return Err(CompileError::new(
                            1,
                            line_number,
                            column,
                            format!("unexpected character {:?}", ch),
                        ))
                    }
                };
                tokens.push(Token::new(kind, ch.to_string(), line_number, column));
                cursor += 1;
            }
            tokens.push(Token::new(
                TokenKind::Newline,
                "\\n",
                line_number,
                raw_line.len() + 1,
            ));
        }

        // popping all remaining indents at EOF so no blocks remain unclosed
        while indentation.len() > 1 {
            indentation.pop();
            tokens.push(Token::new(
                TokenKind::Dedent,
                "<DEDENT>",
                last_line.max(1),
                1,
            ));
        }
        tokens.push(Token::new(TokenKind::Eof, "<EOF>", last_line.max(1), 1));
        Ok(tokens)
    }
}

pub fn tokenize(source: &str) -> CompileResult<Vec<Token>> {
    DfaLexer::tokenize(source)
}
