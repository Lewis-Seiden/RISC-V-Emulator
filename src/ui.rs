use std::{error::Error, fmt::format, io::Stdout, time::Duration};

use ratatui::{
    Frame, Terminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll, read},
    layout::{Constraint, Layout, Margin, Rect},
    prelude::{Backend, CrosstermBackend},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, Paragraph, Wrap},
};

use crate::vm::{self, ArchState, Instruction};

pub struct GUI {
    pause: bool,
    step: bool,
    arch_state: ArchState,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

#[derive(Default)]
struct Inputs {
    exit: bool,
    step: bool,
    toggle_pause: bool,
}

impl GUI {
    pub fn new() -> Self {
        Self {
            pause: true,
            step: false,
            arch_state: ArchState::new(),
            terminal: ratatui::init(),
        }
    }

    pub fn load(mut self, program: Vec<u8>, offset: usize) -> Self {
        self.arch_state.load(program, offset);
        self
    }

    pub fn get_state(&self) -> &ArchState {
        return &self.arch_state;
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            self.terminal.draw(|frame| {
                GUI::draw(
                    frame,
                    self.arch_state.pc as usize,
                    &(0..32).map(|i| self.arch_state.get_register(i)).collect(),
                    &vm::interpret_bytes(u32::from_be_bytes([
                        self.arch_state.mem[self.arch_state.pc as usize],
                        self.arch_state.mem[self.arch_state.pc as usize + 1],
                        self.arch_state.mem[self.arch_state.pc as usize + 2],
                        self.arch_state.mem[self.arch_state.pc as usize + 3],
                    ])),
                )
            })?;

            // TODO: clean
            let inputs = if poll(Duration::from_millis(20)).is_ok_and(|b| b) {
                if let Ok(event) = read() {
                    GUI::handle_input(event)
                } else {
                    Inputs::default()
                }
            } else {
                Inputs::default()
            };

            if inputs.exit {
                return Ok(());
            }

            self.step = inputs.step;
            self.pause = self.pause != inputs.toggle_pause;

            if self.step || !self.pause {
                self.arch_state.tick();
                self.step = false;
            }
        }
    }

    fn draw(frame: &mut Frame, pc: usize, registers: &Vec<u32>, instruction: &Instruction) {
        let columns = Layout::horizontal([Constraint::Min(6), Constraint::Min(0)]);
        let [register_area, main_area] = columns.areas(frame.area());
        let rhs_rows = Layout::vertical([Constraint::Fill(1), Constraint::Length(5)]);
        let [mem_area, control_area] = rhs_rows.areas(main_area);
        let register_area_block = Block::bordered();
        let mem_area_block = Block::bordered();
        let control_area_block = Block::bordered();
        frame.render_widget(&register_area_block, register_area);
        frame.render_widget(&mem_area_block, mem_area);
        frame.render_widget(&control_area_block, control_area);
        frame.render_widget(Text::raw("Hello World"), mem_area_block.inner(mem_area));

        const REGISTER_AREA_LINES: usize = 34;
        let register_lines: [Rect; REGISTER_AREA_LINES] =
            Layout::vertical([Constraint::Length(1); REGISTER_AREA_LINES])
                .areas(register_area_block.inner(register_area));

        frame.render_widget(
            Text::raw(format!("pc : 0x{0:0>8X} | {0:0>10} | ", pc)),
            register_lines[0],
        );

        (0..32).for_each(|i| {
            frame.render_widget(
                Text::styled(
                    format!(
                        "x{: <2}: 0x{1:0>8X} | {1:0>10} | ",
                        i,
                        registers.get(i).unwrap()
                    ),
                    if i % 2 == 0 {
                        Style::new().fg(Color::Black).bg(Color::Gray)
                    } else {
                        Style::new().fg(Color::Gray).bg(Color::Black)
                    },
                ),
                register_lines[i + 1],
            )
        });

        frame.render_widget(
            Paragraph::new(format!("{:?}", instruction)).wrap(Wrap::default()),
            register_lines[32].union(register_lines[33]),
        );
    }

    fn handle_input(event: Event) -> Inputs {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char(c) => Inputs {
                    exit: c == 'q',
                    step: false,
                    toggle_pause: c == ' ',
                },
                KeyCode::Right => Inputs {
                    exit: false,
                    step: true,
                    toggle_pause: false,
                },
                _ => Inputs {
                    exit: false,
                    step: false,
                    toggle_pause: false,
                },
            },
            _ => Inputs {
                exit: false,
                step: false,
                toggle_pause: false,
            },
        }
    }
}
