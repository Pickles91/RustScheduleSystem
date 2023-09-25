use crate::system_state::SystemState;
use crate::process::Process;

struct FCFS {
    processes: Vec<Process>,
    system_state: SystemState,
}

impl FCFS {
    pub fn new(mut processes: Vec<Process>) -> Self {
        processes.sort_by(|a,b| a.arrival.cmp(&b.arrival));
        Self {
            processes,system_state: SystemState::new(),
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