use std::fmt;
use cqrs_es::DomainEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SchemaDefEvent {
    DefCreated {
        id: String,
        schema: String,
    },
    DefCreatedAndValidated {
        id: String,
        schema: String,
    },
    DefValidated,
    DefActivated,
    DefDeactivated,
}

impl DomainEvent for SchemaDefEvent {
    fn event_type(&self) -> String {
        let event_type: &str = match self {
            SchemaDefEvent::DefCreated { .. } => "DefCreated",
            SchemaDefEvent::DefValidated { .. } => "DefValidated",
            SchemaDefEvent::DefActivated { .. } => "DefActivated",
            SchemaDefEvent::DefDeactivated { .. } => "DefDeactivated",
            SchemaDefEvent::DefCreatedAndValidated { .. } => "DefCreatedAndValidated",
        };
        event_type.to_string()
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}

#[derive(Debug)]
pub struct SchemaDefError {
    pub message: String,
    pub code: u32,
}

impl fmt::Display for SchemaDefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for SchemaDefError {}