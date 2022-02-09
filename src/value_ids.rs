#![allow(dead_code)]

use crate::optimization::Value;

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
}

/// A way to generate unique Vids in a program.
#[derive(Debug, Default)]
pub struct VidMaker {
    next_id: usize,
}

impl VidMaker {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    pub fn initial_registers_and_vid_maker() -> (Self, [Value; 4]) {
        let mut vid_maker = Self::new();

        let registers = [
            Value::Exact(vid_maker.next().unwrap(), 0),
            Value::Exact(vid_maker.next().unwrap(), 0),
            Value::Exact(vid_maker.next().unwrap(), 0),
            Value::Exact(vid_maker.next().unwrap(), 0),
        ];

        (vid_maker, registers)
    }

    pub fn make_new_vid(&mut self) -> Vid {
        self.next().unwrap()
    }
}

impl Iterator for VidMaker {
    type Item = Vid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_id == usize::MAX {
            None
        } else {
            let next_id = self.next_id;
            self.next_id += 1;
            Some(Vid(next_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::value_ids::{VidMaker, Vid};

    #[test]
    fn vid_maker_produces_vids() {
        let mut vid_maker = VidMaker::new();
        assert_eq!(Vid(1), vid_maker.next().unwrap());
        assert_eq!(Vid(2), vid_maker.next().unwrap());
    }
}
