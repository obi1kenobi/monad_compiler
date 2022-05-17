use std::fmt::Display;

/// A value in the program being optimized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Exact(i64),   // Exact(k) is the constant value k.
    Input(usize), // Input(i) is the i-th input to the program.
    Unknown,      // An unknown value in the program.
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Exact(val) => {
                write!(f, "Exact({})", *val)
            }
            Value::Input(inp) => {
                write!(f, "Input_{}", *inp)
            }
            Value::Unknown => {
                write!(f, "Unknown")
            }
        }
    }
}
