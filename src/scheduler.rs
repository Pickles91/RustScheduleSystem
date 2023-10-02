use std::collections::VecDeque;

use crate::system_state::SystemState;
use crate::process::Process;

struct FCFS {
    // FCFS is a FIFO algorithm. It takes processes by arrival time,
    // and processes the ones that came in first. A VecDeque,
    // which can function as a queue (or a stack, it has both pop_{front, back} methods)
    // is nice for this.
    processes: VecDeque<Process>,
    finished: Vec<Process>,
}

impl FCFS {
    pub fn new(mut processes: Vec<Process>) -> Self {
        processes.sort_by(|a,b| a.arrival.cmp(&b.arrival));
        Self {
            processes: processes.into(),
            finished: Vec::new(),
        }
    }
}

impl Scheduler for FCFS {
    fn tick(&mut self, system_state: &SystemState) {
       let process = match self.processes.get_mut(0) {
            Some(v) => v,
            None => return,
        };
        // if the next process to work on isn't ready yet -
        // increment the system time and return, since we
        // don't have anything to do.
        if process.arrival > system_state.time {
            return;
        }
        process.tick(system_state);
        if process.burst == 0 {
            // we've been working with the first process this entire time -
            // we know it exists so it's safe to just `unwrap()` it rather
            // then checking if it's there or not.
            self.finished.push(self.processes.pop_front().unwrap());
        }
    }
}

trait Scheduler {
    fn tick(&mut self, system_state: &SystemState);
}

#[cfg(test)]
mod tests {
    use super::*;
    // this shouldn't panic, or do anything really.
    #[test]
    fn test_fcfs_zero_process() {
        let mut sched = super::FCFS::new(Vec::new());
        let state = SystemState::new();
        sched.tick(&state);
    }
    #[test]
    fn test_fcfs_one_process() {
        let mut state = SystemState::new();
        let mut sched = super::FCFS::new(vec![Process::new(String::from("test"), 0, 0, 10, 0)]);
        for _ in 0..10 {
            sched.tick(&state);
            state.time += 1;
        }
        assert_eq!(sched.finished.len(), 1);
    }
    #[test]
    fn test_fcfs_one_process_different_arrival_time() {
        let mut state = SystemState::new();
        let mut sched = super::FCFS::new(vec![Process::new(String::from("test"), 0, 0, 10, 2)]);
        for _ in 0..10 {
            sched.tick(&state);
            state.time += 1;
        }
        assert_eq!(sched.processes[0].burst, 2);
        for _ in 0..2 {
            sched.tick(&state);
            state.time += 1;
        }
        assert_eq!(sched.finished.len(), 1);
    }
    #[test]
    fn test_fcfs_multiple_processes() {
        let mut state = SystemState::new();
        let mut sched = super::FCFS::new(vec![
            Process::new(String::from("test"), 0, 0, 10, 0),
            Process::new(String::from("test2"), 1, 1, 7, 5),
        ]);
        for _ in 0..10 {
            sched.tick(&state);
            state.time += 1;
        }
        assert_eq!(sched.finished.len(), 1);
        for _ in 0..7 {
            sched.tick(&state);
            state.time += 1;
        }
        assert_eq!(sched.finished.len(), 2);
    }
    #[test]
    fn test_fcfs_multiple_processes_with_idle() {
        let mut state = SystemState::new();
        let mut sched = super::FCFS::new(vec![
            Process::new(String::from("test"), 0, 0, 10, 0),
            Process::new(String::from("test2"), 1, 1, 7, 11),
        ]);
        for _ in 0..10 {
            sched.tick(&state);
            state.time += 1;
        }
        assert_eq!(sched.finished.len(), 1);
        for _ in 0..7 {
            sched.tick(&state);
            state.time += 1;
        }
        assert_eq!(sched.finished.len(), 1);
        sched.tick(&state);
        state.time += 1;
        assert_eq!(sched.finished.len(), 2);
    }
}