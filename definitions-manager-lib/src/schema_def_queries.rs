use crate::schema_def::{SchemaDef, Status};
use crate::schema_def_events::SchemaDefEvent;
use cqrs_es::{EventEnvelope, View};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq)]
pub struct SchemaDefView {
    pub id: String,
    pub title: String,
    pub version: u32,
    pub schema: String,
    pub status: Status,
}

impl View<SchemaDef> for SchemaDefView {
    fn update(&mut self, event: &EventEnvelope<SchemaDef>) {
        match &event.payload {
            SchemaDefEvent::DefCreated { id, schema } => {
                self.id = id.clone();
                self.schema = schema.clone();
            }
            SchemaDefEvent::DefValidated { .. } => {
                self.status = Status::Valid;
            }
            SchemaDefEvent::DefActivated { .. } => {
                self.status = Status::Active;
            }
            SchemaDefEvent::DefDeactivated { .. } => {
                self.status = Status::Inactive;
            }
            SchemaDefEvent::DefCreatedAndValidated { id, schema } => {
                self.id = id.clone();
                self.schema = schema.clone();
                self.status = Status::Valid;
            }
        }
    }
}

