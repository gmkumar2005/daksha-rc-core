use definitions_core::definitions_domain::{generate_id_from_title, CreateDefinition};

mod simple_contaner_based_test;

pub fn create_def_cmd_1() -> CreateDefinition {
    CreateDefinition {
        def_id: generate_id_from_title("test_title"),
        def_title: "test_title".to_string(),
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
