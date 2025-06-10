use std::{error::Error, io::Stdout, time::Duration};

use ratatui::{
    Frame, Terminal,
    crossterm::{
        event::{
            DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind, poll, read,
        },
        execute,
    },
    layout::{Constraint, Layout, Margin, Position, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{
        Block, Cell, Row, ScrollDirection, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};

use crate::vm::{self, ArchState, Instruction};

pub struct GUI {
    pause: bool,
    step: bool,
    arch_state: ArchState,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

#[derive(Default, Debug)]
struct GUIState {
    mem_table_state: TableState,
    mem_scroll_pos: usize,
    reg_table_state: TableState,
    reg_scroll_pos: usize,
    last_mouse_pos: Position,
}

#[derive(Default, Debug)]
struct Inputs {
    exit: bool,
    step: bool,
    toggle_pause: bool,
    scroll_dir: Option<ScrollDirection>,
    mouse_loc: Option<(u16, u16)>,
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
        execute!(std::io::stdout(), EnableMouseCapture)?;
        let mut gui_state = GUIState {
            mem_table_state: TableState::new(),
            ..Default::default()
        };
        loop {
            self.terminal.autoresize()?;
            let mut log_event = None;
            let inputs = if poll(Duration::from_millis(100)).is_ok_and(|has_event| has_event) {
                if let Ok(event) = read() {
                    log_event = Some(event.clone());
                    GUI::handle_input(event)
                } else {
                    Inputs::default()
                }
            } else {
                Inputs::default()
            };

            inputs
                .mouse_loc
                .inspect(|(x, y)| gui_state.last_mouse_pos = Position::new(*x, *y));

            self.terminal.draw(|frame| {
                GUI::draw(
                    frame,
                    self.pause,
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
                    frame.render_widget(
                        Text::raw(format!("{:?} {:?}", inputs, log_event)),
                        frame.area(),
                    )
                };
            })?;

            if inputs.exit {
                break;
            }

            self.step = inputs.step;
            self.pause = self.pause != inputs.toggle_pause;

            if self.step || !self.pause {
                self.arch_state.tick();
                self.step = false;
            }
        }
        execute!(std::io::stdout(), DisableMouseCapture)?;
        Ok(())
    }

    fn draw(
        frame: &mut Frame,
        paused: bool,
        pc: usize,
        registers: &Vec<u32>,
        instruction: &Instruction,
        mem: &Vec<u8>,
        gui_state: &mut GUIState,
        inputs: &Inputs,
    ) {
        let columns = Layout::horizontal([Constraint::Fill(1), Constraint::Min(3 * 16 + 8 + 4)]);
        let [register_area, main_area] = columns.areas(frame.area());
        let rhs_rows = Layout::vertical([Constraint::Fill(1), Constraint::Length(8)]);
        let [mem_area, control_area] = rhs_rows.areas(main_area);
        let register_area_block = Block::bordered();
        let mem_area_block = Block::bordered();
        let control_area_block = Block::bordered();
        frame.render_widget(&register_area_block, register_area);
        frame.render_widget(&mem_area_block, mem_area);
        frame.render_widget(&control_area_block, control_area);

        inputs.scroll_dir.inspect(|dir| {
            let scroll_motion = if *dir == ScrollDirection::Forward {
                1
            } else {
                -1
            };
            if mem_area.contains(gui_state.last_mouse_pos) {
                gui_state.mem_scroll_pos = gui_state
                    .mem_scroll_pos
                    .saturating_add_signed(scroll_motion);
            }
            if register_area.contains(gui_state.last_mouse_pos) {
                gui_state.reg_scroll_pos = gui_state
                    .reg_scroll_pos
                    .saturating_add_signed(scroll_motion);
            }
        });
        *gui_state.reg_table_state.offset_mut() = gui_state.reg_scroll_pos;

        // Memory readout
        gui_state.mem_scroll_pos = gui_state
            .mem_scroll_pos
            .clamp(0, mem.len() - mem_area.height as usize + 2);
        let mem_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mem_table_even_style: Style = Style::new();
        let mem_table_odd_style: Style = Style::new().underlined();

        let mem_table = Table::new(
            (0..mem_area.height as usize - 2).map(|i| {
                let start_addr = (gui_state.mem_scroll_pos + i) * 16;
                let mut cols = vec![Cell::new(format!("{:08x}", start_addr))];
                for offset in 0..16 {
                    cols.push(Cell::new(format!("{:02x}|", mem[start_addr + offset])));
                }
                Row::new(cols).style(if i % 2 == 0 {
                    mem_table_even_style
                } else {
                    mem_table_odd_style
                })
            }),
            [
                vec![Constraint::Min(10)],
                vec![Constraint::Length(3); 16],
                vec![Constraint::Length(1)],
            ]
            .concat(),
        )
        .header(
            Row::new(
                [
                    vec![Cell::new("--------")],
                    (0..16)
                        .map(|i| Cell::new(format!("{:02x}", i)))
                        .collect::<Vec<Cell>>(),
                ]
                .concat(),
            )
            .reversed()
            .not_underlined(),
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

        // pc & reg readouts
        let [pc_area, reg_table_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)])
                .areas(register_area_block.inner(register_area));

        gui_state.reg_scroll_pos = gui_state
            .reg_scroll_pos
            .clamp(0, 32_usize.saturating_sub(reg_table_area.height as usize));

        frame.render_widget(
            Text::raw(format!("pc : 0x{0:0>8X} | {0:0>10}", pc)),
            pc_area,
        );

        let reg_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);

        let reg_table = Table::new(
            (0..32)
                .map(|i| {
                    Row::new([Cell::new(format!(
                        "x{: <2}: 0x{1:0>8X} | {1:0>10}",
                        i,
                        registers.get(i).unwrap()
                    ))])
                })
                .collect::<Vec<Row>>(),
            [Constraint::Fill(1)],
        );

        frame.render_stateful_widget(reg_table, reg_table_area, &mut gui_state.reg_table_state);
        frame.render_stateful_widget(
            reg_scrollbar,
            register_area,
            &mut ScrollbarState::new(32_usize.saturating_sub(reg_table_area.height as usize))
                .position(gui_state.reg_scroll_pos),
        );

        let [instruction_area, ui_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(1)])
                .areas(control_area_block.inner(control_area));

        frame.render_widget(Text::raw(format!("{}", instruction)), instruction_area);
        frame.render_widget(
            Text::raw(format!("\n{}", if paused { "||" } else { ">>" })),
            ui_area,
        );
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
                    ..Default::default()
                },
                MouseEventKind::ScrollUp => Inputs {
                    scroll_dir: Some(ScrollDirection::Backward),
                    ..Default::default()
                },
                MouseEventKind::Moved => Inputs {
                    mouse_loc: Some((mouse_event.column, mouse_event.row)),
                    ..Default::default()
                },
                _ => Inputs {
                    ..Default::default()
                },
            },
            _ => Inputs::default(),
        }
    }
}
