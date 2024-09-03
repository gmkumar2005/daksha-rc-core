use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Schema {
    Simple {
        #[serde(rename = "type")]
        prop_type: String,
        #[serde(rename = "title")]
        title: String,
        required: Vec<String>,
        properties: HashMap<String, SchemaProperty>,
        definitions: HashMap<String, Definition>,
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct Entity {
    definitions: HashMap<String, Definition>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Definition {
    pub required: Vec<String>,
    pub properties: HashMap<String, Property>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SchemaProperty {
    Simple {
        #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
        ref_id: Option<String>,
    }
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Property {
    Simple {
        #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(rename = "type")]
        prop_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>,
        #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
        max_length: Option<u32>,
        #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
        min_length: Option<u32>,
        #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
        enum_array: Option<Vec<String>>,
    },
    Nested {
        #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
        prop_type: Option<String>,
        required: Vec<String>,
        properties: HashMap<String, Property>,
    },
    Array {
        #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
        prop_type: Option<String>,
        items: Box<Property>,
    },
}