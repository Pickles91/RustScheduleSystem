use crate::process::{Process, BurstKind, Burst};

use super::{Scheduler, SchedulerResult};

pub struct RoundRobin {
    quantum_time: i32,
    // I couldn't find a good premade RingBuffer that I liked
    // and didn't feel like making my own for this assignment.
    // We're just going to use this vector as a RingBuffer
    processes: Vec<Process>,
    kind: BurstKind,
    remaining_time: i32,
    index: usize,
}

impl RoundRobin {
    pub fn new(processes: Vec<Process>, kind: BurstKind, quantum_time: i32) -> Self {
        Self {
            processes,
            kind,
            quantum_time,
            remaining_time: quantum_time,
            index: 0,
        }
    }
}

impl Scheduler for RoundRobin {
    fn tick(&mut self, system_state: &crate::system_state::SystemState) -> super::SchedulerResult {
        let length = self.processes.len();
        if length == 0 {
            return SchedulerResult::NoBurstLeft;
        }

        let proc = self.processes.get_mut(self.index % length).unwrap();

        if proc.arrival > system_state.time {
            return SchedulerResult::Idle;
        }

        match proc.burst.front_mut() {
            Some(Burst(kind, amt)) if *kind == self.kind => {
                *amt -= 1;
                if *amt == 0 {
                    self.remaining_time = self.quantum_time;
                    let mut proc = self.processes.remove(self.index % length);
                    proc.burst.pop_front();
                    SchedulerResult::Finished(proc)
                } else {
                    self.remaining_time -= 1;
                    // needed so we aren't holding onto a mutable ref. inside of processes
                    // while we're working with it.
                    let proc = proc.clone();
                    // if we're out of remaining time and not done yet, go to the next process.
                    if self.remaining_time == 0 {
                        self.remaining_time = self.quantum_time;
                        self.index = self.index.wrapping_add(1);
                    }
                    SchedulerResult::Processing(proc)
                    
                }
            },
            Some(_) => {
                SchedulerResult::WrongKind
            }
            None => SchedulerResult::NoBurstLeft,
        }
    }

    fn enqueue(&mut self, proc: Process) {
        self.processes.push(proc);
    }

    fn get_queue(&self) -> Vec<&Process> {
        self.processes.iter().collect()
    }
}
