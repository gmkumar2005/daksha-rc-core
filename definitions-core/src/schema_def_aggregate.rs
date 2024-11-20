use async_trait::async_trait;
use cqrs_es::Aggregate;
use crate::schema_def::{SchemaDef, Status};
use crate::schema_def_commands::SchemaDefCommand;
use crate::schema_def_events::{SchemaDefError, SchemaDefEvent};
use crate::schema_def_services::SchemaDefServices;
#[async_trait]
impl Aggregate for SchemaDef {
    type Command = SchemaDefCommand;
    type Event = SchemaDefEvent;
    type Error = SchemaDefError;
    type Services = SchemaDefServices;

    // This identifier should be unique to the system.
    fn aggregate_type() -> String {
        "SchemaDef".to_string()
    }

    // The aggregate logic goes here. Note that this will be the _bulk_ of a CQRS system
    // so expect to use helper functions elsewhere to keep the code clean.
    async fn handle(
        &self,
        command: Self::Command,
        _services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            SchemaDefCommand::CreateDef { os_id: id, schema } => {
                //TODO error handling
                if self.os_id == id {
                    return Err(SchemaDefError::ExistsError {
                        error_message: format!("Aggregate with id {} already exists", id),
                        error_code: 400,
                    });
                }
                let schema_def = SchemaDef::new(id, schema)
                    .map_err(|e| SchemaDefError ::ValidationError{
                        error_message: e,
                        error_code: 400,
                    })?;
                Ok(vec![SchemaDefEvent::DefCreated {
                    os_id: schema_def.os_id,
                    schema: schema_def.schema,
                }])
            }
            SchemaDefCommand::CreateAndValidateDef { os_id: id, schema } => {
                //TODO error handling
                let schema_def = SchemaDef::new(id, schema)
                    .map_err(|e| SchemaDefError ::GeneralError {
                        error_message: e,
                        error_code: 400,
                    })?;
                schema_def.clone().validate_def()
                    .map_err(|e| SchemaDefError::ValidationError {
                        error_message: e,
                        error_code: 400,
                    })?;
                Ok(vec![SchemaDefEvent::DefValidated { os_id: schema_def.os_id, }])
            }
            SchemaDefCommand::ValidateDef { os_id: id } => {
                let schema_def = self.clone();
                if schema_def.os_id != id {
                    return Err(SchemaDefError::ValidationError{
                        error_message: format!("The id:{} in the aggregate does not match with id: {} in the command", id, schema_def.os_id.to_string()),
                        error_code: 400,
                    });
                }
                if let Err(e) = schema_def.clone().validate_def() {
                    return Err(SchemaDefError::ValidationError {
                        error_message: e.to_string(),
                        error_code: 400,
                    });
                }
                Ok(vec![SchemaDefEvent::DefValidated { os_id: schema_def.os_id, }])
            }
            SchemaDefCommand::ActivateDef{ os_id: id } => {
                let schema_def = self.clone();
                if schema_def.os_id != id {
                    return Err(SchemaDefError::ActivationError{
                        error_message: format!("The id:{} in the aggregate does not match with id: {} in the command", id, schema_def.os_id.to_string()),
                        error_code: 400,
                    });
                }
                if let Err(e) = schema_def.clone().activate() {
                    return Err(SchemaDefError::ActivationError {
                        error_message: e.to_string(),
                        error_code: 400,
                    });
                }
                Ok(vec![SchemaDefEvent::DefActivated { os_id: schema_def.os_id, }])
            }
            SchemaDefCommand::DeactivateDef{ os_id: id } => {
                let schema_def = self.clone();
                if schema_def.os_id != id {
                    return Err(SchemaDefError::DeactivationError {
                        error_message: format!("The id:{} in the aggregate does not match with id: {} in the command", id, schema_def.os_id.to_string()),
                        error_code: 400,
                    });
                }
                if let Err(e) = schema_def.clone().de_activate() {
                    return Err(SchemaDefError::DeactivationError {
                        error_message: e.to_string(),
                        error_code: 400,
                    });
                }
                Ok(vec![SchemaDefEvent::DefDeactivated { os_id: schema_def.os_id, }])
            }
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            SchemaDefEvent::DefCreated { os_id: id, schema } => {
                self.schema = schema;
                self.os_id = id;
                self.status = Status::Inactive;
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
            SchemaDefEvent::DefCreatedAndValidated { os_id: id, schema } => {
                self.schema = schema;
                self.os_id = id;
                self.status = Status::Valid;
            }
        }
    }
}