use crate::system_state::SystemState;

#[derive(Debug, PartialEq)]
pub struct Process{
    pub name: String,
    pid: i32,
    priority: i32,
    pub burst: i32,
    pub arrival: i32,
}

impl Process {
    pub fn new(name: String, pid: i32, priority: i32, burst:i32, arrival: i32) -> Self {
        Self {
            name,
            pid,
            priority,
            burst,
            arrival,
        }
    }
    pub fn tick(&mut self, state: &SystemState) {
        assert!(self.arrival <= state.time);
        self.burst -= 1;
    }
}