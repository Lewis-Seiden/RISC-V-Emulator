use std::{
    error::Error,
    sync::{Arc, Mutex, mpsc::channel},
    thread,
};

use ratatui::crossterm::{event::DisableMouseCapture, execute};
use ui::GUI;
use vm::ArchState;

mod ui;
mod vm;

fn main() -> Result<(), Box<dyn Error>> {
    let res = run_tui(vec![
        (
            vec![
                0x3e, 0x80, 0x00, 0x93, 0x7d, 0x00, 0x81, 0x13, 0xc1, 0x81, 0x01, 0x93, 0x83, 0x01,
                0x82, 0x13, 0x3e, 0x82, 0x02, 0x93, 0x00, 0x01, 0x03, 0x17, 0xfe, 0xc3, 0x03, 0x13,
                0x00, 0x43, 0x03, 0x13, 0x00, 0x03, 0x23, 0x83,
            ],
            0,
        ),
        (vec![0xde, 0xad, 0xbe, 0xef], 0x10004),
    ]);
    ratatui::restore();
    execute!(std::io::stdout(), DisableMouseCapture)?;
    res
}

fn run_tui(to_load: Vec<(Vec<u8>, usize)>) -> Result<(), Box<dyn Error>> {
    let mut state = ArchState::new();
    for data in to_load {
        state.load(data.0, data.1);
    }

    let (mut gui, pause_rx, step_rx) = GUI::new();

    let state_mutex = Arc::new(Mutex::new(state));
    let (quit_tx, quit_rx) = channel();

    let arch_state_mutex = Arc::clone(&state_mutex);
    let _ = thread::spawn(move || {
        let mut inst_count = 0;
        let mut pause = true;
        while quit_rx.try_recv().is_err() {
            while pause && step_rx.try_recv().is_err() {
                match pause_rx.recv() {
                    Ok(b) => pause = b,
                    Err(_) => {}
                }
            }
            inst_count += 1;
            match arch_state_mutex.lock().unwrap().tick() {
                Ok(_) => {}
                Err(_) => break,
            }
        }
        println!("instructions run {}", inst_count)
    });

    gui.run(Arc::clone(&state_mutex))?;
    quit_tx.send(())?;
    Ok(())
}
