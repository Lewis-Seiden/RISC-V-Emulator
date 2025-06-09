use std::{error::Error, io::Stdout, time::Duration};

use ratatui::{
    Frame, Terminal,
    crossterm::event::{
        Event, KeyCode, MouseEventKind, poll, read,
    },
    layout::{Constraint, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::Text,
    widgets::{
        Block, Cell, Row, ScrollDirection, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};

use crate::vm::{self, ArchState, Instruction};

pub struct GUI {
    pause: bool,
    step: bool,
    arch_state: ArchState,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

#[derive(Default)]
struct GUIState {
    mem_table_state: TableState,
    mem_scroll_pos: usize,
}

#[derive(Default, Debug)]
struct Inputs {
    exit: bool,
    step: bool,
    toggle_pause: bool,
    scroll_dir: Option<ScrollDirection>,
    cursor_location: (u16, u16),
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
        let mut gui_state = GUIState {
            // mem_scrollbar_state: ScrollbarState::default()
            //     // .content_length(self.arch_state.mem.len()),
            //     .content_length(128),
            mem_table_state: TableState::new(),
            ..Default::default()
        };
        loop {
            let inputs = if poll(Duration::from_millis(20)).is_ok_and(|has_event| has_event) {
                if let Ok(event) = read() {
                    GUI::handle_input(event)
                } else {
                    Inputs::default()
                }
            } else {
                Inputs::default()
            };
            inputs.scroll_dir.inspect(|dir| {
                if *dir == ScrollDirection::Forward {
                    gui_state.mem_scroll_pos = gui_state.mem_scroll_pos.saturating_add(1);
                } else {
                    gui_state.mem_scroll_pos = gui_state.mem_scroll_pos.saturating_sub(1);
                }
            });
            // inputs.scroll_dir.inspect(|dir| {
            //     if *dir == ScrollDirection::Backward {
            //         gui_state.mem_table_state.scroll_up_by(1)
            //     } else {
            //         gui_state.mem_table_state.scroll_down_by(1)
            //     }
            // });

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
                    &self.arch_state.mem,
                    &mut gui_state,
                    &inputs,
                );

                if cfg!(debug_assertions) {
                    frame.render_widget(Text::raw(format!("{:?}", inputs)), frame.area())
                };
            })?;

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

    fn draw(
        frame: &mut Frame,
        pc: usize,
        registers: &Vec<u32>,
        instruction: &Instruction,
        mem: &Vec<u8>,
        gui_state: &mut GUIState,
        inputs: &Inputs,
    ) {
        let columns = Layout::horizontal([Constraint::Min(6), Constraint::Min(0)]);
        let [register_area, main_area] = columns.areas(frame.area());
        let rhs_rows = Layout::vertical([Constraint::Fill(1), Constraint::Length(8)]);
        let [mem_area, control_area] = rhs_rows.areas(main_area);
        let register_area_block = Block::bordered();
        let mem_area_block = Block::bordered();
        let control_area_block = Block::bordered();
        frame.render_widget(&register_area_block, register_area);
        frame.render_widget(&mem_area_block, mem_area);
        frame.render_widget(&control_area_block, control_area);

        // Memory Readout
        gui_state.mem_scroll_pos = gui_state
            .mem_scroll_pos
            .clamp(0, mem.len() - mem_area.height as usize + 2);
        let mem_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mem_table = Table::new(
            mem[gui_state.mem_scroll_pos..gui_state.mem_scroll_pos + mem_area.height as usize - 2]
                .iter()
                .enumerate()
                .map(|(i, byte)| {
                    Row::new([
                        Cell::new(format!("{:x}", i + gui_state.mem_scroll_pos)),
                        Cell::new(format!("{:x}", byte)),
                    ])
                }),
            [Constraint::Fill(1), Constraint::Length(2)],
        )
        .row_highlight_style(Style::new().fg(Color::Black).bg(Color::Gray));

        frame.render_stateful_widget(
            mem_table,
            mem_area_block.inner(mem_area),
            &mut gui_state.mem_table_state,
        );
        frame.render_stateful_widget(
            mem_scrollbar,
            mem_area,
            &mut ScrollbarState::new(mem.len() - mem_area.height as usize)
                .position(gui_state.mem_scroll_pos),
        );

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

        let [instruction_area, ui_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(1)])
                .areas(control_area_block.inner(control_area));

        frame.render_widget(Text::raw(format!("{}", instruction)), instruction_area);
    }

    fn handle_input(event: Event) -> Inputs {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char(c) => Inputs {
                    exit: c == 'q',
                    toggle_pause: c == ' ',
                    ..Default::default()
                },
                KeyCode::Right => Inputs {
                    step: true,
                    ..Default::default()
                },
                KeyCode::Down => Inputs {
                    scroll_dir: Some(ScrollDirection::Forward),
                    ..Default::default()
                },
                KeyCode::Up => Inputs {
                    scroll_dir: Some(ScrollDirection::Backward),
                    ..Default::default()
                },
                _ => Inputs::default(),
            },
            Event::Mouse(mouse_event) => match mouse_event.kind {
                MouseEventKind::ScrollDown => Inputs {
                    scroll_dir: Some(ScrollDirection::Forward),
                    cursor_location: (mouse_event.column, mouse_event.row),
                    ..Default::default()
                },
                MouseEventKind::ScrollUp => Inputs {
                    scroll_dir: Some(ScrollDirection::Backward),
                    cursor_location: (mouse_event.column, mouse_event.row),
                    ..Default::default()
                },
                _ => Inputs {
                    cursor_location: (mouse_event.column, mouse_event.row),
                    ..Default::default()
                },
            },
            _ => Inputs::default(),
        }
    }
}
