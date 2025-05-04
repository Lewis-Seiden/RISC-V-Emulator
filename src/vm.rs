type NBits<const C: usize> = [bool; C];

// impl<const C: usize> NBits<C> {
pub trait Bits {
    fn sign_extend(&self) -> i32;

    fn unsigned(&self) -> u32;
}

impl<const C: usize> Bits for NBits<C> {
    fn sign_extend(&self) -> i32 {
        let msb = self[0];
        (0..32_usize).into_iter().fold(0, |a, i| {
            let b = a << 1;
            if i >= 32 - self.len() {
                b + if *self.get(i + self.len() - 32).unwrap_or(&msb) {
                    1
                } else {
                    0
                }
            } else {
                b
            }
        })
    }

    fn unsigned(&self) -> u32 {
        (0..32_usize).into_iter().fold(0, |a, i| {
            let b = a << 1;
            if i >= 32 - self.len() {
                b + if *self.get(i + self.len() - 32).unwrap_or(&false) {
                    1
                } else {
                    0
                }
            } else {
                b
            }
        })
    }
}

#[test]
fn test_sign_extend() {
    println!("0: {:b}", [false, false, false, false].sign_extend());
    assert_eq!(0, [false, false, false, false].sign_extend());
    println!("1: {:b}", [false, false, false, true].sign_extend());
    assert_eq!(1, [false, false, false, true].sign_extend());
    assert_eq!(8, [true, false, false, false].sign_extend());
    assert_eq!(12, [true, true, false, false].sign_extend());
    println!("max: {:b}", [true; 32].sign_extend());
    assert_eq!(
        i32::from_be_bytes([255, 255, 255, 255]),
        [true; 32].sign_extend()
    );
}

#[test]
fn test_unsigned() {
    println!("0: {:b}", [false, false, false, false].unsigned());
    assert_eq!(0, [false, false, false, false].unsigned());
    println!("1: {:b}", [false, false, false, true].unsigned());
    assert_eq!(1, [false, false, false, true].unsigned());
    assert_eq!(8, [true, false, false, false].unsigned());
    assert_eq!(12, [true, true, false, false].unsigned());
    println!("max: {:b}", [true; 32].unsigned());
    assert_eq!(
        u32::from_be_bytes([255, 255, 255, 255]),
        [true; 32].unsigned()
    );
}

type FiveBits = [bool; 5];
type TwelveBits = [bool; 12];
type TwentyBits = [bool; 20];

// Instruction Formats
#[derive(Clone, Copy, Debug)]
pub struct R {
    rd: FiveBits,
    rs1: FiveBits,
    rs2: FiveBits,
}
#[derive(Clone, Copy, Debug)]
pub struct I {
    rd: FiveBits,
    rs1: FiveBits,
    imm: TwelveBits,
}
#[derive(Clone, Copy, Debug)]
pub struct S {
    imm: TwelveBits,
    rs1: FiveBits,
    rs2: FiveBits,
}
#[derive(Clone, Copy, Debug)]
pub struct U {
    rd: FiveBits,
    imm: TwentyBits,
}
#[derive(Clone, Copy, Debug)]
// Immediate mode variants
pub struct B {
    imm: TwelveBits,
    rs1: FiveBits,
    rs2: FiveBits,
} // Variant of S
#[derive(Clone, Copy, Debug)]
pub struct J {
    rd: FiveBits,
    imm: TwentyBits,
} // Variant of U

#[derive(Debug)]
pub enum Instruction {
    ADD { data: R },
    SUB { data: R },
    XOR { data: R },
    OR { data: R },
    AND { data: R },
    SLL { data: R },
    SRL { data: R },
    SRA { data: R },
    SLT { data: R },
    SLTU { data: R },

    ADDI { data: I },
    XORI { data: I },
    ORI { data: I },
    ANDI { data: I },
    SLLI { data: I },
    SRLI { data: I },
    SRAI { data: I },
    SLTI { data: I },
    SLTUI { data: I },

    LB { data: I },
    LH { data: I },
    LW { data: I },
    LBU { data: I },
    LHU { data: I },

    SB { data: S },
    SH { data: S },
    SW { data: S },

    BEQ { data: B },
    BNE { data: B },
    BLT { data: B },
    BGE { data: B },
    BLTU { data: B },
    BGEU { data: B },

    JAL { data: J },
    JALR { data: I },

    LUI { data: U },
    AUIPC { data: U },

    ECALL { data: I },
    EBREAK { data: I },
}

struct ArchState {
    regs: [u32; 31], // x0 is handled in the getter
    pc: u32,
}

fn transmute_to_signed(unsigned: u32) -> i32 {
    unsafe { std::mem::transmute(unsigned) }
}

fn transmute_to_unsigned(signed: i32) -> u32 {
    unsafe { std::mem::transmute(signed) }
}

impl ArchState {
    pub fn new() -> Self {
        Self {
            regs: [0; 31],
            pc: 0,
        }
    }

    pub fn get_register(&self, reg: usize) -> u32 {
        if reg == 0 {
            return 0;
        }
        self.regs[reg - 1]
    }

    fn set_register(&mut self, reg: usize, val: u32) {
        self.regs[reg - 1] = val;
    }

    pub fn apply(&mut self, inst: &Instruction) {
        match inst {
            // Register Arithmetic
            Instruction::ADD { data } => self.set_register(
                data.rd.unsigned() as usize,
                self.get_register(data.rs1.unsigned() as usize)
                    + self.get_register(data.rs2.unsigned() as usize),
            ),
            Instruction::SUB { data } => self.set_register(
                data.rd.unsigned() as usize,
                self.get_register(data.rs1.unsigned() as usize)
                    - self.get_register(data.rs2.unsigned() as usize),
            ),
            Instruction::XOR { data } => self.set_register(
                data.rd.unsigned() as usize,
                self.get_register(data.rs1.unsigned() as usize)
                    ^ self.get_register(data.rs2.unsigned() as usize),
            ),
            Instruction::OR { data } => self.set_register(
                data.rd.unsigned() as usize,
                self.get_register(data.rs1.unsigned() as usize)
                    | self.get_register(data.rs2.unsigned() as usize),
            ),
            Instruction::AND { data } => self.set_register(
                data.rd.unsigned() as usize,
                self.get_register(data.rs1.unsigned() as usize)
                    & self.get_register(data.rs2.unsigned() as usize),
            ),
            // Shifts
            Instruction::SLL { data } => self.set_register(
                data.rd.unsigned() as usize,
                self.get_register(data.rs1.unsigned() as usize)
                    << self.get_register(data.rs2.unsigned() as usize),
            ),
            Instruction::SRL { data } => self.set_register(
                data.rd.unsigned() as usize,
                self.get_register(data.rs1.unsigned() as usize)
                    >> self.get_register(data.rs2.unsigned() as usize),
            ),
            Instruction::SRA { data } => self.set_register(
                data.rd.unsigned() as usize,
                transmute_to_unsigned(
                    transmute_to_signed(self.get_register(data.rs1.unsigned() as usize))
                        >> self.get_register(data.rs2.unsigned() as usize),
                ),
            ),
            // Register Comparisons
            Instruction::SLT { data } => self.set_register(
                data.rd.unsigned() as usize,
                if transmute_to_signed(self.get_register(data.rs1.unsigned() as usize))
                    < transmute_to_signed(self.get_register(data.rs2.unsigned() as usize))
                {
                    1
                } else {
                    0
                },
            ),
            Instruction::SLTU { data } => self.set_register(
                data.rd.unsigned() as usize,
                if self.get_register(data.rs1.unsigned() as usize)
                    < self.get_register(data.rs2.unsigned() as usize)
                {
                    1
                } else {
                    0
                },
            ),
            // Immediate Arithmetic
            Instruction::ADDI { data } => self.set_register(
                data.rd.unsigned() as usize,
                transmute_to_unsigned(
                    transmute_to_signed(self.get_register(data.rs1.unsigned() as usize))
                        + data.imm.sign_extend(),
                ),
            ),
            Instruction::XORI { data } => self.set_register(
                data.rd.unsigned() as usize,
                transmute_to_unsigned(
                    transmute_to_signed(self.get_register(data.rs1.unsigned() as usize))
                        ^ data.imm.sign_extend(),
                ),
            ),
            Instruction::ORI { data } => self.set_register(
                data.rd.unsigned() as usize,
                transmute_to_unsigned(
                    transmute_to_signed(self.get_register(data.rs1.unsigned() as usize))
                        | data.imm.sign_extend(),
                ),
            ),
            Instruction::ANDI { data } => self.set_register(
                data.rd.unsigned() as usize, transmute_to_unsigned(
                    transmute_to_signed(self.get_register(data.rs1.unsigned() as usize))
                        & data.imm.sign_extend(),
                ),
            ),
            _ => {
                panic!("Instruction Not Implemented!!")
            }
        }
        self.pc += 4;
    }
}

#[test]
fn test_arithmetic() {
    let data = R {
        rd: [false, false, false, false, true],
        rs1: [false, false, false, true, false],
        rs2: [false, false, false, true, true],
    };
    for (inst, expected) in vec![
        (Instruction::ADD { data: data.clone() }, 2),
        (Instruction::SUB { data: data.clone() }, 0),
        (Instruction::XOR { data: data.clone() }, 0),
        (Instruction::OR { data: data.clone() }, 1),
        (Instruction::AND { data: data.clone() }, 1),
        (Instruction::SLL { data: data.clone() }, 2),
        (Instruction::SRL { data: data.clone() }, 0),
        (Instruction::SRA { data: data.clone() }, 0),
    ] {
        let mut state = ArchState::new();
        state.set_register(2, 1);
        state.set_register(3, 1);
        state.apply(&inst);
        println!("Test {:?}", &inst);
        assert_eq!(expected, state.get_register(1));
    }
}

#[test]
fn test_shift_right_logical() {
    let mut state = ArchState::new();
    state.set_register(2, 2_u32.pow(31));
    state.set_register(3, 1);
    let data = R {
        rd: [false, false, false, false, true],
        rs1: [false, false, false, true, false],
        rs2: [false, false, false, true, true],
    };
    let inst = Instruction::SRL { data };
    state.apply(&inst);
    println!(
        "rs1: {:#034b}, rs2:      {:#034b}",
        state.get_register(2),
        state.get_register(3)
    );
    println!(
        "rd:  {:#034b}, expected: {:#034b}",
        state.get_register(1),
        2_u32.pow(31)
    );
    assert_eq!(2_u32.pow(30), state.get_register(1));
}

#[test]
fn test_shift_right_arithmetic() {
    let mut state = ArchState::new();
    state.set_register(2, 2_u32.pow(31));
    state.set_register(3, 1);
    let data = R {
        rd: [false, false, false, false, true],
        rs1: [false, false, false, true, false],
        rs2: [false, false, false, true, true],
    };
    let inst = Instruction::SRA { data };
    state.apply(&inst);
    println!(
        "rs1: {:#034b}, rs2:      {:#034b}",
        state.get_register(2),
        state.get_register(3)
    );
    println!(
        "rd:  {:#034b}, expected: {:#034b}",
        state.get_register(1),
        2_u32.pow(30) + 2_u32.pow(31)
    );
    assert_eq!(2_u32.pow(30) + 2_u32.pow(31), state.get_register(1));
}

#[test]
fn test_comparison() {
    let mut state = ArchState::new();
    state.set_register(2, 1);
    state.set_register(3, 2);
    let data = R {
        rd: [false, false, false, false, true],
        rs1: [false, false, false, true, false],
        rs2: [false, false, false, true, true],
    };
    // signed
    let inst = Instruction::SLT { data };
    state.apply(&inst);
    assert_eq!(1, state.get_register(1));
    // unsigned
    let inst = Instruction::SLTU { data };
    state.apply(&inst);
    assert_eq!(1, state.get_register(1));
}

#[test]
fn test_immediate_arithmetic() {
    let data = I {
        rd: [false, false, false, false, true],
        rs1: [false, false, false, true, false],
        imm: [
            false, false, false, false, false, false, false, false, false, false, false, true,
        ],
    };
    for (inst, expected) in vec![
        (Instruction::ADDI { data: data.clone() }, 2),
        (Instruction::XORI { data: data.clone() }, 0),
        (Instruction::ORI { data: data.clone() }, 1),
        (Instruction::ANDI { data: data.clone() }, 1),
        // (Instruction::SLL { data: data.clone() }, 2),
        // (Instruction::SRL { data: data.clone() }, 0),
        // (Instruction::SRA { data: data.clone() }, 0),
    ] {
        let mut state = ArchState::new();
        state.set_register(2, 1);
        state.set_register(3, 1);
        state.apply(&inst);
        println!("Test {:?}", &inst);
        assert_eq!(expected, state.get_register(1));
    }
}
