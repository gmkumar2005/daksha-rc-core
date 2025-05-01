pub mod test_harness;

// #[cfg(test)]
use chrono::{DateTime, Utc};
use definitions_core::definitions_domain::DomainEvent::DefUpdated;
use definitions_core::definitions_domain::{
    generate_id_from_title, CreateDefinition, DomainEvent, UpdateDefinition, ValidateDefinition,
};
use definitions_core::registry_domain::CreateEntityCmd;
use uuid::Uuid;

pub fn get_valid_json_string() -> String {
    r###"
        {
            "title": "test_title",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###
    .to_string()
}

pub fn get_valid_student_schema_string() -> String {
    r###"
        {
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Student",
  "type": "object",
  "properties": {
    "Student": {
      "type": "object",
      "properties": {
        "identityDetails": {
          "type": "object",
          "properties": {
            "fullName": {
              "type": "string"
            },
            "gender": {
              "type": "string",
              "enum": ["Male", "Female", "Other"]
            }
          },
          "required": ["fullName", "gender"]
        },
        "contactDetails": {
          "type": "object",
          "properties": {
            "email": {
              "type": "string",
              "format": "email"
            },
            "address": {
              "type": "string"
            }
          },
          "required": ["email", "address"]
        }
      }
    }
  },
  "required": ["Student"]
}

        "###
    .to_string()
}
pub fn get_updated_json_string() -> String {
    r###"
        {
            "title": "example_schema",
            "type": "object",
            "properties": {
                "example1": {
                    "type": "string"
                }
            }
        }
        "###
    .to_string()
}

pub fn get_json_string_empty_title() -> String {
    r###"
        {
            "title": "",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###
    .to_string()
}
pub fn get_in_valid_json_string() -> String {
    r###"
        {
            "title":  ,
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###
    .to_string()
}
pub fn get_created_at() -> DateTime<Utc> {
    let date_str = "2024-11-22T16:46:51.757980Z";
    date_str.parse().expect("Failed to parse date")
}
pub fn create_def_cmd_1() -> CreateDefinition {
    CreateDefinition {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_by: "test_created_by".to_string(),
        json_schema_string: get_valid_json_string(),
    }
}

pub fn get_expected_def_created() -> Vec<DomainEvent> {
    vec![get_expected_def_created_simple()]
}
pub fn get_expected_def_created_simple() -> DomainEvent {
    DomainEvent::DefCreated {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_valid_json_string(),
    }
}

pub fn get_def_created_invalid_json() -> DomainEvent {
    DomainEvent::DefCreated {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_in_valid_json_string(),
    }
}
pub fn get_def_created_valid_json() -> DomainEvent {
    DomainEvent::DefCreated {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_valid_json_string(),
    }
}

pub fn get_def_created_valid_student_json() -> DomainEvent {
    DomainEvent::DefCreated {
        id: generate_id_from_title("Student"),
        title: "Student".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_valid_student_schema_string(),
    }
}

pub fn get_def_validated_valid_student_json() -> DomainEvent {
    DomainEvent::DefValidated {
        id: generate_id_from_title("Student"),
        validated_at: Utc::now(),
        validated_by: "test_user".to_string(),
        validation_result: "Success".to_string(),
    }
}

pub fn get_def_activated_valid_student_json() -> DomainEvent {
    DomainEvent::DefActivated {
        id: generate_id_from_title("Student"),
        activated_at: Utc::now(),
        activated_by: "".to_string(),
    }
}
pub fn get_def_created_empty_title() -> DomainEvent {
    DomainEvent::DefCreated {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_json_string_empty_title(),
    }
}
pub fn get_validate_def_cmd() -> ValidateDefinition {
    ValidateDefinition {
        id: generate_id_from_title("test_title"),
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
    }
}

pub fn get_update_def_cmd_mutate() -> UpdateDefinition {
    UpdateDefinition {
        id: generate_id_from_title("test_title"),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        updated_by: "test_updated_by".to_string(),
        json_schema_string: get_updated_json_string(),
    }
}

pub fn get_update_def_cmd() -> UpdateDefinition {
    UpdateDefinition {
        id: generate_id_from_title("test_title"),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        updated_by: "test_updated_by".to_string(),
        json_schema_string: get_updated_json_string(),
    }
}
pub fn get_expected_def_created_empty_json() -> DomainEvent {
    DomainEvent::DefCreated {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: "".to_string(),
    }
}

pub fn get_expected_validation_failed() -> DomainEvent {
    DomainEvent::DefValidatedFailed {
        id: generate_id_from_title("test_title"),
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
        validation_result: "failure".to_string(),
        validation_errors: vec!["Invalid Schema: Schema is empty".to_string()],
    }
}
pub fn get_expected_validation_failed_invalid_json() -> DomainEvent {
    DomainEvent::DefValidatedFailed {
        id: generate_id_from_title("test_title"),
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
        validation_result: "failure".to_string(),
        validation_errors: vec!["Invalid Json: expected value at line 3 column 23".to_string()],
    }
}

pub fn get_expected_validation_failed_empty_title() -> DomainEvent {
    DomainEvent::DefValidatedFailed {
        id: generate_id_from_title("test_title"),
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
        validation_result: "failure".to_string(),
        validation_errors: vec!["Invalid Schema: Title is empty".to_string()],
    }
}

pub fn get_expected_validation_success() -> DomainEvent {
    DomainEvent::DefValidated {
        id: generate_id_from_title("test_title"),
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
        validation_result: "Success".to_string(),
    }
}

pub fn get_expected_def_updated() -> DomainEvent {
    DefUpdated {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        updated_by: "test_updated_by".to_string(),
        json_schema_string: get_updated_json_string(),
    }
}

pub fn get_expected_def_created_valid_json() -> DomainEvent {
    DomainEvent::DefCreated {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_valid_json_string(),
    }
}

pub fn get_valid_student_document() -> String {
    r###"
{
  "Student": {
    "identityDetails":{
      "fullName":"John",
      "gender":"Male"
    },
    "contactDetails":{
      "email":"abc@abc.com",
      "address":"line1"
    }
  }
}
"###
    .to_string()
}
pub fn get_create_entity_cmd() -> CreateEntityCmd {
    CreateEntityCmd {
        id: Uuid::now_v7(),
        entity_body: get_valid_student_document(),
        entity_type: "Student".to_string(),
        created_by: "test_user".to_string(),
    }
}

pub fn get_create_entity_cmd_with_invalid_student() -> CreateEntityCmd {
    let invalid_student_document = r###"
{
  "Student": {
    "identityDetails":{
      "fullName":100,
      "gender":"Child"
    },
    "contactDetails":{
      "email":"abc@abc.com",
      "address":"line1"
    }
  }
}
"###
    .to_string();

    CreateEntityCmd {
        id: Uuid::now_v7(),
        entity_body: invalid_student_document,
        entity_type: "Student".to_string(),
        created_by: "test_user".to_string(),
    }
}
