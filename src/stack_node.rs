use anyhow::Error;
use tree_ds::prelude::Tree;
use std::{fmt, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StackNode {
    id: String,
    value: Duration,
}

impl StackNode {
    pub fn new(id: String, value: Duration) -> Result<StackNode, Error>{
        Ok(StackNode{
            id,
            value,
        })
    }

    pub fn increment(&mut self, delay: Duration) {
        self.value += delay;
    }

    pub fn get_duration(&self) -> Duration{
        return self.value;
    }
}

impl fmt::Display for StackNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(f, "{}: {}ms", self.id, self.value.as_millis());
    }
}