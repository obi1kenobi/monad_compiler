use std::fmt::Display;

use crate::unique_ids::UniqueIdMaker;

/// A value ID -- a unique way to refer to a particular value in a program.
///
/// If a program's register state contains two values with identical Vids,
/// those two registers' values are guaranteed to be identical. However,
/// just because two registers both currently hold the same number,
/// that does not mean their values must have the same Vid.
///
/// A Vid represents a value which was either created as the output
/// of a particular instruction in the program, or existed in the initial
/// state of the program when it started (e.g. as the initial value of
/// a register at program startup).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vid(pub usize);

impl Vid {
    /// Reserved Vid in case we need a way to refer to special-cased program values in the future.
    #[allow(dead_code)]
    pub const RESERVED: Vid = Vid(0);

    #[inline]
    pub fn unique_id_maker() -> UniqueIdMaker<Vid> {
        UniqueIdMaker::starting_at(1)
    }
}

impl From<usize> for Vid {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord)]
pub enum Value {
    Exact(Vid, i64),   // Exact(vid, k) is the constant value k.
    Input(Vid, usize), // Input(vid, i) is the i-th input to the program.
    Unknown(Vid),      // An unknown value in the program.
}

impl Value {
    pub fn vid(&self) -> Vid {
        match self {
            Value::Exact(vid, _) | Value::Input(vid, _) | Value::Unknown(vid) => *vid,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::Exact(_, left_val), &Self::Exact(_, right_val)) => left_val == right_val,
            (&l, &r) => l.vid() == r.vid(),
        }
    }
}

impl Eq for Value {}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Exact(v, val) => {
                write!(f, "{}: Exact({})", v.0, *val)
            }
            Value::Input(v, inp) => {
                write!(f, "{}: Input_{}", v.0, *inp)
            }
            Value::Unknown(v) => {
                write!(f, "{}: Unknown", v.0)
            }
        }
    }
}
