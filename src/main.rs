use std::{error::Error, fs, path::PathBuf};

use clap::{arg, command, Arg, ValueHint};
use ratatui::crossterm::{event::DisableMouseCapture, execute};

mod ui;
mod vm;

fn main() -> Result<(), Box<dyn Error>> {
    let args = command!()
        .arg(Arg::new("file").short('f').value_hint(ValueHint::FilePath))
        .get_matches();
    let default_program = if let Some(file) = args.get_one::<String>("file") {
        vec![(fs::read(file).unwrap(), 0)]
    } else {
        vec![
            (
                vec![
                    0x3e, 0x80, 0x00, 0x93, 0x7d, 0x00, 0x81, 0x13, 0xc1, 0x81, 0x01, 0x93, 0x83,
                    0x01, 0x82, 0x13, 0x3e, 0x82, 0x02, 0x93, 0x00, 0x01, 0x03, 0x17, 0xfe, 0xc3,
                    0x03, 0x13, 0x00, 0x43, 0x03, 0x13, 0x00, 0x03, 0x23, 0x83,
                ],
                0,
            ),
            (vec![0xde, 0xad, 0xbe, 0xef], 0x10004),
        ]
    };

    let res = ui::GUI::run_tui(default_program);
    ratatui::restore();
    execute!(std::io::stdout(), DisableMouseCapture)?;
    res
}
