use std::fmt::Display;

use crate::unique_ids::UniqueIdMaker;

/// A value ID -- a unique way to refer to a particular value in a program.
///
/// Multiple values in a program can be equivalent to each other, but have different Vids.
/// For example, two registers can both contain the number 5, but the Vids corresponding to
/// the values representing those two registers' states can be different.
///
/// A Vid represents a value in a program which was either created as the output of
/// a particular instruction in the program, or existed in the initial state of the program
/// when it started (e.g. as the initial value of a register at program startup).
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
    Exact(Vid, i64),
    Input(Vid, usize),
    Unknown(Vid),
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

impl Value {
    pub fn vid(&self) -> Vid {
        match self {
            Value::Exact(vid, _) | Value::Input(vid, _) | Value::Unknown(vid) => *vid,
        }
    }
}

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
