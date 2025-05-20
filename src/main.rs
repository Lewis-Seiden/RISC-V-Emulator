use ui::GUI;
use vm::ArchState;

mod ui;
mod vm;

fn main() {
    let _ = GUI::new().run();
    ratatui::restore();
}
