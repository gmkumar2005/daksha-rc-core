// #[cfg(test)]
use chrono::{DateTime, Utc};
use definitions_core::definitions_domain::DomainEvent::DefUpdated;
use definitions_core::definitions_domain::{
    CreateDefinition, DomainEvent, UpdateDefinition, ValidateDefinition,
};

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
        def_id: 1,
        def_title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_valid_json_string(),
    }
}

pub fn get_expected_def_created() -> Vec<DomainEvent> {
    vec![get_expected_def_created_simple()]
}
pub fn get_expected_def_created_simple() -> DomainEvent {
    DomainEvent::DefCreated {
        def_id: 1,
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_valid_json_string(),
    }
}

pub fn get_def_created_invalid_json() -> DomainEvent {
    DomainEvent::DefCreated {
        def_id: 1,
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_in_valid_json_string(),
    }
}
pub fn get_def_created_valid_json() -> DomainEvent {
    DomainEvent::DefCreated {
        def_id: 1,
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_valid_json_string(),
    }
}

pub fn get_def_created_empty_title() -> DomainEvent {
    DomainEvent::DefCreated {
        def_id: 1,
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_json_string_empty_title(),
    }
}
pub fn get_validate_def_cmd() -> ValidateDefinition {
    ValidateDefinition {
        def_id: 1,
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
    }
}

pub fn get_update_def_cmd() -> UpdateDefinition {
    let schema_string_updated = r###"
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
    .to_string();
    UpdateDefinition {
        def_id: 1,
        def_title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        updated_by: "test_updated_by".to_string(),
        json_schema_string: get_updated_json_string(),
    }
}

pub fn get_expected_def_created_empty_json() -> DomainEvent {
    let date_str = "2024-11-22T16:46:51.757980Z";
    let created_at: DateTime<Utc> = date_str.parse().expect("Failed to parse date");
    DomainEvent::DefCreated {
        def_id: 1,
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: "".to_string(),
    }
}

pub fn get_expected_validation_failed() -> DomainEvent {
    DomainEvent::DefValidatedFailed {
        def_id: 1,
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
        validation_result: "failure".to_string(),
        validation_errors: vec!["Invalid Schema: Schema is empty".to_string()],
    }
}
pub fn get_expected_validation_failed_invalid_json() -> DomainEvent {
    DomainEvent::DefValidatedFailed {
        def_id: 1,
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
        validation_result: "failure".to_string(),
        validation_errors: vec!["Invalid Json: expected value at line 3 column 23".to_string()],
    }
}

pub fn get_expected_validation_failed_empty_title() -> DomainEvent {
    DomainEvent::DefValidatedFailed {
        def_id: 1,
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
        validation_result: "failure".to_string(),
        validation_errors: vec!["Invalid Schema: Title is empty".to_string()],
    }
}

pub fn get_expected_validation_success() -> DomainEvent {
    DomainEvent::DefValidated {
        def_id: 1,
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
        validation_result: "Success".to_string(),
    }
}

pub fn get_expected_def_updated() -> DomainEvent {
    DefUpdated {
        def_id: 1,
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        updated_by: "test_updated_by".to_string(),
        json_schema_string: get_updated_json_string(),
    }
}
