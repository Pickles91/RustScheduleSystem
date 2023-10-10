use std::collections::VecDeque;

use process::{Burst, BurstKind, Process};
use scheduler::{Scheduler, fcfs::FCFS};
use system_state::SystemState;

mod process;
mod system_state;
mod scheduler;

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


    loop {
        match processes.front() {
            Some(proc) if proc.arrival <= state.time => {
                cpu_sched.enqueue(processes.pop_front().unwrap());
                continue;
            }
            _ => {},
        }
        match cpu_sched.tick(&state) {
            scheduler::SchedulerResult::Finished(p) if p.burst.len() == 0 =>  {
                remaining_processes -= 1;
                println!("process on cpu finished completely {}", p.name);
            },
            scheduler::SchedulerResult::Finished(p) => {
                println!("process on cpu finished burst {}", p.name);
                match p.burst[0].0 {
                    BurstKind::Cpu => cpu_sched.enqueue(p),
                    BurstKind::Io => io_sched.enqueue(p),
                }
            }
            scheduler::SchedulerResult::Processing(p) => println!("cpu processing {}", p.name),
            scheduler::SchedulerResult::Idle => println!("cpu idling"),
            scheduler::SchedulerResult::WrongKind => println!("schedule for IO instead you idiot."),
            scheduler::SchedulerResult::NoBurstLeft => println!("no burst left."),
        };
        match io_sched.tick(&state) {
            scheduler::SchedulerResult::Finished(p) if p.burst.len() == 0 =>  {
                remaining_processes -= 1;
                println!("process on io finished completely {}", p.name);
            },
            scheduler::SchedulerResult::Finished(p) => {
                println!("process on IO finished burst {}", p.name);
                match p.burst[0].0 {
                    BurstKind::Cpu => cpu_sched.enqueue(p),
                    BurstKind::Io => io_sched.enqueue(p),
                }
            }
            scheduler::SchedulerResult::Processing(p) => println!("io processing {}", p.name),
            scheduler::SchedulerResult::Idle => println!("io idling"),
            scheduler::SchedulerResult::WrongKind => println!("schedule for IO instead you idiot."),
            scheduler::SchedulerResult::NoBurstLeft => println!("no burst left."),
        };
        state.time += 1;

        if remaining_processes == 0 {
            return;
        }
    }
}
