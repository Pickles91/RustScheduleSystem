use std::fs::File;
use std::{io::Stdout, collections::HashSet};
use std::io::Write;

use crossterm::event::{self, Event, KeyEventKind};
use tui::layout::{Direction, Constraint, Layout};
use tui::widgets::Paragraph;
use tui::{Terminal, backend::CrosstermBackend, widgets::{List, ListItem, Block, Borders}, layout::Rect};

use crate::{process::Process, scheduler::SchedulerResult};

pub struct Log {
    pub content: Vec<TickEntry>,
    term: Terminal<CrosstermBackend<Stdout>>
}

pub struct TickEntry {
    pub cpu_process: SchedulerResult,
    pub io_process: SchedulerResult,
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

    fn all_processes(content: &TickEntry) -> Vec<Process> {
        content
            .cpu_queue
            .iter()
            .chain(content.io_queue.iter())
            .chain(content.finished_processes.iter())
            .chain(content.yet_to_arrive.iter())
            .cloned()
            .collect()
    }

    // includes CPU and IO bursts.
    fn total_compute_time(pid: i32, content: &[TickEntry]) -> Option<i32> {
        Some(Self::all_processes(&content[0]).iter().find(|proc| proc.pid == pid)?.burst.iter().map(|burst| burst.1).sum())
    }

    fn finished_time(pid: i32, content: &[TickEntry]) ->  Option<i32> {
        Some(content.iter().enumerate().find(|(_time, entry)| entry.finished_processes.iter().any(|proc| proc.pid == pid))?.0 as i32)
    }

    fn arrival_time(pid: i32, content: &[TickEntry]) -> Option<i32> {
        Some(Self::all_processes(&content[0]).iter().find(|proc| proc.pid == pid)?.arrival)
    }

    fn wait_time(pid: i32, content: &[TickEntry]) -> Option<i32> {
        Some(Self::finished_time(pid, content)? - Self::total_compute_time(pid, content)? -  Self::arrival_time(pid, content)?)
    }

    fn turn_around_time(pid: i32, content: &[TickEntry]) -> Option<i32> {
        Some(Self::finished_time(pid, content)?  -  Self::arrival_time(pid, content)?)
    }


    fn throughput(content: &[TickEntry]) -> f64 {
        content.last().unwrap().finished_processes.len() as f64 / content.len() as f64
    }

    fn avg_wait_time(content: &[TickEntry]) -> f64 {
        let (count, sum) = content
            .last()
            .unwrap()
            .finished_processes
            .iter()
            .map(|proc| Self::wait_time(proc.pid, content).unwrap())
            .enumerate()
            .fold((0, 0), |(_, wait_time), (count, next_wait_time)| (count, wait_time + next_wait_time));

        sum as f64 / (count as f64 + 1.)

    }

    fn avg_turnaround_time(content: &[TickEntry]) -> f64 {
        let (count, sum) = content
            .last()
            .unwrap()
            .finished_processes
            .iter()
            .map(|proc| Self::turn_around_time(proc.pid, content).unwrap())
            .enumerate()
            .fold((0, 0), |(_, wait_time), (count, next_wait_time)| (count, wait_time + next_wait_time));

        sum as f64 / (count as f64 + 1.)
    }

    fn get_cpu_arrivals(content: &[TickEntry]) -> Vec<Process> {
        // all the processes in the first entry are logically newly arrived.
        if content.len() == 1 {
            return content.first().unwrap().cpu_queue.clone();
        }
        let current_running_pids = content
            .last()
            .unwrap()
            .cpu_queue
            .iter()
            .map(|proc| proc.pid)
            .collect::<HashSet<_>>();
        let last_running_pids = 
                &content.get(content.len() - 2)
                    .unwrap()
                    .cpu_queue
                    .iter()
                    .map(|proc| proc.pid)
                    .collect::<HashSet<_>>();
        let new_pids = current_running_pids
            .difference(
                last_running_pids
            )
            .collect::<HashSet<_>>();

        Self::all_processes(&content[0])
            .into_iter()
            .filter(|proc| new_pids.contains(&proc.pid))
            .collect()
    }
    fn get_io_arrivals(content: &[TickEntry]) -> Vec<Process> {
        // all the processes in the first entry are logically newly arrived.
        if content.len() == 1 {
            return content.first().unwrap().io_queue.clone();
        }
        let current_running_pids = content
            .last()
            .unwrap()
            .io_queue
            .iter()
            .map(|proc| proc.pid)
            .collect::<HashSet<_>>();
        let last_running_pids = 
                &content.get(content.len() - 2)
                    .unwrap()
                    .io_queue
                    .iter()
                    .map(|proc| proc.pid)
                    .collect::<HashSet<_>>();
        let new_pids = current_running_pids
            .difference(
                last_running_pids
            )
            .collect::<HashSet<_>>();

        Self::all_processes(&content[0])
            .into_iter()
            .filter(|proc| new_pids.contains(&proc.pid))
            .collect()
    }

    fn get_scheduler_process(result: &SchedulerResult) -> Option<Process> {
        match result {
            SchedulerResult::Finished(p) | SchedulerResult::Processing(p) => Some(p.clone()),
            _ => None
        }
    }

    fn get_log_content(content: &[TickEntry]) -> Vec<String> {
        let mut log_contents = vec![];
        for i in 0..content.len() {
            match Self::get_scheduler_process(&content[i].cpu_process) {
                // if time = 1 then the CPU must be being used.
                Some(v) if i == 0 => log_contents.push(format!("T{}: NEW PROCESS IS USING CPU: {}", i, v.name)),
                // if we have a process in the previous tick which has a different PID from the current process
                // we must be a new process
                Some(v) => if let Some(v2) = Self::get_scheduler_process(&content[i - 1].cpu_process) {
                        if v2.pid != v.pid { log_contents.push(format!("T{}: NEW PROCESS IS USING CPU: {}", i, v.name))}
                },
                // if we have no process now, and had a process in the previous tick, we just started idling.
                None => {
                    if i == 0 || Self::get_scheduler_process(&content[i - 1].cpu_process).is_some() {
                        log_contents.push(format!("T{}: CPU IS NOW IDLE", i))
                    }
                }
            };

            // same shctick as above, but with io instead.
            // note: yeah I duplicate code here, and it could probably be abstracted out.
            // FIXME: deduplicate this code so IO / CPU share it.
            match Self::get_scheduler_process(&content[i].io_process) {
                Some(v) if i == 0 => log_contents.push(format!("T{}: NEW PROCESS IS USING IO: {}", i, v.name)),
                Some(v) => if let Some(v2) = Self::get_scheduler_process(&content[i - 1].io_process) {
                        if v2.pid != v.pid {
                            log_contents.push(format!("T{}: NEW PROCESS IS USING IO: {}", i, v.name));
                        }
                    } else {
                        log_contents.push(format!("T{}: NEW PROCESS IS USING IO: {}", i, v.name));
                    },
                None => {
                    if i == 0 || Self::get_scheduler_process(&content[i - 1].io_process).is_some() {
                        log_contents.push(format!("T{}: IO IS NOW IDLE", i))
                    }
                }
            };

            let cpu_arrivals = Self::get_cpu_arrivals(&content[..i + 1]);
            if !cpu_arrivals.is_empty() {
                log_contents.push(format!("T{}: PROCESSES ARRIVED IN READY QUEUE: [{}]", i, cpu_arrivals.into_iter().map(|proc| proc.name).collect::<Vec<_>>().join(",")));
            }
            let io_arrivals = Self::get_io_arrivals(&content[..i + 1]);
            if !io_arrivals.is_empty() {
                log_contents.push(format!("T{}: PROCESSES ARRIVED IN IO QUEUE: [{}]", i, io_arrivals.into_iter().map(|proc| proc.name).collect::<Vec<_>>().join(",")));
            }

            if i != 0 {
                let new_finished = content[i]
                    .finished_processes
                    .iter()
                    .map(|proc| proc.pid)
                    .filter(|&pid| content[i - 1].finished_processes.iter().all(|proc2| proc2.pid != pid))
                    .collect::<HashSet<_>>();
                let new_finished = Self::all_processes(&content[0]).into_iter().filter(|proc| new_finished.contains(&proc.pid));

                for p in new_finished {
                    log_contents
                        .push(format!(
                            "T{}: FINISHED {} with TURNAROUND {} and WAIT {}",
                            i,
                            p.name,
                            Self::turn_around_time(p.pid, content).unwrap(),
                            Self::wait_time(p.pid, content).unwrap()
                        ));
                }
            }
        }
        log_contents
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
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ])
                .split(f.size());
            let first_row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Ratio(1, 6),
                    Constraint::Ratio(1, 6),
                    Constraint::Ratio(1, 6),
                    Constraint::Ratio(1, 6),
                    Constraint::Ratio(1, 6),
                    Constraint::Ratio(1, 6),
                ]).split(main_layout[0]);
            let last_row = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(2),
                    Constraint::Length(3)
                ])
                .split(main_layout[2]);
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
                , first_row[0]
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
                    ),
                    ListItem::new(
                        format!(
                            "AVG WAIT: {:.2}",
                            Self::avg_wait_time(content)
                        )
                    ),
                    ListItem::new(
                        format!(
                            "AVG TURNARND: {:.2}",
                            Self::avg_turnaround_time(content)
                        )
                    ),
                    ListItem::new(
                        format!(
                            "THROUGHPUT: {:.2}",
                            Self::throughput(content)
                        )
                    ),
                ])
                .block(
                        Block::default()
                            .title("SYSTEM STATE")
                            .borders(Borders::all())
                ), first_row[1]
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
                , first_row[2]
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
                , first_row[3]
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
                , first_row[4]
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
                , first_row[5]
            );
            f.render_widget(
                List::new(
                    content.iter()
                        .map(|entry| 
                            Self::all_processes(entry)
                                .into_iter()
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
                , main_layout[1]
            );
            f.render_widget(
                List::new(
                    Self::get_log_content(content)
                        .into_iter()
                        .rev()
                        .map(|process| ListItem::new(format!("{:?}", process)))
                        .collect::<Vec<_>>()
                )
                .block(
                        Block::default()
                            .title("LOG")
                            .borders(Borders::all())
                )
                , last_row[0]
            );
            f.render_widget(
                Paragraph::new("Press left and write arrow keys to progress / step back in time. Press q to exit.")
                    .block(
                        Block::default()
                            .title("Instructions")
                            .borders(Borders::all())
                    )
                ,  last_row[1]
            )
        }).unwrap();
    }
    pub fn write_file(&self, f: &mut File) {
        f.write_all(Self::get_log_content(&self.content).join("\n").as_bytes()).unwrap();
        f.sync_all().unwrap();
    }
    pub fn draw_gui(&mut self) {
        crossterm::terminal::enable_raw_mode().unwrap();
        // this actually supports moving backwards too! :)
        // we just need to set i backwards. That's why I didn't
        // write it as a for loop - the `i` actually changes 
        // bidirectionally here. It's the benefit of logging
        // everything before drawing the GUI.
        let mut i = 1;
        loop {
            Self::draw_frame(&mut self.term, &self.content[0..i]);
            if let Event::Key(k) = event::read().unwrap() {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                match k.code {
                    event::KeyCode::Left => i -= 1,
                    event::KeyCode::Right => i += 1,
                    event::KeyCode::Enter => i += 1,
                    event::KeyCode::Char('q') => break,
                    _ => { continue }
                }
            }
            if i <= 1 {
                i = 1;
            }
            if i >= self.content.len() {
                i = self.content.len();
            }
        }
        crossterm::terminal::disable_raw_mode().unwrap();
    }
}