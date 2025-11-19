use std::collections::HashMap;

pub enum Status {
    Open,
    InProgress,
    Resolved,
    Closed,
}

pub struct Epic {
    name: String,
    description: String,
    status: Status,
    stories: Vec<u32>,
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

pub struct Story {
    name: String,
    description: String,
    status: Status,
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

pub struct DBState {
    last_item_id: u32,
    epics: HashMap<u32, Epic>,
    stories: HashMap<u32, Story>,
}