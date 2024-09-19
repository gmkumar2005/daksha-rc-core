use std::fmt;
use serde::{Deserialize, Serialize};
use regex::Regex;

fn remove_spaces_and_returns(input: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(input, " ").replace("\n", "").replace("\r", "").to_string()
}
#[derive(Debug, Serialize, Deserialize)]
pub enum SchemaDefCommand {
    CreateDef { id: String, schema: String },
    ValidateDef { id: String },
    ActivateDef { id: String },
    CreateAndValidateDef { id: String, schema: String },
    DeactivateDef { id: String },
}


impl fmt::Display for SchemaDefCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaDefCommand::CreateDef { id, schema } => {
                write!(f, "CreateDef {{ id: {}, schema: {} }}", id, remove_spaces_and_returns(schema))
            }
            SchemaDefCommand::ValidateDef { id } => {
                write!(f, "ValidateDef {{ id: {} }}", id)
            }
            SchemaDefCommand::ActivateDef { id } => {
                write!(f, "ActivateDef {{ id: {} }}", id)
            }
            SchemaDefCommand::CreateAndValidateDef { id, schema } => {
                write!(f, "CreateAndValidateDef {{ id: {}, schema: {} }}", id, remove_spaces_and_returns(schema))
            }
            SchemaDefCommand::DeactivateDef { id } => {
                write!(f, "DeactivateDef {{ id: {} }}", id)
            }
        }
    }
}