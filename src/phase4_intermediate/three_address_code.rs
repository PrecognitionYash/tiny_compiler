use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quadruple {
    pub op: String,
    pub arg1: String,
    pub arg2: String,
    pub result: String,
    pub line: usize,
}

impl Quadruple {
    pub fn new(
        op: impl Into<String>,
        arg1: impl Into<String>,
        arg2: impl Into<String>,
        result: impl Into<String>,
        line: usize,
    ) -> Self {
        Self {
            op: op.into(),
            arg1: arg1.into(),
            arg2: arg2.into(),
            result: result.into(),
            line,
        }
    }

    pub fn is_branch(&self) -> bool {
        self.op.starts_with("BR_")
    }
    pub fn is_terminator(&self) -> bool {
        self.is_branch() || self.op == "GOTO" || self.op == "RET" || self.op == "FUNC_END"
    }

    fn pretty(value: &str) -> &str {
        if let Some(temp) = value.strip_prefix('$') {
            temp
        } else {
            value
        }
    }
}

impl fmt::Display for Quadruple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let a = if self.arg1.is_empty() {
            "-"
        } else {
            Self::pretty(&self.arg1)
        };
        let b = if self.arg2.is_empty() {
            "-"
        } else {
            Self::pretty(&self.arg2)
        };
        let r = if self.result.is_empty() {
            "-"
        } else {
            Self::pretty(&self.result)
        };
        write!(f, "({:>8}, {:>10}, {:>10}, {:>10})", self.op, a, b, r)
    }
}
