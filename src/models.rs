use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// TODO: derive the appropriate traits
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Status {
    Open,
    InProgress,
    Resolved,
    Closed,
}

// TODO: derive the appropriate traits
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Epic {
    pub name: String,
    pub description: String,
    pub status: Status,
    pub stories: Vec<u32>,
}

impl Epic {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            status: Status::Open,
            stories: vec![],
        }
    }
}

// TODO: derive the appropriate traits
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Story {
    pub name: String,
    pub description: String,
    pub status: Status,
}

impl Story {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            status: Status::Open,
        }
    }
}

// TODO: derive the appropriate traits
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DBState {
    pub last_item_id: u32,
    pub epics: HashMap<u32, Epic>,
    pub stories: HashMap<u32, Story>,
}