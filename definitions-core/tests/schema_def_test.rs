use definitions_core::schema_def::*;
use hamcrest2::prelude::*;
use std::fs::File;
use std::io::Read;
#[cfg(test)]
mod tests {
    use super::*;

    fn load_contents_from_file(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }
    #[tokio::test]
    async fn test_schema_def_initialization() {
        let schema = r###"
        {
            "title": "example_schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###
            .to_string();
        let schema_doc = SchemaDef::new("example_schema".to_string(), schema.clone()).unwrap();
        assert_that!(schema_doc.os_id, is(equal_to("example_schema")));
        assert_that!(schema_doc.title, is(equal_to("example_schema")));
        assert_that!(schema_doc.schema, is(equal_to(schema)));
        assert_that!(schema_doc.status, is(equal_to(Status::Inactive)));
    }

    #[tokio::test]
    async fn test_schema_def_validation() {
        let schema = r###"
        {
            "title": "example_schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###
            .to_string();

        let schema_doc = SchemaDef::new("example_schema".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let schema_doc = schema_doc.validate_def().expect("Schema validation failed");
        assert_eq!(schema_doc.status, Status::Valid);
    }

    #[tokio::test]
    async fn test_schema_def_activation() {
        let schema = r###"
        {
            "title": "example_schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###
            .to_string();

        let schema_doc = SchemaDef::new("example_schema".to_string(), schema).unwrap();
        let schema_doc = schema_doc.validate_def().expect("Schema validation failed");
        assert_eq!(schema_doc.status, Status::Valid);
        let schema_doc = schema_doc.activate().expect("Schema activation failed");
        assert_eq!(schema_doc.status, Status::Active);
    }

    #[tokio::test]
    async fn test_schema_def_activation_without_validation() {
        let schema = r###"
        {
            "title": "example_schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###.to_string();

        let schema_doc = SchemaDef::new("example_schema".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let result = schema_doc.clone().activate();
        assert!(result.is_err());
        assert_eq!(
            "SchemaDoc must be valid before activation; cannot move status from Inactive to Active".to_string(),
            result.err().unwrap()
        );
        assert_eq!(schema_doc.status, Status::Inactive);
    }

    #[tokio::test]
    async fn test_schema_def_validation_institute() {
        let schema = load_contents_from_file("tests/resources/schemas/Institute.json").unwrap();

        let schema_doc = SchemaDef::new("institute".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let schema_doc = schema_doc.validate_def().expect("Schema validation failed");
        assert_eq!(schema_doc.status, Status::Valid);
    }
    #[tokio::test]
    async fn test_schema_def_validation_student() {
        let schema = load_contents_from_file("tests/resources/schemas/student.json").unwrap();
        let schema_doc = SchemaDef::new("student".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let schema_doc = schema_doc.validate_def().expect("Schema validation failed");
        assert_eq!(schema_doc.status, Status::Valid);
    }
    #[tokio::test]
    async fn test_schema_def_validation_teacher() {
        let schema = load_contents_from_file("tests/resources/schemas/teacher.json").unwrap();
        let schema_doc = SchemaDef::new("teacher".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let schema_doc = schema_doc.validate_def().expect("Schema validation failed");
        assert_eq!(schema_doc.status, Status::Valid);
    }

    #[tokio::test]
    async fn test_schema_def_validation_not_a_json() {
        let schema_not_a_json = r###"
            This is not a json file. It is a text file. It is not a valid json
             "###.to_string();

        let result = SchemaDef::new("1".to_string(), schema_not_a_json);
        assert_that!(result.clone(), err());
        let error_message = result.err().unwrap();
        assert_that!(error_message, matches_regex(r".*Invalid JSON schema.*"));
    }

    #[tokio::test]
    async fn test_schema_def_no_title_schema() {
        let schema = load_contents_from_file("tests/resources/schemas/not_title.json").unwrap();
        let result = SchemaDef::new("1".to_string(), schema);
        assert_that!(result.clone(), err());
        let error_message = result.err().unwrap();
        assert_that!(
            error_message,
            matches_regex(r".*Title not found in schema.*")
        );
    }


    #[tokio::test]
    async fn test_validate_record() {
        let schema = r###"
            {
                "title": "example_schema",
                "type": "object",
                "properties": {
                    "example": {
                        "type": "string"
                    }
                },
                "required": ["example"]
            }
            "###.to_string();

        let schema_doc = SchemaDef::new("example_schema".to_string(), schema).unwrap();
        let valid_record = r###"{ "example": "test" }"###;
        let invalid_record = r###"{ "example": 123 }"###;
        let missing_field_record = r###"{ }"###;

        // Test valid record
        let result = schema_doc.validate_record(valid_record);
        assert!(result.is_ok());

        // Test invalid record
        let result = schema_doc.validate_record(invalid_record);
        assert!(result.is_err());
        let error_message: Vec<String> = result.err().unwrap().collect();
        assert_that!(&*error_message, contains("123 is not of type \"string\"".to_string()));

        // Test record with missing required field
        let result = schema_doc.validate_record(missing_field_record);
        assert!(result.is_err());
        let error_message: Vec<String> = result.err().unwrap().collect();
        assert_that!(&*error_message, contains("\"example\" is a required property".to_string()));
    }

    #[tokio::test]
    async fn test_validate_record_student1() {
        let schema_with_out_references = r###"
            {
          "$schema": "http://json-schema.org/draft-07/schema",
          "type": "object",
          "properties": {
            "Student": {
              "$ref": "#/definitions/Student"
            }
          },
          "required": [
            "Student"
          ],
          "title": "student",
          "definitions": {
            "Student": {
              "$id": "#/properties/Student",
              "type": "object",
              "title": "The Student Schema",
              "required": [],
              "properties": {
                "identityDetails": {
                  "type": "object",
                  "title": "Identity Details",
                  "description": "Identity Details",
                  "required": [],
                  "properties": {
                    "fullName": {
                      "$id": "#/properties/fullName",
                      "type": "string",
                      "title": "Full name"
                    },
                    "gender": {
                      "$id": "#/properties/gender",
                      "type": "string",
                      "enum": [
                        "Male",
                        "Female",
                        "Other"
                      ],
                      "title": "Gender"
                    },
                    "dob": {
                      "$id": "#/properties/dob",
                      "type": "string",
                      "format": "date",
                      "title": "DOB"
                    },
                    "identityHolder": {
                      "type": "object",
                      "properties": {
                        "type": {
                          "$id": "#/properties/type",
                          "type": "string",
                          "$comment": "Nationality",
                          "title": "ID Type",
                          "enum": [
                            "AADHAR",
                            "PAN",
                            "LICENSE",
                            "OTHER"
                          ]
                        },
                        "value": {
                          "$id": "#/properties/value",
                          "type": "string",
                          "$comment": "Nationality",
                          "title": "ID Value"
                        }
                      }
                    }
                  }
                },
                "contactDetails": {
                  "type": "object",
                  "title": "Contact Details",
                  "description": "Contact Details",
                  "required": [],
                  "properties": {
                    "email": {
                      "$id": "#/properties/email",
                      "type": "string",
                      "title": "Email"
                    },
                    "mobile": {
                      "$id": "#/properties/mobile",
                      "type": "string",
                      "title": "Mobile"
                    },
                    "address": {
                      "$id": "#/properties/address",
                      "type": "string",
                      "title": "Address"
                    }
                  }
                }
              }
            }
          },
          "_osConfig": {
            "osComment": [
              "This section contains the OpenSABER specific configuration information",
              "privateFields: Optional; list of field names to be encrypted and stored in database",
              "signedFields: Optional; list of field names that must be pre-signed",
              "indexFields: Optional; list of field names used for creating index. Enclose within braces to indicate it is a composite index. In this definition, (serialNum, studentCode) is a composite index and studentName is a single column index.",
              "uniqueIndexFields: Optional; list of field names used for creating unique index. Field names must be different from index field name",
              "systemFields: Optional; list of fields names used for system standard information like created, updated timestamps and userid"
            ],
            "privateFields": [
              "$.identityDetails.dob",
              "$.identityDetails.identityType",
              "$.identityDetails.identityValue"
            ],
            "internalFields": [
              "$.contactDetails.email",
              "$.contactDetails.mobile",
              "$.contactDetails.address"
            ],
            "signedFields": [],
            "indexFields": [
              "studentName"
            ],
            "uniqueIndexFields": [
              "identityValue"
            ],
            "systemFields": [
              "_osCreatedAt",
              "_osUpdatedAt",
              "_osCreatedBy",
              "_osUpdatedBy",
              "_osAttestedData",
              "_osClaimId",
              "_osState"
            ],
            "attestationAttributes": [
              "educationDetails",
              "nationalIdentifier"
            ],
            "attestationPolicies": [
              {
                "name": "attestationEducationDetails",
                "properties": [
                  "educationDetails/[]"
                ],
                "paths": [
                  "$.educationDetails[?(@.osid == 'PROPERTY_ID')]['instituteName', 'program', 'graduationYear', 'marks']",
                  "$.identityDetails['fullName']"
                ],
                "type": "MANUAL",
                "attestorEntity": "Teacher",
                "attestorPlugin": "did:internal:Claim?entity=Teacher",
                "conditions": "(ATTESTOR#$.experience.[*].instituteOSID#.contains(REQUESTER#$.instituteOSID#) && ATTESTOR#$.experience[?(@.instituteOSID == REQUESTER#$.instituteOSID#)]['_osState']#.contains('PUBLISHED'))"
              }
            ],
            "autoAttestationPolicies": [
              {
                "parentProperty": "identityDetails",
                "property": "identityHolder",
                "nodeRef": "$.identityDetails.identityHolder",
                "valuePath": "$.identityDetails.identityHolder.value",
                "typePath": "$.identityDetails.identityHolder.type"
              }
            ],
            "subjectJsonPath": "mobile",
            "ownershipAttributes": [
              {
                "email": "/contactDetails/email",
                "mobile": "/contactDetails/mobile",
                "userId": "/contactDetails/mobile"
              }
            ],
            "inviteRoles": [
              "anonymous"
            ],
            "roles": [
              "anonymous"
            ]
          }
        }
    "###.to_string();

        let valid_student_record = r###"
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
            "###.to_string();
        let schema_doc = SchemaDef::new("student".to_string(), schema_with_out_references).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let schema_doc = schema_doc.validate_def().expect("Schema validation failed");
        assert_eq!(schema_doc.status, Status::Valid);
        let result = schema_doc.validate_record(&valid_student_record);
        assert!(result.is_ok());
    }
}
