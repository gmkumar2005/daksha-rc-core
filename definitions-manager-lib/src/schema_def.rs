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

    pub fn validate(&mut self) -> Result<(), String> {
        let schema_value: Value = serde_json::from_str(&self.schema)
            .map_err(|e| format!("Invalid JSON schema: {}", e))?;

        JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_value)
            .map_err(|e| format!("Schema compilation error: {:?}", e))?;

        self.status = Status::Valid;
        Ok(())
    }

    pub fn activate(&mut self) -> Result<(), String> {
        if self.status == Status::Valid {
            self.status = Status::Active;
            Ok(())
        } else {
            Err("SchemaDoc must be valid before activation".into())
        }
    }
}
