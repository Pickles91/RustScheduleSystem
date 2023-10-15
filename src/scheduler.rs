use crate::system_state::SystemState;
use crate::process::Process;


pub mod fcfs;
pub mod priority;
pub mod round_robin;

#[derive(PartialEq, Debug, Clone)]
pub enum SchedulerResult {
    Finished(Process),
    // remaining burst
    Processing(Process),
    Idle,
    WrongKind,
    NoBurstLeft,
}


pub trait Scheduler {
    fn tick(&mut self, system_state: &SystemState) -> SchedulerResult;
    fn enqueue(&mut self, proc: Process);
    fn get_queue(&self) -> Vec<&Process>;
}

