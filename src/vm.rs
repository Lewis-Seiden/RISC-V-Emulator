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
struct R {
    rd: FiveBits,
    rs1: FiveBits,
    rs2: FiveBits,
}
struct I {
    rd: FiveBits,
    rs1: FiveBits,
    imm: TwelveBits,
}
struct S {
    imm: TwelveBits,
    rs1: FiveBits,
    rs2: FiveBits,
}
struct U {
    rd: FiveBits,
    imm: TwentyBits,
}
// Immediate mode variants
struct B {
    imm: TwelveBits,
    rs1: FiveBits,
    rs2: FiveBits,
} // Variant of S
struct J {
    rd: FiveBits,
    imm: TwentyBits,
} // Variant of U

enum Instruction {
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
        self.regs[reg + 1]
    }

    fn set_register(&mut self, reg: usize, val: u32) {
        self.regs[reg + 1] = val;
    }

    pub fn apply(&mut self, inst: Instruction) {
        match inst {
            Instruction::ADD { data } => self.set_register(
                data.rd.unsigned() as usize,
                self.get_register(data.rs1.unsigned() as usize)
                    + self.get_register(data.rs2.unsigned() as usize),
            ),
            _ => {}
        }
        self.pc += 4;
    }
}

#[test]
fn test_add() {
    let mut state = ArchState::new();
    state.set_register(2, 1);
    state.set_register(3, 1);
    state.apply(Instruction::ADD {
        data: R {
            rd: [false, false, false, false, true],
            rs1: [false, false, false, true, false],
            rs2: [false, false, false, true, true],
        },
    });
    assert_eq!(2, state.get_register(1));
}
