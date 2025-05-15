use crate::vm::{self, ArchState};

#[test]
fn test_accumulator() {
    const MEM: usize = 2_usize.pow(8);
    let mut state = ArchState::with_mem(MEM);

    let op = 0b1_00001_000_00001_0010011;
    state.load(
        (0..MEM)
            .map(|i| {
                let byte = 3 - (i % 4);
                (op >> (byte * 8)) as u8
            })
            .collect(),
        0,
    );
    println!(
        "mem: {:?}",
        state
            .mem
            .iter()
            .map(|n| format!("{:0>8b}", n))
            .collect::<Vec<String>>()
    );
    println!("op: {:?}", vm::interpret_bytes(op));

    let mut i = 0;
    while i < MEM / 4 {
        state.tick();
        i += 1;
        print!("{}, ", state.get_register(1));
        assert_eq!(state.get_register(1) as usize, i);
    }
}
