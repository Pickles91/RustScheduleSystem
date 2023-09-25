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
    system_state: SystemState,
}

impl FCFS {
    pub fn new(mut processes: Vec<Process>) -> Self {
        processes.sort_by(|a,b| a.arrival.cmp(&b.arrival));
        Self {
            processes: processes.into(),
            system_state: SystemState::new(),
            finished: Vec::new(),
        }
    }
}

impl Scheduler for FCFS {
    fn tick(&mut self) {
       let process = match self.processes.get_mut(0) {
            Some(v) => v,
            None => { self.system_state.time += 1; return;},
        };
        // if the next process to work on isn't ready yet -
        // increment the system time and return, since we
        // don't have anything to do.
        if process.arrival > self.system_state.time {
            self.system_state.time += 1;
            return;
        }
        process.tick(&self.system_state);
        if process.burst == 0 {
            // we've been working with the first process this entire time -
            // we know it exists so it's safe to just `unwrap()` it rather
            // then checking if it's there or not.
            self.finished.push(self.processes.pop_front().unwrap());
        }
        self.system_state.time += 1;
    }
}

trait Scheduler {
    fn tick(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fcfs_zero_process() {
        let mut sched = super::FCFS::new(Vec::new());
        sched.tick();
        assert_eq!(sched.system_state.time, 1);
    }
}