use std::fmt;
use serde::{Deserialize, Serialize};
use regex::Regex;

fn remove_spaces_and_returns(input: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(input, " ").replace("\n", "").replace("\r", "").to_string()
}
#[derive(Debug, Serialize, Deserialize)]
pub enum SchemaDefCommand {
    CreateDef { os_id: String, schema: String },
    ValidateDef { os_id: String },
    ActivateDef { os_id: String },
    CreateAndValidateDef { os_id: String, schema: String },
    DeactivateDef { os_id: String },
}


impl fmt::Display for SchemaDefCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaDefCommand::CreateDef { os_id: id, schema } => {
                write!(f, "CreateDef {{ id: {}, schema: {} }}", id, remove_spaces_and_returns(schema))
            }
            SchemaDefCommand::ValidateDef { os_id: id } => {
                write!(f, "ValidateDef {{ id: {} }}", id)
            }
            SchemaDefCommand::ActivateDef { os_id: id } => {
                write!(f, "ActivateDef {{ id: {} }}", id)
            }
            SchemaDefCommand::CreateAndValidateDef { os_id: id, schema } => {
                write!(f, "CreateAndValidateDef {{ id: {}, schema: {} }}", id, remove_spaces_and_returns(schema))
            }
            SchemaDefCommand::DeactivateDef { os_id: id } => {
                write!(f, "DeactivateDef {{ id: {} }}", id)
            }
        }
    }
}