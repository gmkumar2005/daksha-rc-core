use anyhow::Error;
use chrono::{DateTime, Utc};
use disintegrate::{Decision, Event, StateMutate, StateQuery};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type DefId = i64;
// Start of domain events
#[derive(Debug, Clone, PartialEq, Eq, Event, Serialize, Deserialize)]
#[stream(DefStateEvent, [DefLoaded, DefCreated, DefUpdated, DefDeleted, DefValidated, DefActivated, DefDeactivated]
)]
// #[stream(DefChangeEvents, [PropertiesAdded, PropertiesRemoved, PropertiesReplaced, VisibilityModified, AttestationPoliciesAdded, AttestationPoliciesReplaced, OwnerShipAttributesAdded, OwnerShipAttributesReplaced])]
pub enum DomainEvent {
    DefLoaded {
        #[id]
        def_id: DefId,
        title: String,
        definitions: Vec<String>,
        file_name: String,
        created_at: DateTime<Utc>,
        created_by: String,
        json_schema_string: String,
    },
    DefCreated {
        #[id]
        def_id: DefId,
        title: String,
        definitions: Vec<String>,
        created_at: DateTime<Utc>,
        created_by: String,
        json_schema_string: String,
    },
    DefUpdated {
        #[id]
        def_id: DefId,
        title: String,
        definitions: Vec<String>,
        created_at: DateTime<Utc>,
        updated_by: String,
        json_schema_string: String,
    },
    DefDeleted {
        #[id]
        def_id: DefId,
        deleted_at: DateTime<Utc>,
        deleted_by: String,
    },
    DefValidated {
        #[id]
        def_id: DefId,
        validated_at: DateTime<Utc>,
        validated_by: String,
        validation_result: String,
    },
    DefValidatedFailed {
        #[id]
        def_id: DefId,
        validated_at: DateTime<Utc>,
        validated_by: String,
        validation_result: String,
        validation_errors: Vec<String>,
    },
    DefActivated {
        #[id]
        def_id: DefId,
        activated_at: DateTime<Utc>,
        activated_by: String,
    },
    DefDeactivated {
        #[id]
        def_id: DefId,
        deactivated_at: DateTime<Utc>,
        deactivated_by: String,
    },

}

// start of errors
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DefError {
    #[error("Invalid Json")]
    InvalidJson,
    #[error("Invalid Schema: {0}")]
    InvalidSchema(String),
    #[error("Invalid Definition")]
    InvalidDefinition,
    #[error("Definition Already Exists")]
    DefinitionAlreadyExists,
    #[error("Definition Not Found")]
    DefinitionNotFound,
    #[error("Definition Not Valid")]
    DefinitionNotValid,
    #[error("Definition Not Active")]
    DefinitionNotActive,
}

// start of mutations

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RecordStatus {
    #[default]
    None,
    Draft,
    Valid,
    Active,
    Deactivated,
    Invalid,
    MarkedForDeletion,
}
#[derive(Default, StateQuery, Clone, Debug, Serialize, Deserialize)]
#[state_query(DomainEvent)]
pub struct DefState {
    #[id]
    def_id: DefId,
    record_status: RecordStatus,
    json_schema_string: String,
    created_at: DateTime<Utc>,
    created_by: String,
    title: String,
    definitions: Vec<String>,
}

impl DefState {
    pub fn new(def_id: DefId) -> Self {
        Self {
            def_id,
            record_status: RecordStatus::None,
            json_schema_string: "".to_string(),
            created_at: Utc::now(),
            created_by: "".to_string(),
            title: "".to_string(),
            definitions: vec![],
        }
    }
}

impl StateMutate for DefState {
    fn mutate(&mut self, event: Self::Event) {
        match event {
            DomainEvent::DefCreated { def_id, title, definitions, created_at, created_by, json_schema_string } => {
                self.record_status = RecordStatus::Draft;
                self.def_id = def_id;
                self.title = title;
                self.definitions = definitions;
                self.created_at = created_at;
                self.created_by = created_by;
                self.json_schema_string = json_schema_string;
            }
            DomainEvent::DefUpdated { .. } => {
                self.record_status = RecordStatus::Draft;
            }
            DomainEvent::DefValidated { .. } => {
                self.record_status = RecordStatus::Valid;
            }
            DomainEvent::DefValidatedFailed { .. } => {
                self.record_status = RecordStatus::Invalid;
            }
            DomainEvent::DefActivated { .. } => {
                self.record_status = RecordStatus::Active;
            }
            DomainEvent::DefDeactivated { .. } => {
                self.record_status = RecordStatus::Deactivated;
            }
            DomainEvent::DefDeleted { .. } => {
                self.record_status = RecordStatus::MarkedForDeletion;
            }
            _ => {}
        }
    }
}

// Start of commands
pub struct LoadDefinition {
    def_id: DefId,
    def_title: String,
    definitions: Vec<String>,
    file_name: String,
    created_at: DateTime<Utc>,
    created_by: String,
    json_schema_string: String,
}

impl Decision for LoadDefinition {
    type Event = DomainEvent;
    type StateQuery = DefState;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        DefState::new(self.def_id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.record_status != RecordStatus::None {
            return Err(DefError::DefinitionAlreadyExists);
        }
        Ok(vec![DomainEvent::DefLoaded {
            def_id: self.def_id,
            title: self.def_title.clone(),
            definitions: self.definitions.clone(),
            file_name: self.file_name.clone(),
            created_at: self.created_at,
            created_by: self.created_by.clone(),
            json_schema_string: self.json_schema_string.clone(),
        }])
    }
}

pub struct CreateDefinition {
    def_id: DefId,
    def_title: String,
    definitions: Vec<String>,
    created_at: DateTime<Utc>,
    created_by: String,
    json_schema_string: String,
}
impl Decision for CreateDefinition {
    type Event = DomainEvent;
    type StateQuery = DefState;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        DefState::new(self.def_id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.record_status != RecordStatus::None {
            return Err(DefError::DefinitionAlreadyExists);
        }
        Ok(vec![DomainEvent::DefCreated {
            def_id: self.def_id,
            title: self.def_title.clone(),
            definitions: self.definitions.clone(),
            created_at: self.created_at,
            created_by: self.created_by.clone(),
            json_schema_string: self.json_schema_string.clone(),
        }])
    }
}

pub struct UpdateDefinition {
    def_id: DefId,
    def_title: String,
    definitions: Vec<String>,
    created_at: DateTime<Utc>,
    updated_by: String,
    json_schema_string: String,
}

impl Decision for UpdateDefinition {
    type Event = DomainEvent;
    type StateQuery = DefState;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        DefState::new(self.def_id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {

        // updates are allowed only for draft definitions and inactive definitions
        if state.record_status != RecordStatus::Draft && state.record_status != RecordStatus::Deactivated {
            return Err(DefError::DefinitionNotValid);
        }
        Ok(vec![DomainEvent::DefUpdated {
            def_id: self.def_id,
            title: self.def_title.clone(),
            definitions: self.definitions.clone(),
            created_at: self.created_at,
            updated_by: self.updated_by.clone(),
            json_schema_string: self.json_schema_string.clone(),
        }])
    }
}

pub struct ValidateDefinition {
    def_id: DefId,
    validated_at: DateTime<Utc>,
    validated_by: String,
}

// Load the definition check if the
impl Decision for ValidateDefinition {
    type Event = DomainEvent;
    type StateQuery = DefState;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        DefState::new(self.def_id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.record_status != RecordStatus::Draft {
            return Err(DefError::DefinitionNotValid);
        }

        // TODO: validate the json and json schema
        match validate_schema(&state.json_schema_string) {
            Ok(result) if result == "Success" => Ok(vec![DomainEvent::DefValidated {
                def_id: self.def_id,
                validated_at: self.validated_at,
                validated_by: self.validated_by.clone(),
                validation_result: result,
            }]),
            Ok(result) => Ok(vec![DomainEvent::DefValidatedFailed {
                def_id: self.def_id,
                validated_at: self.validated_at,
                validated_by: self.validated_by.clone(),
                validation_result: result,
                validation_errors: vec![],
            }]),
            Err(err) => Ok(vec![DomainEvent::DefValidatedFailed {
                def_id: self.def_id,
                validated_at: self.validated_at,
                validated_by: self.validated_by.clone(),
                validation_result: "failure".to_string(),
                validation_errors: vec![err.to_string()],
            }]),
        }
    }
}

pub struct ActivateDefinition {
    def_id: DefId,
    activated_at: DateTime<Utc>,
    activated_by: String,
}
impl Decision for ActivateDefinition {
    type Event = DomainEvent;
    type StateQuery = DefState;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        DefState::new(self.def_id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.record_status != RecordStatus::Valid {
            return Err(DefError::DefinitionNotValid);
        }
        Ok(vec![DomainEvent::DefActivated {
            def_id: self.def_id,
            activated_at: self.activated_at,
            activated_by: self.activated_by.clone(),
        }])
    }
}

pub struct DeactivateDefinition {
    def_id: DefId,
    deactivated_at: DateTime<Utc>,
    deactivated_by: String,
}
impl Decision for DeactivateDefinition {
    type Event = DomainEvent;
    type StateQuery = DefState;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        DefState::new(self.def_id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.record_status != RecordStatus::Active {
            return Err(DefError::DefinitionNotActive);
        }
        Ok(vec![DomainEvent::DefDeactivated {
            def_id: self.def_id,
            deactivated_at: self.deactivated_at,
            deactivated_by: self.deactivated_by.clone(),
        }])
    }
}

pub struct DeleteDefinition {
    def_id: DefId,
    deleted_at: DateTime<Utc>,
    deleted_by: String,
}
impl Decision for DeleteDefinition {
    type Event = DomainEvent;
    type StateQuery = DefState;
    type Error = DefError;
    fn state_query(&self) -> Self::StateQuery {
        DefState::new(self.def_id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.record_status == RecordStatus::MarkedForDeletion {
            return Err(DefError::DefinitionNotFound);
        }
        Ok(vec![DomainEvent::DefDeleted {
            def_id: self.def_id,
            deleted_at: self.deleted_at,
            deleted_by: self.deleted_by.clone(),
        }])
    }
}

// start helper functions

fn validate_schema(p0: &String) -> Result<String, Error> {
    if !p0.is_empty() {
        Ok("Success".to_string())
    } else {
        Err(DefError::InvalidSchema("Schema is empty".to_string()).into())
    }
}

// start test cases
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_definition() {
        let date_str = "2024-11-22T16:46:51.757980Z";
        let created_at: DateTime<Utc> = date_str.parse().expect("Failed to parse date");

        let create_def_cmd = CreateDefinition {
            def_id: 1,
            def_title: "test_title".to_string(),
            definitions: vec!["test_def".to_string()],
            created_at: created_at.clone(),
            created_by: "test_created_by".to_string(),
            json_schema_string: "{}".to_string(),
        };

        disintegrate::TestHarness::given([])
            .when(create_def_cmd)
            .then([DomainEvent::DefCreated {
                def_id: 1,
                title: "test_title".to_string(),
                definitions: vec!["test_def".to_string()],
                created_at: created_at.clone(),
                created_by: "test_created_by".to_string(),
                json_schema_string: "{}".to_string(),
            }]);
    }

    #[test]
    fn test_validate_definition() {
        let date_str = "2024-11-22T16:46:51.757980Z";
        let validated_at: DateTime<Utc> = date_str.parse().expect("Failed to parse date");

        let validate_def_cmd = ValidateDefinition {
            def_id: 1,
            validated_at: validated_at.clone(),
            validated_by: "test_validated_by".to_string(),
        };

        disintegrate::TestHarness::given([DomainEvent::DefCreated {
            def_id: 1,
            title: "test_title".to_string(),
            definitions: vec!["test_def".to_string()],
            created_at: validated_at.clone(),
            created_by: "test_created_by".to_string(),
            json_schema_string: "{}".to_string(),
        }])
            .when(validate_def_cmd)
            .then([DomainEvent::DefValidated {
                def_id: 1,
                validated_at: validated_at.clone(),
                validated_by: "test_validated_by".to_string(),
                validation_result: "Success".to_string(),
            }]);
    }

    #[test]
    fn test_validate_definition_empty_schema() {
        let date_str = "2024-11-22T16:46:51.757980Z";
        let validated_at: DateTime<Utc> = date_str.parse().expect("Failed to parse date");

        let validate_def_cmd = ValidateDefinition {
            def_id: 1,
            validated_at: validated_at.clone(),
            validated_by: "test_validated_by".to_string(),
        };

        disintegrate::TestHarness::given([DomainEvent::DefCreated {
            def_id: 1,
            title: "test_title".to_string(),
            definitions: vec!["test_def".to_string()],
            created_at: validated_at.clone(),
            created_by: "test_created_by".to_string(),
            json_schema_string: "".to_string(),
        }])
            .when(validate_def_cmd)
            .then([DomainEvent::DefValidatedFailed {
                def_id: 1,
                validated_at: validated_at.clone(),
                validated_by: "test_validated_by".to_string(),
                validation_result: "failure".to_string(),
                validation_errors: vec!["Invalid Schema: Schema is empty".to_string()],
            }]);
    }

}