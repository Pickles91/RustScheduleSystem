use std::collections::BinaryHeap;
use crate::{process::{Process, BurstKind, Burst}, system_state::SystemState, util::Ordered};
use super::{Scheduler, SchedulerResult};
use std::cmp::Ordering;

pub struct PriorityScheduler {
    processes: BinaryHeap<Ordered<Process, Box<dyn Fn(&Process, &Process) -> Ordering>>>,
    kind: BurstKind,
}

impl PriorityScheduler {
    pub fn new(kind: BurstKind) -> Self {
        PriorityScheduler {
            processes: BinaryHeap::new(),
            kind,
        }
    }
}

fn priority_cmp(a: &Process, b: &Process) -> Ordering {
    a.priority.cmp(&b.priority)
}

impl Scheduler for PriorityScheduler {

    fn tick(&mut self, system_state: &SystemState) -> SchedulerResult<'_> {
        let mut process = match self.processes.peek_mut() {
             Some(v) => v,
             None => return SchedulerResult::Idle,
         };
         if process.0.arrival > system_state.time {
             return SchedulerResult::Idle;
         }

        match process.0.burst.front_mut() {
            Some(Burst(kind, burst_amt)) if self.kind == *kind => { 
                *burst_amt -= 1;
                if *burst_amt == 0 {
                    std::mem::drop(process);
                    let proc = self.processes.pop().unwrap().0;
                    SchedulerResult::Finished(proc)
                } else {
                    std::mem::drop(process);
                    SchedulerResult::Processing(&self.processes.peek().unwrap().0)
                }
            },
            // in this case we must not match the kind
            Some(Burst(_, _)) => SchedulerResult::WrongKind,
            None => SchedulerResult::NoBurstLeft,
        }
    }

    fn enqueue(&mut self, proc: Process) {
        self.processes.push(Ordered(proc, Box::new(priority_cmp)));
    }
}
   
