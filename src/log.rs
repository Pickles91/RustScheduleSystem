use std::io::Stdout;

use tui::{Terminal, backend::CrosstermBackend, widgets::{List, ListItem, Block, Borders}, layout::Rect};

use crate::{process::Process, scheduler::SchedulerResult};

pub struct Log {
    pub content: Vec<TickEntry>,
    term: Terminal<CrosstermBackend<Stdout>>
}

pub struct TickEntry {
    pub cpu_process: SchedulerResult,
    pub io_process: SchedulerResult,
    pub arrived_processes: Vec<Process>,
    pub cpu_queue: Vec<Process>,
    pub io_queue: Vec<Process>,
    pub finished_processes: Vec<Process>,
    pub yet_to_arrive: Vec<Process>,
}

impl Log {
    pub fn new() -> Self {
        Self {
            content: vec![],
            term: Terminal::new(CrosstermBackend::new(std::io::stdout())).unwrap()
        }
    }
    pub fn push(&mut self, entry: TickEntry) {
        self.content.push(entry);
    }
    fn draw_frame(term: &mut Terminal<CrosstermBackend<Stdout>>, content: &[TickEntry]) {
        term.clear().unwrap();
        term.draw(|f| {
            let cpu_text = match &content.last().unwrap().cpu_process {
                SchedulerResult::Finished(p) => format!("CPU0: FINISHED {}", p.name),
                SchedulerResult::Processing(p) => format!("CPU0: PROCESSING {}", p.name),
                SchedulerResult::Idle | SchedulerResult::NoBurstLeft => format!("CPU0: IDLE"),
                _ => panic!("CPU0: ERR"),
            };
            let io_text = match &content.last().unwrap().io_process {
                SchedulerResult::Finished(p) => format!("IO0: FINISHED {}", p.name),
                SchedulerResult::Processing(p) => format!("IO0: PROCESSING {}", p.name),
                SchedulerResult::Idle | SchedulerResult::NoBurstLeft => format!("IO0: IDLE"),
                _ => format!("IO0: IDLE"),
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
                    ListItem::new(format!("TIME: {}", content.len() - 1)),
                    ListItem::new(
                        format!(
                            "CPU USAGE: {:.2}", 
                            content
                                .iter()
                                .map(|entry| if let SchedulerResult::Finished(_) | SchedulerResult::Processing(_) = entry.cpu_process { 1. } else { 0. })
                                .sum::<f64>() / content.len() as f64
                        )
                    ),
                    ListItem::new(
                        format!(
                            "IO USAGE: {:.2}",
                                content
                                .iter()
                                .map(|entry| if let SchedulerResult::Finished(_) | SchedulerResult::Processing(_) = entry.io_process { 1. } else { 0. })
                                .sum::<f64>() / content.len() as f64
                        )
                    )
                ])
                .block(
                        Block::default()
                            .title("SYSTEM STATE")
                            .borders(Borders::all())
                ), Rect::new(40, 0, 20, 5)
            );
            f.render_widget(
                List::new(
                    content.iter().map(|entry| entry.cpu_queue.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()).last().unwrap(),
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
                    content.iter().map(|entry| entry.io_queue.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()).last().unwrap(),
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
                    content.iter().map(|entry| entry.finished_processes.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()).last().unwrap(),
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
                    content.iter().map(|entry| entry.yet_to_arrive.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()).last().unwrap()
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
                    content.iter()
                        .map(|entry| 
                            entry.cpu_queue.iter()
                            .chain(entry.io_queue.iter())
                            .chain(entry.arrived_processes.iter())
                            .chain(entry.yet_to_arrive.iter())
                            .map(|process| ListItem::new(format!("{:?}", process))))
                            .last()
                            .unwrap()
                        .collect::<Vec<_>>()
                )
                .block(
                        Block::default()
                            .title("PROCESS INFO")
                            .borders(Borders::all())
                )
                , Rect::new(0, 5, 140, 5)
            );
        }).unwrap();
    }
    pub fn draw_gui(&mut self) {
        // this actually supports moving backwards too! :)
        // we just need to set i backwards. That's why I didn't
        // write it as a for loop - the `i` actually changes 
        // bidirectionally here. It's the benefit of logging
        // everything before drawing the GUI.
        let mut i = 1;
        loop {
            if i > self.content.len() {
                break;
            }
            Self::draw_frame(&mut self.term, &self.content[0..i]);
            let mut buff = String::new();
            std::io::stdin().read_line(&mut buff).unwrap();
            i += 1;
        }
    }
}