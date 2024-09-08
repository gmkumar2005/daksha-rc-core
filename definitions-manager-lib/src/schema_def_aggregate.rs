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
        services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            SchemaDefCommand::CreateDef { id, schema } => {
                //TODO error handling
                let schema_def = SchemaDef::new(id, schema)
                    .map_err(|e| SchemaDefError {
                        message: e,
                        code: 400,
                    })?;
                Ok(vec![SchemaDefEvent::DefCreated {
                    id: schema_def.id,
                    schema: schema_def.schema,
                }])
            }
            SchemaDefCommand::CreateAndValidateDef { id, schema } => {
                //TODO error handling
                let schema_def = SchemaDef::new(id, schema)
                    .map_err(|e| SchemaDefError {
                        message: e,
                        code: 400,
                    })?;
                schema_def.clone().validate_def()
                    .map_err(|e| SchemaDefError {
                        message: e,
                        code: 400,
                    })?;
                Ok(vec![SchemaDefEvent::DefValidated])
            }
            SchemaDefCommand::ValidateDef => {
                let schema_def = self.clone();
                if let Err(e) = schema_def.clone().validate_def() {
                    return Err(SchemaDefError {
                        message: e.to_string(),
                        code: 400,
                    });
                }
                Ok(vec![SchemaDefEvent::DefValidated])
            }
            SchemaDefCommand::ActivateDef => {
                let schema_def = self.clone();
                if let Err(e) = schema_def.clone().activate() {
                    return Err(SchemaDefError {
                        message: e.to_string(),
                        code: 400,
                    });
                }
                Ok(vec![SchemaDefEvent::DefActivated])
            }
            SchemaDefCommand::DeactivateDef => {
                let schema_def = self.clone();
                let schema_def = schema_def.clone().de_activate();
                Ok(vec![SchemaDefEvent::DefDeactivated])
            }
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            SchemaDefEvent::DefCreated { id, schema } => {
                self.schema = schema;
                self.id = id;
                self.status = Status::Inactive;
            }

            SchemaDefEvent::DefValidated => {
                self.status = Status::Valid;
            }
            SchemaDefEvent::DefActivated => {
                self.status = Status::Active;
            }
            SchemaDefEvent::DefDeactivated => {
                self.status = Status::Inactive;
            }
            SchemaDefEvent::DefCreatedAndValidated { id, schema } => {
                self.schema = schema;
                self.id = id;
                self.status = Status::Valid;
            }
        }
    }
}