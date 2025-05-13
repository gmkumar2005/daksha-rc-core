//TODO Json Schema Validation with REF
//TODO RollBack Command
use crate::registry_domain::EntityId;
use chrono::{DateTime, Utc};
use disintegrate::{Decision, Event, StateMutate, StateQuery};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::num::NonZeroU16;
use strum_macros::Display;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version(NonZeroU16);
impl Default for Version {
    fn default() -> Self {
        Version(NonZeroU16::new(1).unwrap())
    }
}
impl Version {
    pub fn increment(self) -> Version {
        let new_val = self.0.get().wrapping_add(1).max(1); // ensures result is never 0
        Version(NonZeroU16::new(new_val).unwrap())
    }
    pub fn get(self) -> u16 {
        self.0.get()
    }
}

pub type DefId = Uuid;
// Start of domain events
#[derive(Debug, Clone, PartialEq, Eq, Event, Serialize, Deserialize)]
#[stream(DefStateEvent, [DefLoaded, DefCreated, DefUpdated, DefDeleted, DefValidated, DefActivated,
DefDeactivated]
)]
pub enum DomainEvent {
    DefLoaded {
        #[id]
        id: DefId,
        title: String,
        definitions: Vec<String>,
        file_name: String,
        created_at: DateTime<Utc>,
        created_by: String,
        json_schema_string: String,
    },
    DefCreated {
        #[id]
        id: DefId,
        title: String,
        definitions: Vec<String>,
        created_at: DateTime<Utc>,
        created_by: String,
        json_schema_string: String,
    },
    DefUpdated {
        #[id]
        id: DefId,
        title: String,
        definitions: Vec<String>,
        created_at: DateTime<Utc>,
        updated_by: String,
        json_schema_string: String,
    },
    DefDeleted {
        #[id]
        id: DefId,
        deleted_at: DateTime<Utc>,
        deleted_by: String,
    },
    DefValidated {
        #[id]
        id: DefId,
        validated_at: DateTime<Utc>,
        validated_by: String,
        validation_result: String,
    },
    DefValidatedFailed {
        #[id]
        id: DefId,
        validated_at: DateTime<Utc>,
        validated_by: String,
        validation_result: String,
        validation_errors: Vec<String>,
    },
    DefActivated {
        #[id]
        id: DefId,
        activated_at: DateTime<Utc>,
        activated_by: String,
    },
    DefDeactivated {
        #[id]
        id: DefId,
        deactivated_at: DateTime<Utc>,
        deactivated_by: String,
        json_schema_string: String,
    },
    EntityCreated {
        #[id]
        id: EntityId,
        registry_def_id: DefId,
        registry_def_version: Version,
        entity_body: String,
        entity_type: String,
        created_at: DateTime<Utc>,
        created_by: String,
        version: Version,
    },
    EntityInvited {
        #[id]
        id: EntityId,
        registry_def_id: DefId,
        registry_def_version: Version,
        entity_body: String,
        entity_type: String,
        invited_at: DateTime<Utc>,
        invited_by: String,
        version: Version,
    },
    EntityUpdated {
        #[id]
        id: EntityId,
        registry_def_id: DefId,
        registry_def_version: Version,
        entity_body: String,
        entity_type: String,
        updated_at: DateTime<Utc>,
        updated_by: String,
        version: Version,
    },
    EntityPropertyUpdated {
        #[id]
        id: EntityId,
        def_id: DefId,
        def_version: Version,
        property_name: String,
        property_value: String,
        updated_at: DateTime<Utc>,
        created_by: String,
    },
    EntityPropertyAdded {
        #[id]
        id: EntityId,
        def_id: DefId,
        def_version: Version,
        property_name: String,
        property_value: String,
        added_at: DateTime<Utc>,
        created_by: String,
    },
    EntityDeleted {
        #[id]
        id: EntityId,
        deleted_at: DateTime<Utc>,
        deleted_by: String,
    },
}

// start of errors
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DefError {
    #[error("Invalid Json: {0}")]
    InvalidJson(String),
    #[error("Invalid Schema: {0}")]
    InvalidSchema(String),
    #[error("Definition Already Exists for : {0} with id: {1}")]
    DefinitionAlreadyExists(String, String),
    #[error("Definition Not Valid")]
    DefinitionNotValid,
    #[error("Cannot validate definition which is in `{0}` state")]
    ValidateNotAllowed(DefRecordStatus),
    #[error("Cannot deactivate definition which is in `{0}` state")]
    DeactivateNotAllowed(DefRecordStatus),
    #[error("Cannot delete definition which is in `{0}` state")]
    DeleteNotAllowed(DefRecordStatus),
    #[error("Cannot modify definition which is in `{0}` state")]
    ModifyNotAllowed(DefRecordStatus),
    #[error("Cannot activate definition which is in `{0}` state")]
    ActivateNotAllowed(DefRecordStatus),
    #[error("Updating title: {0} to {1} is not allowed")]
    TitleIsNotMutable(String, String),
    #[error("Title  `{0}` and its Id `{1}` does not match")]
    DigestMismatch(String, DefId),
}

// start of mutations

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Display)]
pub enum DefRecordStatus {
    #[default]
    None,
    Draft,
    Valid,
    Active,
    Deactivated,
    Invalid,
    MarkedForDeletion,
    Modified,
}

/// This Aggregate is responsible for managing JSON Schemas and definitions
#[derive(Default, StateQuery, Clone, Debug, Serialize, Deserialize)]
#[state_query(DomainEvent)]
pub struct RegistryDefinition {
    #[id]
    pub id: DefId,
    pub record_status: DefRecordStatus,
    pub json_schema_string: String,
    pub title: String,
    pub version: Version,
}

impl RegistryDefinition {
    pub fn new(id: DefId) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
}

impl StateMutate for RegistryDefinition {
    fn mutate(&mut self, event: Self::Event) {
        match event {
            DomainEvent::DefCreated {
                id,
                title,
                definitions: _,
                created_at: _,
                created_by: _,
                json_schema_string,
            } => {
                self.record_status = DefRecordStatus::Draft;
                self.id = id;
                self.title = title;
                self.json_schema_string = json_schema_string;
            }
            DomainEvent::DefUpdated {
                id: _,
                title: _,
                definitions: _,
                created_at: _,
                updated_by: _,
                json_schema_string,
            } => {
                self.record_status = DefRecordStatus::Draft;
                self.json_schema_string = json_schema_string;
                self.version = self.version.increment();
            }
            DomainEvent::DefValidated { .. } => {
                self.record_status = DefRecordStatus::Valid;
            }
            DomainEvent::DefValidatedFailed { .. } => {
                self.record_status = DefRecordStatus::Invalid;
            }
            DomainEvent::DefActivated { .. } => {
                self.record_status = DefRecordStatus::Active;
            }
            DomainEvent::DefDeactivated { .. } => {
                self.record_status = DefRecordStatus::Deactivated;
            }
            DomainEvent::DefDeleted { .. } => {
                self.record_status = DefRecordStatus::MarkedForDeletion;
            }
            _ => {}
        }
    }
}

// Start of commands

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CreateDefinitionCmd {
    pub id: DefId,
    pub title: String,
    pub definitions: Vec<String>,
    pub created_by: String,
    pub json_schema_string: String,
}
impl Decision for CreateDefinitionCmd {
    type Event = DomainEvent;
    type StateQuery = RegistryDefinition;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        RegistryDefinition::new(self.id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state_machine(&state.record_status, RegistryDefAction::Create) {
            return Err(DefError::DefinitionAlreadyExists(
                self.title.clone(),
                self.id.to_string(),
            ));
        }
        let def_title = read_title(&self.json_schema_string)?;
        if generate_id_from_title(&def_title) != self.id {
            return Err(DefError::DigestMismatch(def_title, self.id));
        }
        Ok(vec![DomainEvent::DefCreated {
            id: self.id,
            title: def_title,
            definitions: self.definitions.clone(),
            created_at: Utc::now(),
            created_by: self.created_by.clone(),
            json_schema_string: self.json_schema_string.clone(),
        }])
    }
}

pub struct UpdateDefinitionCmd {
    pub id: DefId,
    pub definitions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: String,
    pub json_schema_string: String,
}

impl Decision for UpdateDefinitionCmd {
    type Event = DomainEvent;
    type StateQuery = RegistryDefinition;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        RegistryDefinition::new(self.id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state_machine(&state.record_status, RegistryDefAction::Modify) {
            return Err(DefError::ModifyNotAllowed(state.record_status.clone()));
        }
        let def_title = read_title(&self.json_schema_string)?;
        if def_title != state.title {
            return Err(DefError::TitleIsNotMutable(def_title, state.title.clone()));
        }
        Ok(vec![DomainEvent::DefUpdated {
            id: self.id,
            title: def_title,
            definitions: self.definitions.clone(),
            created_at: self.created_at,
            updated_by: self.updated_by.clone(),
            json_schema_string: self.json_schema_string.clone(),
        }])
    }
}

pub struct ValidateDefinitionCmd {
    pub id: DefId,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
}

// Load the definition check if the
impl Decision for ValidateDefinitionCmd {
    type Event = DomainEvent;
    type StateQuery = RegistryDefinition;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        RegistryDefinition::new(self.id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state_machine(&state.record_status, RegistryDefAction::Validate) {
            return Err(DefError::ValidateNotAllowed(state.record_status.clone()));
        }

        match read_title(&state.json_schema_string) {
            Ok(result) if !result.is_empty() => Ok(vec![DomainEvent::DefValidated {
                id: self.id,
                validated_at: self.validated_at,
                validated_by: self.validated_by.clone(),
                validation_result: "Success".to_string(),
            }]),
            Ok(result) => Ok(vec![DomainEvent::DefValidatedFailed {
                id: self.id,
                validated_at: self.validated_at,
                validated_by: self.validated_by.clone(),
                validation_result: result,
                validation_errors: vec![],
            }]),
            Err(err) => Ok(vec![DomainEvent::DefValidatedFailed {
                id: self.id,
                validated_at: self.validated_at,
                validated_by: self.validated_by.clone(),
                validation_result: "failure".to_string(),
                validation_errors: vec![err.to_string()],
            }]),
        }
    }
}

pub struct ActivateDefinitionCmd {
    pub id: DefId,
    pub activated_at: DateTime<Utc>,
    pub activated_by: String,
}
impl Decision for ActivateDefinitionCmd {
    type Event = DomainEvent;
    type StateQuery = RegistryDefinition;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        RegistryDefinition::new(self.id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state_machine(&state.record_status, RegistryDefAction::Activate) {
            return Ok(vec![]);
        }
        Ok(vec![DomainEvent::DefActivated {
            id: self.id,
            activated_at: self.activated_at,
            activated_by: self.activated_by.clone(),
        }])
    }
}

pub struct DeactivateDefinitionCmd {
    id: DefId,
    deactivated_at: DateTime<Utc>,
    deactivated_by: String,
}
impl Decision for DeactivateDefinitionCmd {
    type Event = DomainEvent;
    type StateQuery = RegistryDefinition;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        RegistryDefinition::new(self.id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state_machine(&state.record_status, RegistryDefAction::Deactivate) {
            return Ok(vec![]);
        }
        Ok(vec![DomainEvent::DefDeactivated {
            id: self.id,
            deactivated_at: self.deactivated_at,
            deactivated_by: self.deactivated_by.clone(),
            json_schema_string: state.json_schema_string.clone(),
        }])
    }
}

pub struct DeleteDefinitionCmd {
    id: DefId,
    deleted_at: DateTime<Utc>,
    deleted_by: String,
}
impl Decision for DeleteDefinitionCmd {
    type Event = DomainEvent;
    type StateQuery = RegistryDefinition;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        RegistryDefinition::new(self.id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state_machine(&state.record_status, RegistryDefAction::MarkForDeletion) {
            return Ok(vec![]);
        }
        Ok(vec![DomainEvent::DefDeleted {
            id: self.id,
            deleted_at: self.deleted_at,
            deleted_by: self.deleted_by.clone(),
        }])
    }
}

// start helper functions

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryDefAction {
    Create,
    UpLoad,
    Validate,
    Activate,
    Deactivate,
    Modify,
    MarkForDeletion,
}

pub fn state_machine(current_status: &DefRecordStatus, action: RegistryDefAction) -> bool {
    match action {
        RegistryDefAction::Modify => {
            matches!(
                current_status,
                DefRecordStatus::Active | DefRecordStatus::Modified
            )
        }
        RegistryDefAction::Create => matches!(current_status, DefRecordStatus::None),
        RegistryDefAction::MarkForDeletion => {
            matches!(current_status, DefRecordStatus::Active)
        }
        RegistryDefAction::Validate => {
            matches!(current_status, DefRecordStatus::Draft)
        }
        RegistryDefAction::Deactivate => {
            matches!(current_status, DefRecordStatus::Active)
        }
        RegistryDefAction::Activate => {
            matches!(current_status, DefRecordStatus::Valid)
        }

        _ => false,
        // Add more actions as needed, with appropriate logic
    }
}

pub fn read_title(p0: &str) -> Result<String, DefError> {
    if !p0.is_empty() {
        let schema_value: Value =
            serde_json::from_str(p0).map_err(|e| DefError::InvalidJson(e.to_string()))?;

        let title = schema_value["title"].as_str().unwrap_or("").to_string();
        if title.is_empty() {
            return Err(DefError::InvalidSchema("Title is empty".to_string()));
        }
        Ok(title)
    } else {
        Err(DefError::InvalidSchema("Schema is empty".to_string()))
    }
}

pub fn generate_id_from_title(title: &str) -> Uuid {
    // Step 1: Unicode normalization (NFC)
    let normalized_title = title.trim().nfc().collect::<String>();

    // Step 2: Convert to uppercase
    let upper_title = normalized_title.to_uppercase();

    // Step 3: Compute BLAKE3 hash
    let hash_bytes = blake3::hash(upper_title.as_bytes());

    // let truncated_hash = u128::from_le_bytes(hash_bytes[0..16].try_into().unwrap());
    let truncated_hash = &hash_bytes.as_bytes()[0..16];

    // let truncated_hash = i64::from_le_bytes(hash_bytes[0..8].try_into().unwrap());
    Uuid::from_bytes(truncated_hash.try_into().expect("Invalid hash length"))
}

#[cfg(test)] // Marks this module for testing
mod tests {
    use super::*;
    // Import the function from the current module

    #[test]
    fn test_generate_id_from_title() {
        // Arrange
        let title = "   My Test Title  "; // Input with extra spaces
        let generated_id = generate_id_from_title(title);
        println!("Generated UUID: {}", generated_id);
        // Assert that the generated UUID is valid
        let generated_id_again = generate_id_from_title(title);
        assert_eq!(generated_id, generated_id_again);
        // Different titles should generate different UUIDs
        let different_title = "Another Test Title";
        let different_id = generate_id_from_title(different_title);
        assert_ne!(generated_id, different_id);
    }

    #[test]
    fn test_generate_id_from_title_simple_values() {
        // Arrange
        let title = "Student"; // Input with extra spaces
        let generated_id = generate_id_from_title(title);
        println!("Generated UUID: {}", generated_id);
        // Assert that the generated UUID is valid
        let generated_id_again = generate_id_from_title(title);
        assert_eq!(generated_id, generated_id_again);
        assert_eq!(
            generated_id.to_string(),
            "1bd23c91-3379-b65b-11cc-64984050e35c"
        );
        // Different titles should generate different UUIDs
        let different_title = "test_title";
        let different_id = generate_id_from_title(different_title);
        assert_ne!(generated_id, different_id);
        assert_eq!(
            different_id.to_string(),
            "edddcff8-4970-283f-7ab1-9b925d059b69"
        );
    }
    #[test]
    fn test_generate_id_with_unicode() {
        // Arrange
        let title = "Café du Monde";
        let generated_id = generate_id_from_title(title);
        println!("Generated UUID: {}", generated_id);
        // Assert that the generated UUID is valid
        // let generated_id_again = generate_id_from_title(title);
        // Different titles should generate different UUIDs
        let different_title = "Another Test Title Café du Monde";
        let different_id = generate_id_from_title(different_title);
        assert_ne!(generated_id, different_id);
    }
}
