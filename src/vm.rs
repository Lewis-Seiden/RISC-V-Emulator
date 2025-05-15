type NBits<const C: usize> = [bool; C];

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

type RegisterPointer = u8;
/** 12 Bit Immediate */
#[derive(Clone, Copy, Debug)]
struct SmallImmediate {
    val: u32,
}
/** 20 Bit Immediate */
#[derive(Clone, Copy, Debug)]
struct BigImmediate {
    val: u32,
}

impl Into<u32> for SmallImmediate {
    fn into(self) -> u32 {
        self.val
    }
}

impl From<u32> for SmallImmediate {
    fn from(value: u32) -> Self {
        Self { val: value }
    }
}

impl Into<u32> for BigImmediate {
    fn into(self) -> u32 {
        self.val
    }
}

impl From<u32> for BigImmediate {
    fn from(value: u32) -> Self {
        Self { val: value }
    }
}

trait SignExtend {
    fn sign_extend(&self) -> i32;
}

impl SignExtend for SmallImmediate {
    fn sign_extend(&self) -> i32 {
        let msb = self.val & 1 << 11 == 1;
        transmute_to_signed(if msb { self.val + 0xFFFFF000 } else { self.val })
    }
}

impl SignExtend for BigImmediate {
    fn sign_extend(&self) -> i32 {
        let msb = self.val & 1 << 11 == 1;
        transmute_to_signed(if msb { self.val + 0xFFFFF000 } else { self.val })
    }
}

// Instruction Formats
#[derive(Clone, Copy, Debug)]
pub struct R {
    rd: RegisterPointer,
    rs1: RegisterPointer,
    rs2: RegisterPointer,
}
#[derive(Clone, Copy, Debug)]
pub struct I {
    rd: RegisterPointer,
    rs1: RegisterPointer,
    imm: SmallImmediate,
}
#[derive(Clone, Copy, Debug)]
pub struct S {
    imm: SmallImmediate,
    rs1: RegisterPointer,
    rs2: RegisterPointer,
}
#[derive(Clone, Copy, Debug)]
pub struct U {
    rd: RegisterPointer,
    imm: BigImmediate,
}
#[derive(Clone, Copy, Debug)]
// Immediate mode variants
pub struct B {
    imm: SmallImmediate,
    rs1: RegisterPointer,
    rs2: RegisterPointer,
} // Variant of S
#[derive(Clone, Copy, Debug)]
pub struct J {
    rd: RegisterPointer,
    imm: BigImmediate,
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

pub struct ArchState {
    regs: [u32; 31], // x0 is handled in the getter
    pub pc: i64,     // must be able to be negative so we can jump to 0
    pub mem: Vec<u8>,
}

fn transmute_to_signed(unsigned: u32) -> i32 {
    unsafe { std::mem::transmute(unsigned) }
}

fn transmute_to_unsigned(signed: i32) -> u32 {
    unsafe { std::mem::transmute(signed) }
}

pub fn interpret_bytes(bytes: u32) -> Instruction {
    let opcode = bytes & 0b1111111;
    let func3 = (bytes & (0b111 << 11)) >> 11;
    let nop = Instruction::ADDI {
        data: I {
            rd: 0,
            rs1: 0,
            imm: SmallImmediate::from(0),
        },
    };
    match opcode {
        0b0110011 => {
            // integer register to register
            let data = R {
                rd: (bytes >> 7) as u8 & 0b11111,
                rs1: (bytes >> 15) as u8 & 0b11111,
                rs2: (bytes >> 20) as u8 & 0b11111,
            };
            // check func3 and 30 bit for function
            match func3 + (bytes >> 27) {
                0000 => Instruction::ADD { data },
                1000 => Instruction::SUB { data },
                0001 | 1001 => Instruction::SLL { data },
                0010 | 1010 => Instruction::SLT { data },
                0011 | 1011 => Instruction::SLTU { data },
                0100 | 1100 => Instruction::XOR { data },
                0101 => Instruction::SRL { data },
                1101 => Instruction::SRA { data },
                0110 | 1110 => Instruction::OR { data },
                0111 | 1111 => Instruction::AND { data },
                _ => nop,
            }
        }
        0b0010011 => {
            // integer register immediate
            let data = I {
                rd: (bytes >> 7) as u8 & 0b11111,
                rs1: (bytes >> 15) as u8 & 0b11111,
                imm: SmallImmediate::from(bytes >> 20),
            };
            match func3 {
                000 => Instruction::ADDI { data },
                010 => Instruction::SLTI { data },
                011 => Instruction::SLTUI { data },
                100 => Instruction::XORI { data },
                110 => Instruction::ORI { data },
                111 => Instruction::ANDI { data },
                001 => Instruction::SLLI { data },
                // Check f7
                101 => {
                    if bytes & 2_u32.pow(30) == 0 {
                        Instruction::SRLI { data }
                    } else {
                        Instruction::SRAI { data }
                    }
                }
                _ => nop,
            }
        }
        0b0100011 => {
            // store instructions
            let data = S {
                rs1: (bytes >> 15) as u8 & 0b11111,
                rs2: (bytes >> 20) as u8 & 0b11111,
                imm: SmallImmediate::from((bytes >> 7) & 0b11111 + (bytes >> 24)),
            };
            match func3 {
                000 => Instruction::SB { data },
                001 => Instruction::SH { data },
                010 => Instruction::SW { data },
                _ => nop,
            }
        }
        0b0000011 => {
            // load instructions
            let data = I {
                rd: (bytes >> 7) as u8 & 0b11111,
                rs1: (bytes >> 15) as u8 & 0b11111,
                imm: SmallImmediate::from(bytes >> 20),
            };
            match func3 {
                000 => Instruction::LB { data },
                001 => Instruction::LH { data },
                010 => Instruction::LW { data },
                100 => Instruction::LBU { data },
                101 => Instruction::LHU { data },
                _ => nop,
            }
        }
        0b1100111 => {
            // JALR
            Instruction::JALR {
                data: I {
                    rd: (bytes >> 7) as u8,
                    rs1: (bytes >> 15) as u8,
                    imm: SmallImmediate::from(bytes >> 20),
                },
            }
        }
        0b1100011 => {
            // Branch
            let data = B {
                rs1: (bytes >> 15) as u8 & 0b11111,
                rs2: (bytes >> 20) as u8 & 0b11111,
                imm: SmallImmediate::from(
                    (((bytes >> 7) & 0b11111 +
                    (bytes >> 24)) & 0b111111111100) +
                    // lower order bits are moved to higher order for branches
                    ((bytes & 128) << (11 - 7)) +
                    ((bytes & 2_u32.pow(31) >> (31 - 12))),
                ),
            };
            match func3 {
                000 => Instruction::BEQ { data },
                001 => Instruction::BNE { data },
                100 => Instruction::BLT { data },
                101 => Instruction::BGE { data },
                110 => Instruction::BLTU { data },
                111 => Instruction::BGEU { data },
                _ => nop,
            }
        }
        0b1101111 => {
            // JAL
            Instruction::JAL {
                data: J {
                    rd: (bytes >> 7) as u8 & 0b11111,
                    imm: BigImmediate::from(
                        ((bytes >> 20) & 0b1111111111)
                            + (((bytes >> 20) & 1) << 10)
                            + (((bytes >> 12) & 0b11111111) << 11)
                            + (((bytes >> 30) & 1) << 19),
                    ),
                },
            }
        }
        0b0110111 => {
            // LUI
            Instruction::LUI {
                data: U {
                    rd: (bytes >> 7) as u8 & 0b11111,
                    imm: BigImmediate::from(bytes >> 12),
                },
            }
        }
        0b0010111 => {
            // AUIPC
            Instruction::AUIPC {
                data: U {
                    rd: (bytes >> 7) as u8 & 0b11111,
                    imm: BigImmediate::from(bytes >> 12),
                },
            }
        }
        // unknown instruction so no-op
        _ => nop,
    }
}

impl ArchState {
    pub fn new() -> Self {
        Self::with_mem(2_usize.pow(16))
    }

    pub fn with_mem(cap: usize) -> Self {
        Self {
            regs: [0; 31],
            pc: 0,
            mem: vec![0; cap],
        }
    }

    pub fn get_register(&self, reg: usize) -> u32 {
        if reg == 0 {
            return 0;
        }
        self.regs[reg - 1]
    }

    fn set_register(&mut self, index: usize, val: u32) {
        if index == 0 {
            return;
        }
        if let Some(reg) = self.regs.get_mut(index - 1) {
            *reg = val;
        }
    }

    pub fn load(&mut self, program: Vec<u8>, offset: usize) {
        (offset..offset + program.len()).for_each(|i| self.mem[i] = program[i - offset]);
    }

    pub fn apply(&mut self, inst: &Instruction) {
        match inst {
            // Register Arithmetic
            Instruction::ADD { data } => self.set_register(
                data.rd as usize,
                self.get_register(data.rs1 as usize) + self.get_register(data.rs2 as usize),
            ),
            Instruction::SUB { data } => self.set_register(
                data.rd as usize,
                self.get_register(data.rs1 as usize) - self.get_register(data.rs2 as usize),
            ),
            Instruction::XOR { data } => self.set_register(
                data.rd as usize,
                self.get_register(data.rs1 as usize) ^ self.get_register(data.rs2 as usize),
            ),
            Instruction::OR { data } => self.set_register(
                data.rd as usize,
                self.get_register(data.rs1 as usize) | self.get_register(data.rs2 as usize),
            ),
            Instruction::AND { data } => self.set_register(
                data.rd as usize,
                self.get_register(data.rs1 as usize) & self.get_register(data.rs2 as usize),
            ),
            // Shifts
            Instruction::SLL { data } => self.set_register(
                data.rd as usize,
                self.get_register(data.rs1 as usize) << self.get_register(data.rs2 as usize),
            ),
            Instruction::SRL { data } => self.set_register(
                data.rd as usize,
                self.get_register(data.rs1 as usize) >> self.get_register(data.rs2 as usize),
            ),
            Instruction::SRA { data } => self.set_register(
                data.rd as usize,
                transmute_to_unsigned(
                    transmute_to_signed(self.get_register(data.rs1 as usize))
                        >> self.get_register(data.rs2 as usize),
                ),
            ),
            // Register Comparisons
            Instruction::SLT { data } => self.set_register(
                data.rd as usize,
                if transmute_to_signed(self.get_register(data.rs1 as usize))
                    < transmute_to_signed(self.get_register(data.rs2 as usize))
                {
                    1
                } else {
                    0
                },
            ),
            Instruction::SLTU { data } => self.set_register(
                data.rd as usize,
                if self.get_register(data.rs1 as usize) < self.get_register(data.rs2 as usize) {
                    1
                } else {
                    0
                },
            ),
            // Immediate Arithmetic
            Instruction::ADDI { data } => self.set_register(
                data.rd as usize,
                transmute_to_unsigned(
                    transmute_to_signed(self.get_register(data.rs1 as usize))
                        + data.imm.sign_extend(),
                ),
            ),
            Instruction::XORI { data } => self.set_register(
                data.rd as usize,
                transmute_to_unsigned(
                    transmute_to_signed(self.get_register(data.rs1 as usize))
                        ^ data.imm.sign_extend(),
                ),
            ),
            Instruction::ORI { data } => self.set_register(
                data.rd as usize,
                transmute_to_unsigned(
                    transmute_to_signed(self.get_register(data.rs1 as usize))
                        | data.imm.sign_extend(),
                ),
            ),
            Instruction::ANDI { data } => self.set_register(
                data.rd as usize,
                transmute_to_unsigned(
                    transmute_to_signed(self.get_register(data.rs1 as usize))
                        & data.imm.sign_extend(),
                ),
            ),
            // Immediate Shifts
            Instruction::SLLI { data } => self.set_register(
                data.rd as usize,
                self.get_register(data.rs1 as usize) << data.imm.val,
            ),
            Instruction::SRLI { data } => self.set_register(
                data.rd as usize,
                self.get_register(data.rs1 as usize)
                // Skip first few bits because arithmetic vs logical shift is encoded in them
                    >> data.imm.val
                    & 0b11111,
            ),
            Instruction::SRAI { data } => self.set_register(
                data.rd as usize,
                transmute_to_unsigned(
                    transmute_to_signed(self.get_register(data.rs1 as usize))
                    // Skip first few bits because arithmetic vs logical shift is encoded in them
                        >> data.imm.val
                        & 0b11111,
                ),
            ),
            // Immediate Comparisons
            Instruction::SLTI { data } => self.set_register(
                data.rd as usize,
                if transmute_to_signed(self.get_register(data.rs1 as usize))
                    < data.imm.sign_extend()
                {
                    1
                } else {
                    0
                },
            ),
            Instruction::SLTUI { data } => self.set_register(
                data.rd as usize,
                if self.get_register(data.rs1 as usize)
                    < transmute_to_unsigned(data.imm.sign_extend())
                {
                    1
                } else {
                    0
                },
            ),
            // Loads
            Instruction::LBU { data } => self.set_register(
                data.rd as usize,
                *self
                    .mem
                    .get(
                        (self.get_register(data.rs1 as usize) as usize)
                            .wrapping_add_signed(data.imm.sign_extend() as isize),
                    )
                    .unwrap() as u32,
            ),
            Instruction::LHU { data } => {
                let index = (self.get_register(data.rs1 as usize) as usize)
                    .wrapping_add_signed(data.imm.sign_extend() as isize);
                self.set_register(
                    data.rd as usize,
                    (0..2)
                        .map(|offset| {
                            (*self.mem.get(index + offset).unwrap() as u32) << 8 * (1 - offset)
                        })
                        .sum::<u32>(),
                )
            }
            Instruction::LB { data } => {
                let val = *self
                    .mem
                    .get(
                        (self.get_register(data.rs1 as usize) as usize)
                            .wrapping_add_signed(data.imm.sign_extend() as isize),
                    )
                    .unwrap() as u32;
                self.set_register(
                    data.rd as usize,
                    // sign extension magic
                    // check if most significant defined bit is 1
                    // if so, set remaining significant bits to 1 with magic number
                    val + if val & 0b10000000 == 128 {
                        0xFFFFFF00
                    } else {
                        0
                    },
                );
            }
            Instruction::LH { data } => {
                let index = (self.get_register(data.rs1 as usize) as usize)
                    .wrapping_add_signed(data.imm.sign_extend() as isize);
                let val = (0..2)
                    .map(|offset| {
                        (*self.mem.get(index + offset).unwrap() as u32) << 8 * (1 - offset)
                    })
                    .sum::<u32>();
                self.set_register(
                    data.rd as usize,
                    // sign extension magic
                    // check if most significant defined bit is 1
                    // if so, set remaining significant bits to 1 with magic number
                    val + if val & (1 << 15) == 1 << 15 {
                        0xFFFF0000
                    } else {
                        0
                    },
                )
            }
            Instruction::LW { data } => {
                let index = (self.get_register(data.rs1 as usize) as usize)
                    .wrapping_add_signed(data.imm.sign_extend() as isize);
                self.set_register(
                    data.rd as usize,
                    (0..4)
                        .map(|offset| {
                            (*self.mem.get(index + offset).unwrap() as u32) << 8 * (3 - offset)
                        })
                        .sum::<u32>(),
                )
            }
            Instruction::SB { data } => {
                let index = self
                    .get_register(data.rs1 as usize)
                    .wrapping_add_signed(data.imm.sign_extend() as i32);
                self.mem[index as usize] = self.get_register(data.rs2 as usize) as u8;
            }
            Instruction::SH { data } => {
                let index = self
                    .get_register(data.rs1 as usize)
                    .wrapping_add_signed(data.imm.sign_extend() as i32);
                (0..2).for_each(|offset| {
                    self.mem[index as usize + offset] =
                        (self.get_register(data.rs2 as usize) >> 8 * (1 - offset)) as u8
                });
            }
            Instruction::SW { data } => {
                let index = self
                    .get_register(data.rs1 as usize)
                    .wrapping_add_signed(data.imm.sign_extend() as i32);
                (0..4).for_each(|offset| {
                    self.mem[index as usize + offset] =
                        (self.get_register(data.rs2 as usize) >> 8 * (3 - offset)) as u8
                });
            }
            Instruction::BEQ { data } => {
                self.pc += if self.get_register(data.rs1 as usize)
                    == self.get_register(data.rs2 as usize)
                {
                    // decrement because we will increment later
                    data.imm.sign_extend() * 2 - 4
                } else {
                    0
                } as i64
            }
            Instruction::BNE { data } => {
                self.pc += if self.get_register(data.rs1 as usize)
                    != self.get_register(data.rs2 as usize)
                {
                    // decrement because we will increment later
                    data.imm.sign_extend() * 2 - 4
                } else {
                    0
                } as i64
            }
            Instruction::BLT { data } => {
                self.pc += if transmute_to_signed(self.get_register(data.rs1 as usize))
                    < transmute_to_signed(self.get_register(data.rs2 as usize))
                {
                    // decrement because we will increment later
                    data.imm.sign_extend() * 2 - 4
                } else {
                    0
                } as i64
            }
            Instruction::BLTU { data } => {
                self.pc +=
                    if self.get_register(data.rs1 as usize) < self.get_register(data.rs2 as usize) {
                        // decrement because we will increment later
                        data.imm.sign_extend() * 2 - 4
                    } else {
                        0
                    } as i64
            }
            Instruction::BGE { data } => {
                self.pc += if transmute_to_signed(self.get_register(data.rs1 as usize))
                    >= transmute_to_signed(self.get_register(data.rs2 as usize))
                {
                    // decrement because we will increment later
                    data.imm.sign_extend() * 2 - 4
                } else {
                    0
                } as i64
            }
            Instruction::BGEU { data } => {
                self.pc += if self.get_register(data.rs1 as usize)
                    >= self.get_register(data.rs2 as usize)
                {
                    // decrement because we will increment later
                    data.imm.sign_extend() * 2 - 4
                } else {
                    0
                } as i64
            }
            Instruction::JAL { data } => {
                self.set_register(data.rd as usize, self.pc as u32 + 4);
                self.pc += data.imm.sign_extend() as i64 * 2 - 4;
            }
            Instruction::JALR { data } => {
                self.set_register(data.rd as usize, self.pc as u32 + 4);
                self.pc = (self
                    .get_register(data.rs1 as usize)
                    .saturating_add_signed(data.imm.sign_extend())
                    as i64
                    & 0xFFFE)
                    - 4;
            }
            Instruction::LUI { data } => {
                self.set_register(data.rd as usize, data.imm.val << 12);
            }
            Instruction::AUIPC { data } => {
                self.set_register(data.rd as usize, self.pc as u32 + (data.imm.val << 12));
            }
            _ => {
                panic!("Instruction Not Implemented!!")
            }
        }
        self.pc += 4;
    }

    pub fn tick(&mut self) {
        self.apply(&interpret_bytes(u32::from_be_bytes([
            self.mem[self.pc as usize],
            self.mem[self.pc as usize + 1],
            self.mem[self.pc as usize + 2],
            self.mem[self.pc as usize + 3],
        ])));
    }
}

#[test]
fn test_arithmetic() {
    let data = R {
        rd: 1,
        rs1: 2,
        rs2: 3,
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
        rd: 1,
        rs1: 2,
        rs2: 3,
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
        rd: 1,
        rs1: 2,
        rs2: 3,
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
        rd: 1,
        rs1: 2,
        rs2: 3,
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
        rd: 1,
        rs1: 2,
        imm: SmallImmediate::from(1),
    };
    for (inst, expected) in vec![
        (Instruction::ADDI { data: data.clone() }, 2),
        (Instruction::XORI { data: data.clone() }, 0),
        (Instruction::ORI { data: data.clone() }, 1),
        (Instruction::ANDI { data: data.clone() }, 1),
        (Instruction::SLLI { data: data.clone() }, 2),
        (Instruction::SRLI { data: data.clone() }, 0),
        (Instruction::SRAI { data: data.clone() }, 0),
    ] {
        let mut state = ArchState::new();
        state.set_register(2, 1);
        state.apply(&inst);
        println!("Test {:?}", &inst);
        assert_eq!(expected, state.get_register(1));
    }
}

#[test]
fn test_comparison_immediate() {
    let mut state = ArchState::new();
    state.set_register(2, 1);
    let data = I {
        rd: 1,
        rs1: 2,
        imm: SmallImmediate::from(2),
    };
    // signed
    let inst = Instruction::SLTI { data };
    state.apply(&inst);
    assert_eq!(1, state.get_register(1));
    // unsigned
    let inst = Instruction::SLTUI { data };
    state.apply(&inst);
    assert_eq!(1, state.get_register(1));
}

#[test]
fn test_loads() {
    let mut state = ArchState::new();
    state.mem.insert(0, 1);
    state.mem.insert(1, 2);
    state.mem.insert(2, 4);
    state.mem.insert(3, 8);
    state.mem.insert(4, 16);

    // byte
    state.apply(&Instruction::LB {
        data: I {
            rd: 1,
            rs1: 0,
            imm: SmallImmediate::from(0),
        },
    });
    assert_eq!(state.get_register(1), 1);
    // test offset
    state.apply(&Instruction::LB {
        data: I {
            rd: 1,
            rs1: 0,
            imm: SmallImmediate::from(1),
        },
    });
    assert_eq!(state.get_register(1), 2);

    // half
    state.apply(&Instruction::LH {
        data: I {
            rd: 1,
            rs1: 0,
            imm: SmallImmediate::from(0),
        },
    });
    assert_eq!(state.get_register(1), 258);
    // test offset
    state.apply(&Instruction::LH {
        data: I {
            rd: 1,
            rs1: 0,
            imm: SmallImmediate::from(1),
        },
    });
    assert_eq!(state.get_register(1), 258 << 1);

    // word
    state.apply(&Instruction::LW {
        data: I {
            rd: 1,
            rs1: 0,
            imm: SmallImmediate::from(0),
        },
    });
    assert_eq!(state.get_register(1), 16909320);
    // test offset
    state.apply(&Instruction::LW {
        data: I {
            rd: 1,
            rs1: 0,
            imm: SmallImmediate::from(1),
        },
    });
    assert_eq!(state.get_register(1), 16909320 << 1);
}

#[test]
fn test_stores() {
    let mut state = ArchState::new();

    state.set_register(1, 1 + (2 << 8) + (4 << 16) + (8 << 24));
    println!("register 1: {:b}", state.get_register(1));

    state.apply(&Instruction::SB {
        data: S {
            imm: SmallImmediate::from(0),
            rs1: 0,
            rs2: 1,
        },
    });
    assert_eq!(state.mem[0], 1);
    state.mem[0] = 0;

    state.apply(&Instruction::SH {
        data: S {
            imm: SmallImmediate::from(0),
            rs1: 0,
            rs2: 1,
        },
    });
    println!("{} {}", (state.mem[0] as u32), state.mem[1]);
    assert_eq!(
        ((state.mem[0] as u32) << 8) + state.mem[1] as u32,
        1_u32 + (2 << 8)
    );
    state.mem[0] = 0;
    state.mem[1] = 0;

    state.apply(&Instruction::SW {
        data: S {
            imm: SmallImmediate::from(0),
            rs1: 0,
            rs2: 1,
        },
    });
    println!("{} {}", (state.mem[0] as u32), state.mem[1]);
    assert_eq!(
        ((state.mem[0] as u32) << 24)
            + ((state.mem[1] as u32) << 16)
            + ((state.mem[2] as u32) << 8)
            + state.mem[3] as u32,
        1 + (2 << 8) + (4 << 16) + (8 << 24)
    );
}

#[test]
fn test_load_signs() {
    let mut state = ArchState::new();
    // byte loads
    state.mem[0] = 0b10000000;
    let test = I {
        imm: SmallImmediate::from(0),
        rs1: 1,
        rd: 4,
    };
    // unsigned load will 0 pad
    state.apply(&Instruction::LBU { data: test });
    println!("unsigned byte: {:b}", state.get_register(4));
    assert_eq!(state.get_register(4), 128);
    // signed will sign extend
    state.apply(&Instruction::LB { data: test });
    println!("signed byte: {:b}", state.get_register(4));
    assert_eq!(transmute_to_signed(state.get_register(4)), -128);

    // half loads
    let val = 1_u32 << 15;
    state.mem[0] = (val >> 8) as u8;
    state.mem[1] = val as u8;
    // unsigned load will 0 pad
    state.apply(&Instruction::LHU { data: test });
    println!("unsigned half: {:b}", state.get_register(4));
    assert_eq!(state.get_register(4), 1 << 15);
    // signed will sign extend
    state.apply(&Instruction::LH { data: test });
    println!("signed half: {:b}", state.get_register(4));
    assert_eq!(transmute_to_signed(state.get_register(4)), -(1_i32 << 15));
}

#[test]
fn test_conditional_jumps() {
    let mut state = ArchState::new();
    state.set_register(1, 1);
    state.set_register(2, 1);
    let test = B {
        rs1: 1,
        rs2: 2,
        imm: SmallImmediate::from(4),
    };

    state.apply(&Instruction::BEQ { data: test });
    assert_eq!(state.pc, 8);
    state.set_register(2, 0);
    state.apply(&Instruction::BEQ { data: test });
    assert_eq!(state.pc, 12);

    state.apply(&Instruction::BNE { data: test });
    assert_eq!(state.pc, 20);
    state.set_register(2, 1);
    state.apply(&Instruction::BNE { data: test });
    assert_eq!(state.pc, 24);

    state.apply(&Instruction::BLT { data: test });
    assert_eq!(state.pc, 28);
    state.set_register(2, 2);
    state.apply(&Instruction::BLT { data: test });
    assert_eq!(state.pc, 36);

    state.apply(&Instruction::BGE { data: test });
    assert_eq!(state.pc, 40);
    state.set_register(2, 1);
    state.apply(&Instruction::BGE { data: test });
    assert_eq!(state.pc, 48);
    state.set_register(2, 0);
    state.apply(&Instruction::BGE { data: test });
    assert_eq!(state.pc, 56);

    state.apply(&Instruction::BLTU { data: test });
    assert_eq!(state.pc, 28 + 32);
    state.set_register(2, 2);
    state.apply(&Instruction::BLTU { data: test });
    assert_eq!(state.pc, 36 + 32);

    state.apply(&Instruction::BGEU { data: test });
    assert_eq!(state.pc, 40 + 32);
    state.set_register(2, 1);
    state.apply(&Instruction::BGEU { data: test });
    assert_eq!(state.pc, 48 + 32);
    state.set_register(2, 0);
    state.apply(&Instruction::BGEU { data: test });
    assert_eq!(state.pc, 56 + 32);
}

#[test]
fn test_unconditional_jumps() {
    let mut state = ArchState::new();
    state.set_register(1, 1);

    state.apply(&Instruction::JAL {
        data: J {
            rd: 1,
            imm: BigImmediate::from(8),
        },
    });
    assert_eq!(state.pc, 16);
    assert_eq!(state.get_register(1), 4);

    state.apply(&Instruction::JALR {
        data: I {
            rd: 1,
            rs1: 0,
            imm: SmallImmediate::from(8),
        },
    });
    assert_eq!(state.pc, 8);
    assert_eq!(state.get_register(1), 20);
}

#[test]
fn test_lui_auipc() {
    let mut state = ArchState::new();

    let test = U {
        rd: 1,
        imm: BigImmediate::from(1 << 19),
    };

    state.apply(&Instruction::LUI { data: test });
    assert_eq!(state.get_register(1), 2_u32.pow(31));

    state.apply(&Instruction::AUIPC { data: test });
    assert_eq!(state.get_register(1), 2_u32.pow(31) + 4);
}
