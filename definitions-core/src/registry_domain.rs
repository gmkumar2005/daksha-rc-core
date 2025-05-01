//TODO Json Schema Validation with Records
use crate::definitions_domain::{
    generate_id_from_title, DefId, DomainEvent, RecordStatus, RegistryDefinition, Version,
};
use chrono::Utc;
use disintegrate::{Decision, StateMutate, StateQuery};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use thiserror::Error;
use uuid::Uuid;

pub type EntityId = Uuid;
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EntityError {
    #[error("Entity Already Exists for : {0} with id: {1}")]
    EntityAlreadyExists(String, EntityId),
    #[error("Invalid Json: {0}")]
    InvalidJson(String),
    #[error("Invalid Schema: {0}")]
    InvalidSchema(String),
    #[error("Invalid Definition")]
    InvalidDefinition,
    #[error("Definition is expected to be `{0}` state. It is in `{1}` state")]
    DefinitionNotInProperState(RecordStatus, RecordStatus),
    #[error("Validation of the entity {0} failed with following errors : \n{1}")]
    JsonSchemaError(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Display)]
pub enum LifecycleState {
    #[default]
    None,
    Draft,
    Invited,
    Valid,
    Active,
    Deactivated,
    Invalid,
    MarkedForDeletion,
}

#[derive(Default, StateQuery, Clone, Debug, Serialize, Deserialize)]
#[state_query(DomainEvent)]
pub struct RegistryResource {
    #[id]
    id: EntityId,
    status: LifecycleState,
    /// Version of the definitions used to create or modify this resource
    registry_def_version: Version,
    registry_def_id: DefId,
    /// Version indicating a number of modifications happened to this resource
    version: Version,
    entity_body: String,
    entity_type: String,
}

impl RegistryResource {
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
}

impl StateMutate for RegistryResource {
    fn mutate(&mut self, event: Self::Event) {
        match event {
            DomainEvent::EntityCreated {
                id,
                registry_def_id,
                registry_def_version,
                entity_body,
                entity_type,
                ..
            } => {
                self.id = id;
                self.registry_def_id = registry_def_id;
                self.registry_def_version = registry_def_version;
                self.entity_body = entity_body;
                self.entity_type = entity_type;
                self.status = LifecycleState::Draft;
            }
            DomainEvent::EntityInvited {
                id,
                registry_def_id,
                registry_def_version,
                entity_body,
                entity_type,
                ..
            } => {
                self.id = id;
                self.registry_def_id = registry_def_id;
                self.registry_def_version = registry_def_version;
                self.entity_body = entity_body;
                self.entity_type = entity_type;
                self.status = LifecycleState::Invited;
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CreateEntityCmd {
    pub id: EntityId,
    pub entity_body: String,
    pub entity_type: String,
    pub created_by: String,
}
impl Decision for CreateEntityCmd {
    type Event = DomainEvent;
    type StateQuery = (RegistryResource, RegistryDefinition);
    type Error = EntityError;

    // StateQuery should load schema_def which has an id  same as the generate_id_from_title
    fn state_query(&self) -> Self::StateQuery {
        (
            RegistryResource::new(self.id),
            RegistryDefinition::new(generate_id_from_title(&self.entity_type)),
        )
    }

    fn process(
        &self,
        (resource, def_state): &Self::StateQuery,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        if resource.status != LifecycleState::None {
            return Err(EntityError::EntityAlreadyExists(
                self.entity_type.clone(),
                self.id,
            ));
        }
        if def_state.record_status != RecordStatus::Active {
            return Err(EntityError::DefinitionNotInProperState(
                RecordStatus::Active,
                def_state.record_status.clone(),
            ));
        }

        let schema: serde_json::Value = serde_json::from_str(&def_state.json_schema_string)
            .map_err(|e| EntityError::JsonSchemaError(self.entity_type.clone(), e.to_string()))?;
        let instance: serde_json::Value = serde_json::from_str(&self.entity_body)
            .map_err(|e| EntityError::JsonSchemaError(self.entity_type.clone(), e.to_string()))?;

        let validator = jsonschema::draft7::new(&schema)
            .map_err(|e| EntityError::JsonSchemaError(self.entity_type.clone(), e.to_string()))?;

        let full_errors = validator
            .iter_errors(&instance)
            .map(|error| format!("Error: {} \tLocation: {}", error, error.instance_path))
            .collect::<Vec<_>>()
            .join("\n");

        if !full_errors.is_empty() {
            return Err(EntityError::JsonSchemaError(
                self.entity_type.clone(),
                full_errors,
            ));
        }

        Ok(vec![DomainEvent::EntityCreated {
            id: self.id,
            registry_def_id: def_state.id,
            registry_def_version: def_state.version,
            entity_body: self.entity_body.clone(),
            entity_type: self.entity_type.to_string(),
            created_at: Utc::now(),
            created_by: self.created_by.clone(),
        }])
    }
}
