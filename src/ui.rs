use std::{
    fmt::Display, io, sync::{LazyLock, mpsc}, time::Duration
};

use anyhow::anyhow;
use crossterm::{
    event::{self, DisableMouseCapture, KeyCode},
    execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use tui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{
    instruction::{Data16, Register, RegisterPair},
    machine::Machine,
    ui::memory_view::MemoryView,
};

mod memory_view;

fn parse_hex(hex: &str) -> anyhow::Result<Color> {
    let digits = hex
        .strip_prefix("#")
        .ok_or_else(|| anyhow!("Couldn't parse '{}': Missing '#'", hex))?;
    if digits.len() != 6 {
        return Err(anyhow!(
            "Couldn't parse '{}': Incorrect length {}",
            hex,
            digits.len()
        ));
    }

    let red: u8 = u8::from_str_radix(&digits[0..2], 16)
        .map_err(|err| anyhow!("Couldn't parse '{}': Invalid color component: {}", hex, err))?;
    let green: u8 = u8::from_str_radix(&digits[2..4], 16)
        .map_err(|err| anyhow!("Couldn't parse '{}': Invalid color component: {}", hex, err))?;
    let blue: u8 = u8::from_str_radix(&digits[4..6], 16)
        .map_err(|err| anyhow!("Couldn't parse '{}': Invalid color component: {}", hex, err))?;

    Ok(Color::Rgb(red, green, blue))
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RegisterSingleOrPair {
    Single(Register),
    Pair(RegisterPair),
}

impl Display for RegisterSingleOrPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterSingleOrPair::Single(register) => register.fmt(f),
            RegisterSingleOrPair::Pair(register_pair) => register_pair.fmt(f),
        }
    }
}

#[allow(unused)]
static COLOR_TEXT: LazyLock<Color> = LazyLock::new(|| parse_hex("#cdd6f4").unwrap());
#[allow(unused)]
static COLOR_SUBTEXT_1: LazyLock<Color> = LazyLock::new(|| parse_hex("#bac2de").unwrap());
#[allow(unused)]
static COLOR_SUBTEXT_0: LazyLock<Color> = LazyLock::new(|| parse_hex("#a6adc8").unwrap());
#[allow(unused)]
static COLOR_OVERLAY_2: LazyLock<Color> = LazyLock::new(|| parse_hex("#6c7086").unwrap());
#[allow(unused)]
static COLOR_GREEN: LazyLock<Color> = LazyLock::new(|| parse_hex("#A7DFA2").unwrap());
#[allow(unused)]
static COLOR_PEACH: LazyLock<Color> = LazyLock::new(|| parse_hex("#fab387").unwrap());
#[allow(unused)]
static COLOR_RED: LazyLock<Color> = LazyLock::new(|| parse_hex("#f38ba8").unwrap());
#[allow(unused)]
static COLOR_LAVENDER: LazyLock<Color> = LazyLock::new(|| parse_hex("#b4befe").unwrap());

static STYLE_BLOCK_BORDER: LazyLock<Style> =
    LazyLock::new(|| Style::default().fg(*COLOR_OVERLAY_2));
static STYLE_BLOCK_LABEL: LazyLock<Style> = LazyLock::new(|| {
    Style::default()
        .fg(*COLOR_OVERLAY_2)
        .add_modifier(Modifier::BOLD)
});
static STYLE_ADDRESS: LazyLock<Style> = LazyLock::new(|| Style::default().fg(*COLOR_GREEN));
static STYLE_LABEL: LazyLock<Style> = LazyLock::new(|| {
    Style::default()
        .fg(*COLOR_TEXT)
        .add_modifier(Modifier::empty())
});
static STYLE_VALUE: LazyLock<Style> = LazyLock::new(|| Style::default().fg(*COLOR_PEACH));
static STYLE_DATA: LazyLock<Style> = LazyLock::new(|| Style::default().fg(*COLOR_SUBTEXT_0));

struct Ui {
    machine: Machine,
    quit_sender: mpsc::Sender<()>,
}

impl Ui {
    fn new(machine: Machine, quit_sender: mpsc::Sender<()>) -> Self {
        Self {
            machine,
            quit_sender,
        }
    }

    fn tick(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> anyhow::Result<()> {
        self.draw(terminal)
    }

    fn draw(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
        terminal.draw(|f| {
            static REGISTERS_HEIGHT: u16 = 5 + 2;
            let registers_instructions_area_height = Constraint::Ratio(2, 5)
                .apply(f.size().height)
                .max(REGISTERS_HEIGHT);

            let mut memory_area = f.size();
            memory_area.height -= registers_instructions_area_height;

            let mut registers_instructions_area = f.size();
            registers_instructions_area.height = registers_instructions_area_height;
            registers_instructions_area.y = memory_area.bottom();

            let [registers_area, instructions_area]: [Rect; 2] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(28 + 2), Constraint::Ratio(1, 1)].as_ref())
                .split(registers_instructions_area)
                .try_into()
                .expect("We created two areas");

            self.draw_memory(f, memory_area);

            self.draw_registers(f, registers_area);
            self.draw_instructions(f, instructions_area);
        })?;
        Ok(())
    }

    fn draw_memory(&self, f: &mut Frame<'_, CrosstermBackend<io::Stdout>>, area: Rect) {
        let block = Block::default()
            .title(Span::styled("Memory", *STYLE_BLOCK_LABEL))
            .borders(Borders::all())
            .border_type(BorderType::Rounded)
            .border_style(*STYLE_BLOCK_BORDER);
        let widget_area = block.inner(area).inner(&Margin {
            vertical: 0,
            horizontal: 1,
        });
        f.render_widget(block, area);

        let memory_view = MemoryView::new(self.machine.memory().as_raw())
            .shown_address(Data16::from(0))
            .label_style(*STYLE_LABEL)
            .address_style(*STYLE_ADDRESS)
            .data_style(*STYLE_DATA);

        f.render_widget(memory_view, widget_area);
    }

    fn draw_registers(&self, f: &mut Frame<'_, CrosstermBackend<io::Stdout>>, area: Rect) {
        let block = Block::default()
            .title(Span::styled("Registers", *STYLE_BLOCK_LABEL))
            .borders(Borders::all())
            .border_type(BorderType::Rounded)
            .border_style(*STYLE_BLOCK_BORDER);
        let list_area = block.inner(area).inner(&Margin {
            vertical: 0,
            horizontal: 1,
        });
        f.render_widget(block, area);

        let column_areas: [Rect; 3] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(7 + 1),
                    Constraint::Length(7 + 1),
                    Constraint::Length(10 + 1),
                ]
                .as_ref(),
            )
            .split(list_area)
            .try_into()
            .expect("We created 3 areas");

        const ROWS: usize = 5;
        let register_grid: [[Option<RegisterSingleOrPair>; ROWS]; 3] = [
            [
                Some(RegisterSingleOrPair::Single(Register::B(()))),
                Some(RegisterSingleOrPair::Single(Register::D(()))),
                Some(RegisterSingleOrPair::Single(Register::H(()))),
                Some(RegisterSingleOrPair::Single(Register::M(()))),
                Some(RegisterSingleOrPair::Single(Register::A(()))),
            ],
            [
                Some(RegisterSingleOrPair::Single(Register::C(()))),
                Some(RegisterSingleOrPair::Single(Register::E(()))),
                Some(RegisterSingleOrPair::Single(Register::L(()))),
                None,
                None,
            ],
            [
                Some(RegisterSingleOrPair::Pair(RegisterPair::Bc(()))),
                Some(RegisterSingleOrPair::Pair(RegisterPair::De(()))),
                Some(RegisterSingleOrPair::Pair(RegisterPair::Hl(()))),
                Some(RegisterSingleOrPair::Pair(RegisterPair::Sp(()))),
                None,
            ],
        ];

        for (col_index, rows) in register_grid.into_iter().enumerate() {
            let areas: [Rect; ROWS] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1); ROWS].as_ref())
                .split(column_areas[col_index])
                .try_into()
                .expect("We created LENGTH areas");

            for (row_index, register) in rows.into_iter().enumerate() {
                let Some(register) = register else {
                    continue;
                };
                let value_string = match register {
                    RegisterSingleOrPair::Single(register) => {
                        let value = self
                            .machine
                            .registers()
                            .get_8(register, self.machine.memory());
                        format!("0x{:02x}", value)
                    }
                    RegisterSingleOrPair::Pair(register) => {
                        let value = self.machine.registers().get_16(register);
                        format!("0x{:04x}", value.value())
                    }
                };
                let par = Paragraph::new(vec![Spans::from(vec![
                    Span::styled(format!("{}", register), *STYLE_LABEL),
                    Span::raw(": "),
                    Span::styled(value_string, *STYLE_VALUE),
                ])]);

                f.render_widget(par, areas[row_index]);
            }
        }
    }

    fn draw_instructions(&self, f: &mut Frame<'_, CrosstermBackend<io::Stdout>>, area: Rect) {
        let block = Block::default()
            .title(Span::styled("Instructions", *STYLE_BLOCK_LABEL))
            .borders(Borders::all())
            .border_type(BorderType::Rounded)
            .border_style(*STYLE_BLOCK_BORDER);
        let block_area = block.inner(area).inner(&Margin {
            vertical: 0,
            horizontal: 1,
        });
        f.render_widget(block, area);

        {
            let value = self.machine.pc();
            let pc = Paragraph::new(Spans::from(vec![
                Span::styled("PC", *STYLE_LABEL),
                Span::raw(": "),
                Span::styled(format!("0x{:04x}", value.value()), *STYLE_VALUE),
            ]));
            f.render_widget(pc, block_area);
        }

        let mut instructions_area = block_area;
        instructions_area.y -= 1;
        instructions_area.height -= 1;
    }

    fn input(&mut self, event: event::KeyEvent) -> anyhow::Result<()> {
        match event.code {
            KeyCode::Char('q') => {
                self.quit_sender.send(())?;
            }
            _ => {}
        }
        Ok(())
    }
}

pub fn start(machine: Machine) -> anyhow::Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    enable_raw_mode()?;

    let (quit_sender, quit_receiver) = mpsc::channel::<()>();
    let mut ui = Ui::new(machine, quit_sender.clone());

    std::thread::spawn(move || -> Result<(), anyhow::Error> {
        loop {
            ui.tick(&mut terminal)?;
            if event::poll(Duration::from_millis(100))? {
                if let event::Event::Key(key_event) = event::read()? {
                    if key_event.code == event::KeyCode::Char('c')
                        && key_event
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                    {
                        // signal by settting our AtomicBool to false
                        quit_sender.send(())?;
                    } else {
                        ui.input(key_event)?;
                    }
                }
            }
        }
    });

    if quit_receiver.iter().next().is_none() {
        return Err(anyhow!("UI channel broken"));
    };

    // restore terminal
    let mut backend = CrosstermBackend::new(io::stdout());
    disable_raw_mode()?;
    execute!(&mut backend, LeaveAlternateScreen, DisableMouseCapture,)?;
    backend.show_cursor()?;

    Ok(())
}
