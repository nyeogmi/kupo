pub struct Bytecode {
    pub instructions: Vec<Instruction>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Register {
    Arg(usize),
    Local(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GenInstruction<Write, Read, Label> {
    Move { out: Write, arg: Read },

    // FFI to Rust
    RustCallMut { rust_fn: usize, out: Write, arg: Read },
    RustCallRef { rust_fn: usize, arg: Read},

    Jump { label: Label },
    Return {},
}

pub type Instruction = GenInstruction<Register, Register, usize>;

impl<Write: PartialEq<Write>, Read: PartialEq<Read>, Label: PartialEq<Label>> GenInstruction<Write, Read, Label> {
    pub fn is_jump(&self) -> bool {
        true
    }

    pub fn for_populated_registers(&self, f: impl FnMut(&Write)) {
    }

    pub fn for_jumped_labels(&self, mut f: impl FnMut(&Label)) {
        match self {
            GenInstruction::Move { out, arg } => {}
            GenInstruction::RustCallMut { .. } => {}
            GenInstruction::RustCallRef { .. } => {}
            GenInstruction::Jump { label } => { f(label); }
            GenInstruction::Return {  } => {}
        }
    }

    pub fn map_write<Write2>(self, f: impl Fn(Write) -> Write2) -> GenInstruction<Write2, Read, Label> {
        todo!()
    }

    pub fn map_read<Read2>(self, f: impl Fn(Read) -> Read2) -> GenInstruction<Write, Read2, Label> {
        todo!()
    }

    pub fn map_label<Label2>(self, f: impl Fn(Label) -> Label2) -> GenInstruction<Write, Read, Label2> {
        todo!()
    }

    pub fn replace_read(self, old: &Read, new: impl Fn() -> Read) -> Self {
        self.map_read(|r| if &r == old { new() } else { r })
    }

    pub fn replace_jump(self, old: &Label, new: impl Fn() -> Label) -> Self {
        self.map_label(|l| if &l == old { new() } else { l })
    }
}