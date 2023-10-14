use std::io::Stdout;

use tui::{backend::CrosstermBackend, Terminal, widgets::{Paragraph, Block, Borders, List, ListItem}, layout::Rect, style::Style};

use crate::process::Process;

#[derive(PartialEq, Eq)]
pub enum SchedulerState {
    Processing(Process),
    Idle,
}

pub struct Gui {
    term: tui::Terminal<CrosstermBackend<Stdout>>,
    pub cpu_state: SchedulerState,
    pub io_state: SchedulerState,
}
impl Gui {
    pub fn new() -> Self {
        Self {
            term: Terminal::new(CrosstermBackend::new(std::io::stdout())).unwrap(),
            cpu_state: SchedulerState::Idle,
            io_state: SchedulerState::Idle,
        }
    }
    pub fn draw(&mut self) {
        self.term.clear().unwrap();
        self.term.draw(|f| {
            let cpu_text = match &self.cpu_state {
                SchedulerState::Processing(p ) => {
                    format!("CPU0: PROCESSING {}", p.name)
                }
                SchedulerState::Idle => {
                    format!("CPU0: IDLE")
                }
            };
            let io_text = match &self.io_state {
                SchedulerState::Processing(p ) => {
                    format!("IO0: PROCESSING {}", p.name)
                }
                SchedulerState::Idle => {
                    format!("IO0: IDLE")
                }
            };
            f.render_widget(
                List::new([
                    ListItem::new(cpu_text),
                    ListItem::new(io_text),
                ])
                .block(Block::default()
                    .title("STATUS")
                    .borders(Borders::all())
                )
                , Rect::new(0, 0, 35, 4)
            );
        }).unwrap();
        let mut buff = String::new();
        std::io::stdin().read_line(&mut buff).unwrap();
    }
}
