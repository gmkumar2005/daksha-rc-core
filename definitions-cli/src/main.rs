use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader};
use definitions_manager_lib::definitions_manager::{Definition, Property, Schema, SchemaProperty};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut address_properties = HashMap::new();
    address_properties.insert("line1".to_string(), Property::Simple { id: None, prop_type: "string".to_string(), format: None, max_length: None, min_length: None, enum_array: None });
    address_properties.insert("pincode".to_string(), Property::Simple { id: None, prop_type: "string".to_string(), format: None, max_length: Some(6), min_length: Some(6), enum_array: None });

    let address = Property::Nested {
        prop_type: None,
        required: vec!["line1".to_string(), "pincode".to_string()],
        properties: address_properties,
    };
    let mut education_properties = HashMap::new();
    education_properties.insert("title".to_string(), Property::Simple { id: None, prop_type: "string".to_string(), format: None, max_length: None, min_length: None, enum_array: None });
    education_properties.insert("fromDate".to_string(), Property::Simple { id: None, prop_type: "string".to_string(), format: Some("date".to_string()), max_length: None, min_length: None, enum_array: None });
    education_properties.insert("toDate".to_string(), Property::Simple { id: None, prop_type: "string".to_string(), format: Some("date".to_string()), max_length: None, min_length: None, enum_array: None });

    let education = Property::Array {
        prop_type: Some("array".to_string()),
        items: Box::new(Property::Nested {
            prop_type: Some("object".to_string()),
            required: vec!["title".to_string(), "fromDate".to_string()],
            properties: education_properties,
        }),
    };

    let mut student_properties = HashMap::new();
    student_properties.insert("name".to_string(), Property::Simple { id: None, prop_type: "string".to_string(), format: None, max_length: None, min_length: None, enum_array: None });
    student_properties.insert("dob".to_string(), Property::Simple { id: None, prop_type: "string".to_string(), format: Some("date".to_string()), max_length: None, min_length: None, enum_array: Some(vec!["title".to_string(), "fromDate".to_string()]) });
    student_properties.insert("address".to_string(), address);
    student_properties.insert("education".to_string(), education);

    let student_definition = Definition {
        required: vec!["name".to_string(), "dob".to_string()],
        properties: student_properties,
    };

    let mut definitions = HashMap::new();
    definitions.insert("Student".to_string(), student_definition);

    let mut schema_properties = HashMap::new();
    schema_properties.insert("Student".to_string(), SchemaProperty::Simple { ref_id: Some("#/definitions/Student".to_string()) });

    let student_schema = Schema::Simple {
        prop_type: "object".to_string(),
        required: vec!["Student".to_string()],
        properties: schema_properties,
        definitions: definitions,
        title: "Student".to_string(),
    };
    let json = serde_json::to_string(&student_schema);
    println!("Object tree");
    println!("{}", json.unwrap());
    println!("Begin deserializing");
    let file = File::open("../tests/resources/student.json")?;
    let reader = BufReader::new(file);

    // Deserialize the JSON data into the Schema struct
    let student_deserialized: Schema = serde_json::from_reader(reader)?;
    println!("{:#?}", student_deserialized);
    let deserialized_json = serde_json::to_string(&student_deserialized);
    println!("deserialized_json");
    println!("{}", deserialized_json.unwrap());
    Ok(())
}
