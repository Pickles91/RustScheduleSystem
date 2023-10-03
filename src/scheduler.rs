use std::collections::VecDeque;

use crate::system_state::SystemState;
use crate::process::{Process, Burst, BurstKind};

struct FCFS {
    // FCFS is a FIFO algorithm. It takes processes by arrival time,
    // and processes the ones that came in first. A VecDeque,
    // which can function as a queue (or a stack, it has both pop_{front, back} methods)
    // is nice for this.
    processes: VecDeque<Process>,
    kind: BurstKind,
}

impl FCFS {
    pub fn new(mut processes: Vec<Process>, kind: BurstKind) -> Self {
        processes.sort_by(|a,b| a.arrival.cmp(&b.arrival));
        Self {
            processes: processes.into(),
            kind,
        }
    }
}

impl Scheduler for FCFS {

    fn tick(&mut self, system_state: &SystemState) -> SchedulerResult<'_> {
       let process = match self.processes.front_mut() {
            Some(v) => v,
            None => return SchedulerResult::Idle,
        };
        if process.arrival > system_state.time {
            return SchedulerResult::Idle;
        }

        match process.burst.front_mut() {
            Some(Burst(kind, burst_amt)) if self.kind == *kind => { 
                *burst_amt -= 1;
                if *burst_amt == 0 {
                    let mut proc = self.processes.pop_front().unwrap();
                    proc.burst.pop_front().unwrap();
                    SchedulerResult::Finished(proc)
                } else {
                    SchedulerResult::Processing(&self.processes[0])
                }
            },
            // in this case we must not match the kind
            Some(Burst(_, _)) => SchedulerResult::WrongKind,
            None => SchedulerResult::NoBurstLeft,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum SchedulerResult<'a> {
    Finished(Process),
    // remaining burst
    Processing(&'a Process),
    Idle,
    WrongKind,
    NoBurstLeft,
}


trait Scheduler {
    fn tick(&mut self, system_state: &SystemState) -> SchedulerResult;
}

#[cfg(test)]
mod tests {
    use super::*;
    // this shouldn't panic, or do anything really.
    #[test]
    fn test_fcfs_zero_process() {
        let mut sched = super::FCFS::new(Vec::new(), BurstKind::Cpu);
        let state = SystemState::new();
        assert_eq!(sched.tick(&state), SchedulerResult::Idle);
    }
    #[test]
    fn test_fcfs_one_process() {
        let mut state = SystemState::new();
        let mut sched = super::FCFS::new(vec![Process::new(String::from("test"), 0, 0, vec![Burst(BurstKind::Cpu, 7)], 0)], BurstKind::Cpu);
        for _ in 0..10 {
            sched.tick(&state);
            state.time += 1;
        }
        assert_eq!(sched.processes.len(), 0);
    }
    #[test]
    fn test_fcfs_one_process_different_arrival_time() {
        let mut state = SystemState::new();
        let mut sched = super::FCFS::new(vec![Process::new(String::from("test"), 0, 0, vec![Burst(BurstKind::Cpu, 10)], 2)], BurstKind::Cpu);
        for _ in 0..10 {
            sched.tick(&state);
            state.time += 1;
        }
        assert_eq!(sched.processes[0].burst.front(), Some(&Burst(BurstKind::Cpu, 2)));
    }
    #[test]
    fn test_fcfs_multiple_processes() {
        let mut state = SystemState::new();
        let mut sched = super::FCFS::new(vec![
            Process::new(String::from("test"), 0, 0, vec![Burst(BurstKind::Cpu, 10)], 0),
            Process::new(String::from("test2"), 1, 1, vec![Burst(BurstKind::Cpu, 7)], 5),
        ], BurstKind::Cpu);
        for _ in 0..9 {
            sched.tick(&state);
            state.time += 1;
        }
        match sched.tick(&state) {
            SchedulerResult::Finished(p) => assert_eq!(p.name, "test"),
            p => panic!("Expected SchedulerResult::Finished, got {p:?}"),
        }
        for _ in 0..6 {
            sched.tick(&state);
            state.time += 1;
        }
        assert!(matches!(sched.tick(&state), SchedulerResult::Finished(_)));
    }
    #[test]
    fn test_fcfs_multiple_processes_with_idle() {
        let mut state = SystemState::new();
        let mut sched = super::FCFS::new(vec![
            Process::new(String::from("test"), 0, 0, vec![Burst(BurstKind::Cpu, 10)], 0),
            Process::new(String::from("test2"), 1, 1, vec![Burst(BurstKind::Cpu, 7)], 11),
        ], BurstKind::Cpu);
        for _ in 0..10 {
            sched.tick(&state);
            state.time += 1;
        }
        assert_eq!(sched.processes.len(), 1);
        assert_eq!(sched.tick(&state), SchedulerResult::Idle);
        state.time += 1;
        for _ in 0..7 {
            sched.tick(&state);
            state.time += 1;
        }
        assert_eq!(sched.processes.len(), 0);
    }

     #[test]
     fn test_multi_bursts() {
         let mut state = SystemState::new();
         let mut cpu_sched = super::FCFS::new(vec![
             Process::new(String::from("cpu and io"), 0, 0, vec![Burst(BurstKind::Cpu, 10), Burst(BurstKind::Io, 10)], 0)
         ], BurstKind::Cpu);
         let mut io_sched = super::FCFS::new(vec![], BurstKind::Io);
         let proc = loop {
             if let SchedulerResult::Finished(proc) = cpu_sched.tick(&state) {
                 break proc;
             }
             state.time += 1;
         };
         assert_eq!(proc.burst[0], Burst(BurstKind::Io, 10));
         io_sched.processes.insert(0, proc);
         let proc = loop {
             if let SchedulerResult::Finished(proc) = io_sched.tick(&state) {
                 break proc;
             }
         };
         assert!(proc.burst.is_empty());
     }
}
