use anyhow::Error;
use tree_ds::prelude::Tree;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StackNode {
    id: String,
    value: i32,
}

impl StackNode {
    pub fn new(id: String, value: i32) -> Result<StackNode, Error>{
        Ok(StackNode{
            id,
            value,
        })
    }
}

impl fmt::Display for StackNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(f, "{}: {}", self.id, self.value);
    }
}