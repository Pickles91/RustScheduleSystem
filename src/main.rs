use std::collections::VecDeque;

use process::{Burst, BurstKind, Process};
use scheduler::{Scheduler, fcfs::FCFS};
use system_state::SystemState;
use gui::Gui;

use crate::gui::SchedulerState;

mod process;
mod system_state;
mod scheduler;
mod gui;

fn main() {
    let mut args = std::env::args();
    let _ = args.next().unwrap();
    let file = match args.next() {
        Some(v) => v,
        None => panic!("Please pass in a file name"),
    };
    let content = std::fs::read_to_string(&file).unwrap();
    let mut processes: Vec<Process> = content
        .split("\n")
        .enumerate()
        .map(|(pid, line)| {
            let mut process_info = line.split(" ");
            let name = process_info.next().unwrap();
            let arrival_time = process_info.next().unwrap().parse().unwrap();
            let priority = process_info.next().unwrap().parse().unwrap();
            let mut next = BurstKind::Cpu;
            let mut bursts = vec![];
            while let Some(v) = process_info.next() {
                bursts.push(Burst(next, v.parse().unwrap()));
                next = match next {
                    BurstKind::Cpu => BurstKind::Io,
                    BurstKind::Io => BurstKind::Cpu,
                };
            }
            dbg!(Process::new(name.to_owned(), pid.try_into().unwrap(), priority, bursts, arrival_time))
        })
        .collect();
    // sort them to be sorted by arrival time, since we only want to add them to the scheduler once they're in.
    processes.sort_unstable_by(|proc, other_proc| proc.arrival.cmp(&other_proc.arrival));

    // this is somewhat bad design, both CPU and IO schedulers share a type (willfully, it lets me reuse code)
    // but instead of storing the BurstKind as a field, it probably would of been better to make a type like
    // BurstKindCpu<FCFS> and BurstKindIo<FCFS>. Oh well. That would of had it's own complexities.
    // ...I can just do a runtime check to validate them but that's not hip and cool.
    start_sim(processes.into_iter().collect(), FCFS::new(vec![], BurstKind::Cpu), FCFS::new(vec![], BurstKind::Io));
}


fn start_sim(mut processes: VecDeque<Process>, mut cpu_sched: impl Scheduler, mut io_sched: impl Scheduler) {
    let mut state = SystemState::new();

    // when this hits zero we're done.
    let mut remaining_processes = processes.len();

    let mut gui = gui::Gui::new();
    loop {
        match processes.front() {
            Some(proc) if proc.arrival <= state.time => {
                cpu_sched.enqueue(processes.pop_front().unwrap());
                continue;
            }
            _ => {},
        }

        let mut cpu_queue = vec![];
        let mut io_queue = vec![];

        // duplicated code (with subtle differences that makes abstracting this annoying)
        // alert.
        match cpu_sched.tick(&state) {
            scheduler::SchedulerResult::Finished(p) if p.burst.len() == 0 =>  {
                gui.cpu_state = SchedulerState::Processing(p);
                remaining_processes -= 1;
            },
            scheduler::SchedulerResult::Finished(p) => {
                gui.cpu_state = SchedulerState::Processing(p.clone());
                match p.burst[0].0 {
                    BurstKind::Cpu => cpu_queue.push(p),
                    BurstKind::Io => io_queue.push(p),
                }
            }
            scheduler::SchedulerResult::Processing(p) => {
                gui.cpu_state = SchedulerState::Processing(p.clone());
            },
            scheduler::SchedulerResult::Idle => gui.cpu_state = SchedulerState::Idle,
            scheduler::SchedulerResult::WrongKind => panic!("schedule for IO instead you idiot."),
            scheduler::SchedulerResult::NoBurstLeft => gui.cpu_state = SchedulerState::Idle,
        };
        match io_sched.tick(&state) {
            scheduler::SchedulerResult::Finished(p) if p.burst.len() == 0 =>  {
                remaining_processes -= 1;
                gui.io_state = SchedulerState::Processing(p.clone());
            },
            scheduler::SchedulerResult::Finished(p) => {
                gui.io_state = SchedulerState::Processing(p.clone());
                println!("process on IO finished burst {}", p.name);
                match p.burst[0].0 {
                    BurstKind::Cpu => cpu_queue.push(p),
                    BurstKind::Io => io_queue.push(p),
                }
            }
            scheduler::SchedulerResult::Processing(p) => gui.io_state = SchedulerState::Processing(p.clone()),
            scheduler::SchedulerResult::Idle => gui.io_state = SchedulerState::Idle,
            scheduler::SchedulerResult::WrongKind => panic!("schedule for IO instead you idiot."),
            scheduler::SchedulerResult::NoBurstLeft => gui.io_state = SchedulerState::Idle,
        };
        for i in cpu_queue { cpu_sched.enqueue(i); }
        for i in io_queue { io_sched.enqueue(i); }
        state.time += 1;
        gui.draw();

        if remaining_processes == 0 {
            return;
        }
    }
}
