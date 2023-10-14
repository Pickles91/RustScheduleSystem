use std::io::Stdout;

use tui::{Terminal, backend::CrosstermBackend, widgets::{List, ListItem, Block, Borders}, layout::Rect};

use crate::process::Process;

pub struct Log {
    pub content: Vec<TickEntry>,
    term: Terminal<CrosstermBackend<Stdout>>
}

pub struct TickEntry {
    pub cpu_process: Option<Process>,
    pub io_process: Option<Process>,
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
    pub fn draw_gui(&mut self) {
        self.term.clear().unwrap();
        self.term.draw(|f| {
            let cpu_text = match &self.content.last().unwrap().cpu_process {
                Some(v) => format!("CPU0: PROCESSING {}", v.name),
                None => format!("CPU0: IDLE"),
            };
            let io_text = match &self.content.last().unwrap().io_process {
                Some(v) => format!("IO0: PROCESSING {}", v.name),
                None => format!("IO0: IDLE"),
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
                    ListItem::new(format!("TIME: {}", self.content.len() - 1)),
                    ListItem::new(format!("CPU USAGE: {:.2}", self.content.iter().map(|entry| if entry.cpu_process.is_some() { 1. } else { 0. }).sum::<f64>() / self.content.len() as f64)),
                    ListItem::new(format!("IO USAGE: {:.2}", self.content.iter().map(|entry| if entry.io_process.is_some() { 1. } else { 0. }).sum::<f64>()  / self.content.len() as f64)),
                ])
                .block(
                        Block::default()
                            .title("SYSTEM STATE")
                            .borders(Borders::all())
                ), Rect::new(40, 0, 20, 5)
            );
            f.render_widget(
                List::new(
                    self.content.iter().map(|entry| entry.cpu_queue.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()).last().unwrap(),
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
                    self.content.iter().map(|entry| entry.io_queue.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()).last().unwrap(),
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
                    self.content.iter().map(|entry| entry.finished_processes.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()).last().unwrap(),
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
                    self.content.iter().map(|entry| entry.yet_to_arrive.iter().map(|process| ListItem::new(process.name.clone())).collect::<Vec<_>>()).last().unwrap()
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
                    self.content.iter()
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
        let mut buff = String::new();
        std::io::stdin().read_line(&mut buff).unwrap();
    }
}