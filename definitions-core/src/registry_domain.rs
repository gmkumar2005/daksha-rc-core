//TODO RollBack Command
use crate::definitions_domain::{
    generate_id_from_title, DefId, DefRecordStatus, DomainEvent, RegistryDefinition, Version,
};
use chrono::{DateTime, Utc};
use disintegrate::{event_types, union, Decision, StateMutate, StateQuery, StreamQuery};
use log::debug;
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
    DefinitionNotInProperState(DefRecordStatus, DefRecordStatus),
    #[error("Cannot modify entity which is in `{0}")]
    ModifyNotAllowed(EntityRecordStatus),
    #[error("Validation of the entity {0} failed with following errors : \n{1}")]
    JsonSchemaError(String, String),
    #[error("Cannot delete entity which is in `{0}")]
    DeleteNotAllowed(EntityRecordStatus),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Display)]
pub enum EntityRecordStatus {
    #[default]
    None,
    Active,
    Invited,
    Modified,
    Deactivated,
    MarkedForDeletion,
}

#[derive(Default, StateQuery, Clone, Debug, Serialize, Deserialize)]
#[state_query(DomainEvent)]
pub struct RegistryResource {
    #[id]
    id: EntityId,
    status: EntityRecordStatus,
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
                self.status = EntityRecordStatus::Active;
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
                self.status = EntityRecordStatus::Invited;
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
    // TODO ignore records which are in modified status
    fn state_query(&self) -> Self::StateQuery {
        (
            RegistryResource::new(self.id),
            RegistryDefinition::new(generate_id_from_title(&self.entity_type)),
        )
    }

    fn validation_query<ID: disintegrate::EventId>(&self) -> Option<StreamQuery<ID, Self::Event>> {
        let (resource, def_state) = self.state_query();
        Some(union!(
            &resource,
            def_state.exclude_events(event_types!(DomainEvent, [DefUpdated]))
        ))
    }
    fn process(
        &self,
        (resource, def_state): &Self::StateQuery,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        if !state_machine(&resource.status, RegistryEntityAction::Create) {
            return Err(EntityError::EntityAlreadyExists(
                self.entity_type.clone(),
                self.id,
            ));
        }
        if def_state.record_status != DefRecordStatus::Active {
            return Err(EntityError::DefinitionNotInProperState(
                DefRecordStatus::Active,
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
            let pretty = serde_json::to_string_pretty(&schema).unwrap();
            debug!("pretty schema = {}", pretty);
            let pretty = serde_json::to_string_pretty(&instance).unwrap();
            debug!("pretty instance = {}", pretty);
        }
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
            version: Default::default(),
        }])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModifyEntityCmd {
    pub id: EntityId,
    pub entity_body: String,
    pub entity_type: String,
    pub modified_by: String,
}

impl Decision for ModifyEntityCmd {
    type Event = DomainEvent;
    type StateQuery = (RegistryResource, RegistryDefinition);
    type Error = EntityError;

    // StateQuery should load schema_def which has an id  same as the generate_id_from_title
    // TODO ignore records which are in modified status
    fn state_query(&self) -> Self::StateQuery {
        (
            RegistryResource::new(self.id),
            RegistryDefinition::new(generate_id_from_title(&self.entity_type)),
        )
    }

    fn validation_query<ID: disintegrate::EventId>(&self) -> Option<StreamQuery<ID, Self::Event>> {
        let (resource, def_state) = self.state_query();
        Some(union!(
            &resource,
            def_state.exclude_events(event_types!(DomainEvent, [DefUpdated]))
        ))
    }

    fn process(
        &self,
        (resource, def_state): &Self::StateQuery,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        if !state_machine(&resource.status, RegistryEntityAction::Modify) {
            return Err(EntityError::ModifyNotAllowed(resource.status.clone()));
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
            let pretty = serde_json::to_string_pretty(&schema).unwrap();
            debug!("pretty schema = {}", pretty);
            let pretty = serde_json::to_string_pretty(&instance).unwrap();
            debug!("pretty instance = {}", pretty);
        }
        if !full_errors.is_empty() {
            return Err(EntityError::JsonSchemaError(
                self.entity_type.clone(),
                full_errors,
            ));
        }
        Ok(vec![DomainEvent::EntityUpdated {
            id: self.id,
            registry_def_id: def_state.id,
            registry_def_version: def_state.version,
            entity_body: self.entity_body.clone(),
            entity_type: self.entity_type.to_string(),
            updated_at: Utc::now(),
            updated_by: self.modified_by.clone(),
            version: resource.version.increment(),
        }])
    }
}

pub struct DeleteCmd {
    id: DefId,
    deleted_at: DateTime<Utc>,
    deleted_by: String,
}
impl Decision for DeleteCmd {
    type Event = DomainEvent;
    type StateQuery = RegistryResource;
    type Error = EntityError;
    fn state_query(&self) -> Self::StateQuery {
        RegistryResource::new(self.id)
    }

    fn process(&self, resource: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state_machine(&resource.status, RegistryEntityAction::MarkForDeletion) {
            return Err(EntityError::DeleteNotAllowed(resource.status.clone()));
        }

        Ok(vec![DomainEvent::EntityDeleted {
            id: self.id,
            deleted_at: self.deleted_at,
            deleted_by: self.deleted_by.clone(),
        }])
    }
}

// Start of state machine
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryEntityAction {
    Create,
    Modify,
    MarkForDeletion,
    Deactivate,
    Invite,
}

pub fn state_machine(current_status: &EntityRecordStatus, action: RegistryEntityAction) -> bool {
    match action {
        RegistryEntityAction::Modify => {
            matches!(
                current_status,
                EntityRecordStatus::Active
                    | EntityRecordStatus::Modified
                    | EntityRecordStatus::Invited
            )
        }
        RegistryEntityAction::Create => {
            matches!(current_status, EntityRecordStatus::None)
        }
        RegistryEntityAction::MarkForDeletion => {
            matches!(current_status, EntityRecordStatus::Active)
        }
        _ => false,
        // Add more actions as needed, with appropriate logic
    }
}
