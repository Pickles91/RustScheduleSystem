use crate::system_state::SystemState;
use crate::process::{Process, Burst, BurstKind};

pub mod fcfs;

#[derive(PartialEq, Debug)]
pub enum SchedulerResult<'a> {
    Finished(Process),
    // remaining burst
    Processing(&'a Process),
    Idle,
    WrongKind,
    NoBurstLeft,
}


pub trait Scheduler {
    fn tick(&mut self, system_state: &SystemState) -> SchedulerResult;
    fn enqueue(&mut self, proc: Process);
    fn get_queue(&self) -> Vec<&Process>;
}

