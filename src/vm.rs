use std::{
    fmt::{Display, Write},
    u32,
};

#[cfg(test)]
mod instruction_tests;
#[cfg(test)]
mod integration_tests;

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
        let msb = self.val & (1 << 11) != 0;
        transmute_to_signed(if msb { self.val + 0xFFFFF000 } else { self.val })
    }
}

impl SignExtend for BigImmediate {
    fn sign_extend(&self) -> i32 {
        let msb = self.val & (1 << 19) != 0;
        transmute_to_signed(if msb { self.val + 0xFFF00000 } else { self.val })
    }
}

#[test]
fn test_sign_extension() {
    assert_eq!(1, SmallImmediate::from(1).sign_extend());
    assert_eq!(-1, SmallImmediate::from(2_u32.pow(12) - 1).sign_extend());

    assert_eq!(1, BigImmediate::from(1).sign_extend());
    assert_eq!(-1, BigImmediate::from(2_u32.pow(20) - 1).sign_extend());
}

// Instruction Formats
#[derive(Clone, Copy, Debug)]
pub struct R {
    rd: RegisterPointer,
    rs1: RegisterPointer,
    rs2: RegisterPointer,
}

impl Display for R {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("rd:  x{} | ", self.rd))?;
        f.write_fmt(format_args!("rs1: x{} | ", self.rs1))?;
        f.write_fmt(format_args!("rs2: x{}", self.rs2))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct I {
    rd: RegisterPointer,
    rs1: RegisterPointer,
    imm: SmallImmediate,
}

impl Display for I {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("rd:  x{} | ", self.rd))?;
        f.write_fmt(format_args!("rs1: x{} | ", self.rs1))?;
        f.write_fmt(format_args!("imm: {:#014b}", self.imm.val))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct S {
    imm: SmallImmediate,
    rs1: RegisterPointer,
    rs2: RegisterPointer,
}
impl Display for S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("rs1: x{} | ", self.rs1))?;
        f.write_fmt(format_args!("rs2: x{} | ", self.rs2))?;
        f.write_fmt(format_args!("imm: {:#014b}", self.imm.val))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct U {
    rd: RegisterPointer,
    imm: BigImmediate,
}

impl Display for U {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("rd:  x{} | ", self.rd))?;
        f.write_fmt(format_args!("imm: {:#022b}", self.imm.val))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
// Immediate mode variants
pub struct B {
    imm: SmallImmediate,
    rs1: RegisterPointer,
    rs2: RegisterPointer,
} // Variant of S

impl Display for B {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("rs1: x{} | ", self.rs1))?;
        f.write_fmt(format_args!("rs2: x{} | ", self.rs2))?;
        f.write_fmt(format_args!("imm: {:#014b}", self.imm.val))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct J {
    rd: RegisterPointer,
    imm: BigImmediate,
} // Variant of U

impl Display for J {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("rd:  x{} | ", self.rd))?;
        f.write_fmt(format_args!("imm: {:#022b}", self.imm.val))?;
        Ok(())
    }
}

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

impl Instruction {
    fn get_payload(&self) -> String {
        match self {
            Instruction::ADD { data } => data.to_string(),
            Instruction::SUB { data } => data.to_string(),
            Instruction::XOR { data } => data.to_string(),
            Instruction::OR { data } => data.to_string(),
            Instruction::AND { data } => data.to_string(),
            Instruction::SLL { data } => data.to_string(),
            Instruction::SRL { data } => data.to_string(),
            Instruction::SRA { data } => data.to_string(),
            Instruction::SLT { data } => data.to_string(),
            Instruction::SLTU { data } => data.to_string(),
            Instruction::ADDI { data } => data.to_string(),
            Instruction::XORI { data } => data.to_string(),
            Instruction::ORI { data } => data.to_string(),
            Instruction::ANDI { data } => data.to_string(),
            Instruction::SLLI { data } => data.to_string(),
            Instruction::SRLI { data } => data.to_string(),
            Instruction::SRAI { data } => data.to_string(),
            Instruction::SLTI { data } => data.to_string(),
            Instruction::SLTUI { data } => data.to_string(),
            Instruction::LB { data } => data.to_string(),
            Instruction::LH { data } => data.to_string(),
            Instruction::LW { data } => data.to_string(),
            Instruction::LBU { data } => data.to_string(),
            Instruction::LHU { data } => data.to_string(),
            Instruction::SB { data } => data.to_string(),
            Instruction::SH { data } => data.to_string(),
            Instruction::SW { data } => data.to_string(),
            Instruction::BEQ { data } => data.to_string(),
            Instruction::BNE { data } => data.to_string(),
            Instruction::BLT { data } => data.to_string(),
            Instruction::BGE { data } => data.to_string(),
            Instruction::BLTU { data } => data.to_string(),
            Instruction::BGEU { data } => data.to_string(),
            Instruction::JAL { data } => data.to_string(),
            Instruction::JALR { data } => data.to_string(),
            Instruction::LUI { data } => data.to_string(),
            Instruction::AUIPC { data } => data.to_string(),
            Instruction::ECALL { data } => data.to_string(),
            Instruction::EBREAK { data } => data.to_string(),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format_args!("{:?}", self)
                .to_string()
                .split_whitespace()
                .next()
                .unwrap(),
        )?;
        f.write_fmt(format_args!(" {}", self.get_payload()))?;
        Ok(())
    }
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
    let func3 = (bytes & (0b111 << 12)) >> 12;
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
                0b0000 => Instruction::ADD { data },
                0b1000 => Instruction::SUB { data },
                0b0001 | 1001 => Instruction::SLL { data },
                0b0010 | 1010 => Instruction::SLT { data },
                0b0011 | 1011 => Instruction::SLTU { data },
                0b0100 | 1100 => Instruction::XOR { data },
                0b0101 => Instruction::SRL { data },
                0b1101 => Instruction::SRA { data },
                0b0110 | 1110 => Instruction::OR { data },
                0b0111 | 1111 => Instruction::AND { data },
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
                0b000 => Instruction::ADDI { data },
                0b010 => Instruction::SLTI { data },
                0b011 => Instruction::SLTUI { data },
                0b100 => Instruction::XORI { data },
                0b110 => Instruction::ORI { data },
                0b111 => Instruction::ANDI { data },
                0b001 => Instruction::SLLI { data },
                // Check f7
                0b101 => {
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
                0b000 => Instruction::SB { data },
                0b001 => Instruction::SH { data },
                0b010 => Instruction::SW { data },
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
                0b000 => Instruction::LB { data },
                0b001 => Instruction::LH { data },
                0b010 => Instruction::LW { data },
                0b100 => Instruction::LBU { data },
                0b101 => Instruction::LHU { data },
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
                0b000 => Instruction::BEQ { data },
                0b001 => Instruction::BNE { data },
                0b100 => Instruction::BLT { data },
                0b101 => Instruction::BGE { data },
                0b110 => Instruction::BLTU { data },
                0b111 => Instruction::BGEU { data },
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
        Self::with_mem(2_usize.pow(32))
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
