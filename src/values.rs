#![allow(dead_code)]

use std::{collections::{BTreeMap, BTreeSet}, fmt::Display};

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
    /// Reserved Vid in case we need a way to refer to program values whose contents are undefined.
    pub const UNDEFINED: Vid = Vid(0);

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

pub fn vid_maker_and_initial_registers() -> (UniqueIdMaker<Vid>, [Value; 4]) {
    let mut vid_maker = Vid::unique_id_maker();

    let registers = [
        Value::Exact(vid_maker.make_new_id(), 0),
        Value::Exact(vid_maker.make_new_id(), 0),
        Value::Exact(vid_maker.make_new_id(), 0),
        Value::Exact(vid_maker.make_new_id(), 0),
    ];

    (vid_maker, registers)
}

pub struct ProgramValues {
    vid_maker: UniqueIdMaker<Vid>,
    inputs_seen: usize,
    values: BTreeMap<Vid, Value>,
    used_values: BTreeSet<Vid>,
}

impl ProgramValues {
    pub fn new() -> Self {
        Self {
            vid_maker: Vid::unique_id_maker(),
            inputs_seen: 0,
            values: Default::default(),
            used_values: Default::default(),
        }
    }

    pub fn register_input_value(&mut self) -> Value {
        let input_number = self.inputs_seen;
        self.inputs_seen += 1;

        let vid = self.vid_maker.make_new_id();
        let value = Value::Input(vid, input_number);
        self.register_value(value)
    }

    pub fn register_exact_value(&mut self, exact_value: i64) -> Value {
        let vid = self.vid_maker.make_new_id();
        let value = Value::Exact(vid, exact_value);
        self.register_value(value)
    }

    pub fn register_unknown_value(&mut self) -> Value {
        let vid = self.vid_maker.make_new_id();
        let value = Value::Unknown(vid);
        self.register_value(value)
    }

    fn register_value(&mut self, value: Value) -> Value {
        let existing = self.values.insert(value.vid(), value);
        assert!(existing.is_none());
        value
    }

    pub fn get(&self, vid: Vid) -> Value {
        *self.values.get(&vid).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::values::Vid;
    use crate::unique_ids::UniqueIdMaker;

    #[test]
    fn vid_maker_produces_vids() {
        let mut vid_maker = UniqueIdMaker::starting_at(1);
        assert_eq!(Vid(1), vid_maker.next().unwrap());
        assert_eq!(Vid(2), vid_maker.next().unwrap());
    }
}
