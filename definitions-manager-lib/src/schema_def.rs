#[allow(deprecated)]
use jsonschema::{Draft, JSONSchema};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::{Validate, ValidationError};
#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq)]
pub enum Status {
    #[default]
    Inactive,
    Valid,
    Active,
    Invalid,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq,Validate)]
pub struct SchemaDef {
    pub id: String,
    pub title: String,
    pub version: u32,
    pub schema: String,
    pub status: Status,
}

fn validate_lower_snake_case(value: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^[a-z0-9]+(_[a-z0-9]+)*$").unwrap();
    if re.is_match(value) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_format"))
    }
}
fn validate_id_and_title(id: &str , title:&str) -> Result<(), ValidationError> {
    if id == title {
        Ok(())
    } else {
        Err(ValidationError::new("id_and_title_mismatch"))
    }
}

impl SchemaDef {
    pub fn new(id: String, schema: String) -> Result<Self, String> {
        validate_lower_snake_case(&id).map_err(|_e| format!("id has to be lower_case and snake_case: {}", id))?;
        let schema_value: Value =
            serde_json::from_str(&schema).map_err(|e| format!("Invalid JSON schema: {}", e))?;

        let title = schema_value["title"]
            .as_str()
            .ok_or("Title not found in schema")?
            .to_string();
        validate_id_and_title(&id, &title)
            .map_err(|_e| format!("id and title mismatch: id : {} title: {} ", id,title))?;
        Ok(Self {
            id,
            title,
            version: 1,
            schema,
            status: Status::Inactive,
        })
    }

    pub fn validate_def(self) -> Result<Self, String> {
        let schema_val: Value = serde_json::from_str(&self.schema)
            .map_err(|e| format!("Invalid JSON schema: {}", e))?;
        jsonschema::draft7::new(&schema_val)
            .map_err(|e| format!("Schema compilation error: {:?}", e))?;

        Ok(Self {
            status: Status::Valid,
            ..self
        })
    }

    pub fn activate(self) -> Result<Self, String> {
        if self.status == Status::Valid {
            Ok(Self {
                status: Status::Active,
                ..self
            })
        } else {
            Err(format!("SchemaDoc must be valid before activation; cannot move status from {:?} to {:?}", self.status, Status::Active).into())
        }
    }

    pub fn de_activate(self) -> Result<Self, String> {
        if self.status == Status::Active {
            Ok(Self {
                status: Status::Inactive,
                ..self
            })
        } else {
            Err("SchemaDoc must be active before de_activation".into())
        }
    }

    #[allow(deprecated)]
    pub fn validate_record<'a>(&'a self, json_record: &'a str) -> Result<(), Box<dyn Iterator<Item=String> + 'a>> {
        // Check if the json_record is a valid JSON
        let instance_val: Value = serde_json::from_str(json_record)
            .map_err(|e| Box::new(std::iter::once(format!("Invalid JSON record: {}", e))) as Box<dyn Iterator<Item=String>>)?;

        // Check if the json_record conforms to the JSON schema
        let schema_val: Value = serde_json::from_str(&self.schema)
            .map_err(|e| Box::new(std::iter::once(format!("Invalid JSON schema: {}", e))) as Box<dyn Iterator<Item=String>>)?;


        let compiled_schema_val = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_val)
            .map_err(|e| Box::new(std::iter::once(format!("Schema compilation error: {:?}", e))) as Box<dyn Iterator<Item=String>>)?;

        let validation_result = match compiled_schema_val.validate(&instance_val) {
            Ok(_) => Ok(()),
            Err(errors) => {
                let error_messages: Vec<String> = errors.map(|e| e.to_string()).collect();
                Err(Box::new(error_messages.into_iter()) as Box<dyn Iterator<Item=String>>)
            }
        };

        validation_result
    }
}