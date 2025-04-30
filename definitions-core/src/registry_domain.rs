use chrono::{DateTime, Utc};
use disintegrate::{Decision, Event, StateMutate, StateQuery};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use thiserror::Error;
use uuid::Uuid;

pub type EntityId = Uuid;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq, Event, Serialize, Deserialize)]
#[stream(RegistryStateEvent, [EntityCreated,EntityInvited,Updated,EntityPropertyUpdated,EntityPropertyAdded,EntityDeleted])]
pub enum RegistryDomainEvent {
    EntityCreated {
        #[id]
        id: EntityId,
        entity_body: String,
    },
    EntityInvited {
        #[id]
        id: EntityId,
        entity_body: String,
    },
    EntityUpdated {
        #[id]
        id: EntityId,
        entity_body: String,
    },
    EntityPropertyUpdated {
        #[id]
        id: EntityId,
        property_name: String,
        property_value: String,
    },
    EntityPropertyAdded {
        #[id]
        id: EntityId,
        property_name: String,
        property_value: String,
    },
    EntityDeleted {
        #[id]
        id: EntityId,
    },
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EntityError {
    #[error("Invalid Json: {0}")]
    InvalidJson(String),
    #[error("Invalid Schema: {0}")]
    InvalidSchema(String),
    #[error("Invalid Definition")]
    InvalidDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Display)]
pub enum LifecycleState {
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
#[state_query(RegistryDomainEvent)]
pub struct ResourceState {
    #[id]
    id: EntityId,
    status: LifecycleState,
    record: String,
    schema_version: String,
    created_at: DateTime<Utc>,
    created_by: String,
}

impl ResourceState {
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
}

impl StateMutate for ResourceState {
    fn mutate(&mut self, event: Self::Event) {
        match event {
            RegistryDomainEvent::EntityCreated { id, .. } => {
                self.id = id;
            }
            RegistryDomainEvent::EntityInvited { id, .. } => {
                self.id = id;
            }
            _ => {}
        }
    }
}

pub struct CreateEvent {
    pub id: EntityId,
    pub entity_body: String,
}
impl Decision for CreateEvent {
    type Event = RegistryDomainEvent;
    type StateQuery = ResourceState;
    type Error = EntityError;

    fn state_query(&self) -> Self::StateQuery {
        ResourceState::new(self.id)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        todo!()
    }
}
