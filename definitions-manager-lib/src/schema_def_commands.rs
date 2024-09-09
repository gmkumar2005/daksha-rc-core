use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub enum SchemaDefCommand {
    CreateDef { id: String, schema: String },
    ValidateDef,
    ActivateDef,
    CreateAndValidateDef { id: String, schema: String },
    DeactivateDef,
}



