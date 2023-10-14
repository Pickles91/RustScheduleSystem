use std::io::Stdout;

use tui::{backend::CrosstermBackend, Terminal, widgets::{Paragraph, Block, Borders, List, ListItem}, layout::Rect, style::Style};

use crate::{process::Process, system_state::SystemState};

#[derive(PartialEq, Eq)]
pub enum SchedulerState {
    Processing(Process),
    Idle,
}

pub struct Gui {
    term: tui::Terminal<CrosstermBackend<Stdout>>,
    pub cpu_state: SchedulerState,
    pub io_state: SchedulerState,
    // efficiency be danmned I'm not in the mood
    // to deal with pointer lifetime shenannigans.
    // FIXME: This should probably be Vec<&'a Process>
    pub cpu_process_queue: Vec<Process>,
    pub io_process_queue: Vec<Process>,
    pub finished_processes: Vec<Process>,
    pub yet_to_arrive: Vec<Process>,
    pub all_processes: Vec<Process>,
    pub system_state: SystemState,
}
impl Gui {
    pub fn new() -> Self {
        Self {
            term: Terminal::new(CrosstermBackend::new(std::io::stdout())).unwrap(),
            cpu_state: SchedulerState::Idle,
            io_state: SchedulerState::Idle,
            cpu_process_queue: vec![],
            io_process_queue: vec![],
            finished_processes: vec![],
            yet_to_arrive: vec![],
            all_processes: vec![],
            system_state: SystemState::new(),
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
                .block(
                    Block::default()
                        .title("STATUS")
                        .borders(Borders::all())
                )
                , Rect::new(0, 0, 40, 5)
            );
            f.render_widget(
                List::new([
                    ListItem::new(format!("TIME: {}", self.system_state.time))
                ])
                .block(
                        Block::default()
                            .title("SYSTEM STATE")
                            .borders(Borders::all())
                ), Rect::new(40, 0, 20, 5)
            );
            f.render_widget(
                List::new(
                    self.cpu_process_queue.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()
                )
                .block(
                    Block::default()
                        .title("CPU QUEUE")
                        .borders(Borders::all())
                )
                , Rect::new(60, 0, 20, 5)
            );
            f.render_widget(
                List::new(
                    self.io_process_queue.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()
                )
                .block(
                    Block::default()
                        .title("IO QUEUE")
                        .borders(Borders::all())
                )
                , Rect::new(80, 0, 20, 5)
            );
            f.render_widget(
                List::new(
                    self.finished_processes.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()
                )
                .block(
                    Block::default()
                        .title("FINISHED PROCESSES")
                        .borders(Borders::all())
                )
                , Rect::new(100, 0, 20, 5)
            );
            f.render_widget(
                List::new(
                    self.yet_to_arrive.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()
                )
                .block(
                    Block::default()
                        .title("FUTURE PROCESSES")
                        .borders(Borders::all())
                )
                , Rect::new(120, 0, 20, 5)
            );
            f.render_widget(
                List::new(
                    self.all_processes.iter().map(|process| ListItem::new(format!("{:?}", process))).collect::<Vec<_>>()
                )
                .block(
                        Block::default()
                            .title("PROCESS INFO")
                            .borders(Borders::all())
                )
                , Rect::new(0, 5, 150, 5)
            );

        }).unwrap();
        let mut buff = String::new();
        std::io::stdin().read_line(&mut buff).unwrap();
    }
}
