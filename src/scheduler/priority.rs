
use std::collections::{VecDeque, BinaryHeap};

use crate::{process::{Process, BurstKind, Burst}, system_state::SystemState};

use super::{Scheduler, SchedulerResult};

#[derive(PartialEq, Eq)]
struct PriorityProcess {
    process: Process
}

impl PartialOrd for PriorityProcess {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.process.priority.partial_cmp(&self.process.priority)
    }
}
impl Ord for PriorityProcess {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.process.priority.cmp(&self.process.priority)
    }
}

pub struct Priority {
    // FCFS is a FIFO algorithm. It takes processes by arrival time,
    // and processes the ones that came in first. A VecDeque,
    // which can function as a queue (or a stack, it has both pop_{front, back} methods)
    // is nice for this.
    processes: BinaryHeap<PriorityProcess>,
    kind: BurstKind,
}

impl Priority {
    pub fn new(processes: Vec<Process>, kind: BurstKind) -> Self {
        Self {
            processes: processes.into_iter().map(|proc| PriorityProcess { process: proc }).collect(),
            kind,
        }
    }
}

impl Scheduler for Priority {
    fn get_queue(&self) -> Vec<&Process> {
        self.processes.iter().map(|proc| &proc.process).collect()
    }

    fn tick(&mut self, system_state: &SystemState) -> SchedulerResult {
        let mut process = match self.processes.peek_mut() {
            Some(p) => p,
            None => return SchedulerResult::Idle,
        };
        if process.process.arrival > system_state.time { return SchedulerResult::Idle; }
        match process.process.burst.front_mut() {
            Some(Burst(kind, burst_amt)) if self.kind == *kind => {
                *burst_amt -= 1;
                if *burst_amt == 0 {
                    std::mem::drop(process);
                    let mut proc = self.processes.pop().unwrap();
                    proc.process.burst.pop_front();
                    SchedulerResult::Finished(proc.process)
                } else {
                    SchedulerResult::Processing(process.process.clone())
                }
            }
            Some(Burst(_, _)) => SchedulerResult::WrongKind,
            None => SchedulerResult::NoBurstLeft,
        }
    }

    fn enqueue(&mut self, proc: Process) {
        self.processes.push(PriorityProcess { process: proc })
    }
}
