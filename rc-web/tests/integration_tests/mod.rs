use definitions_core::definitions_domain::{generate_id_from_title, CreateDefinitionCmd, ValidateDefinitionCmd, UpdateDefinitionCmd, DomainEvent};
use chrono::{DateTime, Utc};

#[cfg(feature = "integration_tests")]
mod simple_contaner_based_test;
#[cfg(feature = "integration_tests")]
mod definitions_domain_integration_tests;

pub fn create_def_cmd_1() -> CreateDefinitionCmd {
    CreateDefinitionCmd {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_by: "test_created_by".to_string(),
        json_schema_string: get_valid_json_string(),
    }
}

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

pub fn get_updated_json_string_test_title() -> String {
    r###"
        {
            "title": "test_title",
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

pub fn get_created_at() -> DateTime<Utc> {
    let date_str = "2024-11-22T16:46:51.757980Z";
    date_str.parse().expect("Failed to parse date")
}

pub fn get_validate_def_cmd() -> ValidateDefinitionCmd {
    ValidateDefinitionCmd {
        id: generate_id_from_title("test_title"),
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
    }
}

pub fn get_update_def_cmd_mutate() -> UpdateDefinitionCmd {
    UpdateDefinitionCmd {
        id: generate_id_from_title("test_title"),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        updated_by: "test_updated_by".to_string(),
        json_schema_string: get_updated_json_string(),
    }
}

pub fn get_update_title_def_cmd() -> UpdateDefinitionCmd {
    UpdateDefinitionCmd {
        id: generate_id_from_title("test_title"),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        updated_by: "test_updated_by".to_string(),
        json_schema_string: get_updated_json_string_test_title(),
    }
}

pub fn def_created_valid_json_draft() -> DomainEvent {
    DomainEvent::DefCreated {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: get_valid_json_string(),
    }
}

pub fn def_validated_valid_json() -> DomainEvent {
    DomainEvent::DefValidated {
        id: generate_id_from_title("test_title"),
        validated_at: get_created_at(),
        validated_by: "test_user".to_string(),
        validation_result: "Success".to_string(),
    }
}

pub fn def_activated_valid_json() -> DomainEvent {
    DomainEvent::DefActivated {
        id: generate_id_from_title("test_title"),
        activated_at: get_created_at(),
        activated_by: "".to_string(),
        json_schema_string: get_valid_json_string(),
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

pub fn get_def_created_invalid_json() -> DomainEvent {
    DomainEvent::DefCreated {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: r###"
        {
            "title":  ,
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###.to_string(),
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

pub fn get_expected_validation_success() -> DomainEvent {
    DomainEvent::DefValidated {
        id: generate_id_from_title("test_title"),
        validated_at: get_created_at(),
        validated_by: "test_validated_by".to_string(),
        validation_result: "Success".to_string(),
    }
}

pub fn get_def_created_empty_title() -> DomainEvent {
    DomainEvent::DefCreated {
        id: generate_id_from_title("test_title"),
        title: "test_title".to_string(),
        definitions: vec!["test_def".to_string()],
        created_at: get_created_at(),
        created_by: "test_created_by".to_string(),
        json_schema_string: r###"
        {
            "title": "",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###.to_string(),
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
