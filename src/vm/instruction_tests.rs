use crate::vm::{ArchState, B, BigImmediate, J, S, U, transmute_to_signed};

use super::{I, Instruction, R, SmallImmediate};

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
