use jsonschema::{Draft, JSONSchema};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Inactive,
    Valid,
    Active,
    Invalid,
}

#[derive(Debug, Clone)]
pub struct SchemaDef {
    pub id: String,
    pub title: String,
    pub version: u32,
    pub schema: String,
    pub status: Status,
}

impl SchemaDef {
    pub fn new(id: String, schema: String) -> Result<Self, String> {
        let schema_value: Value =
            serde_json::from_str(&schema).map_err(|e| format!("Invalid JSON schema: {}", e))?;

        let title = schema_value["title"]
            .as_str()
            .ok_or("Title not found in schema")?
            .to_string();

        Ok(Self {
            id,
            title,
            version: 1,
            schema,
            status: Status::Inactive,
        })
    }

    pub fn validate_def(self) -> Result<Self, String> {
        let schema_value: Value = serde_json::from_str(&self.schema)
            .map_err(|e| format!("Invalid JSON schema: {}", e))?;

        JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_value)
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
            Err("SchemaDoc must be valid before activation".into())
        }
    }

    pub fn validate_record<'a>(&'a self, json_record: &'a str) -> Result<(), Box<dyn Iterator<Item = String> + 'a>> {
        // Check if the json_record is a valid JSON
        let instance_val: Value = serde_json::from_str(json_record)
            .map_err(|e| Box::new(std::iter::once(format!("Invalid JSON record: {}", e))) as Box<dyn Iterator<Item = String>>)?;

        // Check if the json_record conforms to the JSON schema
        let schema_val: Value = serde_json::from_str(&self.schema)
            .map_err(|e| Box::new(std::iter::once(format!("Invalid JSON schema: {}", e))) as Box<dyn Iterator<Item = String>>)?;

        let compiled_schema_val = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_val)
            .map_err(|e| Box::new(std::iter::once(format!("Schema compilation error: {:?}", e))) as Box<dyn Iterator<Item = String>>)?;

        let validation_result = match compiled_schema_val.validate(&instance_val) {
            Ok(_) => Ok(()),
            Err(errors) => {
                let error_messages: Vec<String> = errors.map(|e| e.to_string()).collect();
                Err(Box::new(error_messages.into_iter()) as Box<dyn Iterator<Item = String>>)
            }
        };

        validation_result
    }


}