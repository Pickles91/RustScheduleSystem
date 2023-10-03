use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
pub struct Burst(pub BurstKind, pub i32);

#[derive(Debug, PartialEq)]
pub enum BurstKind {
    Cpu,
    Io,
}

#[derive(Debug, PartialEq)]
pub struct Process{
    pub name: String,
    pid: i32,
    priority: i32,
    pub burst: VecDeque<Burst>,
    pub arrival: i32,
}

impl Process {
    pub fn new<T: Into<VecDeque<Burst>>>(name: String, pid: i32, priority: i32, burst:T, arrival: i32) -> Self {
        Self {
            name,
            pid,
            priority,
            burst: burst.into(),
            arrival,
        }
    }
}