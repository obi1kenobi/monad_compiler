use crate::{program::Instruction, values::Vid, unique_ids::UniqueIdMaker};

/// An instruction ID -- a unique way to refer to a particular instruction in a program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstrId(pub usize);

impl From<usize> for InstrId {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

impl InstrId {
    #[inline]
    pub fn unique_id_maker() -> UniqueIdMaker<InstrId> {
        UniqueIdMaker::starting_at(0)
    }
}

/// An instruction in a program, with additional annotations based on our compiler's analysis.
///
/// As the compiler analyzes and optimizes a program, it will discover information about how
/// various program instructions affect the program's state. This struct allows us to uniquely
/// refer to a particular instruction in the program, and store the information the compiler
/// discovers in a handy and ergonomic format.
#[derive(Debug, Clone)]
pub struct AnnotatedInstr {
    pub id: InstrId,
    pub instr: Instruction,

    // Input instructions have Vid::UNDEFINED as source and operand,
    // since they don't read the value in the destination register, and never have an operand.
    pub source: Vid,
    pub operand: Vid,

    pub result: Vid,
}
