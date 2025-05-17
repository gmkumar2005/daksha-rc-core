mod common;
#[cfg(test)]
mod birth_certificate_tests {
    use crate::common::get_created_at;
    use crate::common::test_harness::SimpleTestHarness;
    use chrono::Utc;
    use definitions_core::definitions_domain::{generate_id_from_title, DomainEvent};
    use definitions_core::registry_domain::{CreateEntityCmd, ModifyEntityCmd};
    use std::fs;
    use std::path::Path;
    use uuid::Uuid;

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        // let _ = env_logger::builder().is_test(true).try_init();
        env_logger::init();
    }

    pub fn repeatable_uuid_stting() -> String {
        "0196910b-93fc-793f-8f8f-1f4ea6b949df".to_string()
    }
    pub fn repeatable_id_uuid() -> Uuid {
        Uuid::parse_str(repeatable_uuid_stting().as_str()).unwrap()
    }
    pub fn read_birth_certificate_schema() -> std::io::Result<String> {
        let path = Path::new("tests/resources/schemas/BirthCertificate_Schema.json");
        fs::read_to_string(path)
    }
    pub fn create_birth_certificate_entity_cmd() -> CreateEntityCmd {
        CreateEntityCmd {
            id: Uuid::now_v7(),
            entity_body: valid_birth_certificate_entity_json(),
            entity_type: "BirthCertificate".to_string(),
            created_by: "Admin".to_string(),
        }
    }

    pub fn def_created_valid_birth_certificate_event() -> DomainEvent {
        DomainEvent::DefCreated {
            id: generate_id_from_title("BirthCertificate"),
            title: "BirthCertificate".to_string(),
            definitions: vec!["BirthCertificate".to_string()],
            created_at: get_created_at(),
            created_by: "Admin".to_string(),
            json_schema_string: read_birth_certificate_schema().unwrap(),
        }
    }

    pub fn valid_birth_certificate_entity_json() -> String {
        r###"
        {
          "BirthCertificate": {
            "contact": "+91-123321123916",
            "date_of_birth": "2022-10-28T06:00:00Z",
            "gender": "male",
            "hospital": "Apollo",
            "name": "user 1234 demo",
            "name_of_father": "Ram",
            "name_of_mother": "Lakshmi",
            "place_of_birth": "Bangalore",
            "present_address": "200, bangalore"
          }
        }
        "###
        .to_string()
    }

    pub fn def_validated_valid_birth_certificate_event() -> DomainEvent {
        DomainEvent::DefValidated {
            id: generate_id_from_title("BirthCertificate"),
            validated_at: Utc::now(),
            validated_by: "Admin".to_string(),
            validation_result: "Success".to_string(),
        }
    }

    pub fn get_def_activated_birth_certificate_event() -> DomainEvent {
        DomainEvent::DefActivated {
            id: generate_id_from_title("BirthCertificate"),
            activated_at: Utc::now(),
            activated_by: "Admin".to_string(),
            json_schema_string: read_birth_certificate_schema().unwrap(),
        }
    }
    pub fn entity_created_birth_certificate_event() -> DomainEvent {
        let id = repeatable_id_uuid();
        DomainEvent::EntityCreated {
            id,
            registry_def_id: generate_id_from_title("BirthCertificate"),
            registry_def_version: Default::default(),
            entity_body: valid_birth_certificate_entity_json(),
            entity_type: "BirthCertificate".to_string(),
            created_at: Utc::now(),
            created_by: "Admin".to_string(),
            version: Default::default(),
        }
    }

    pub fn modify_birth_certificate_entity_cmd() -> ModifyEntityCmd {
        let id = repeatable_id_uuid();
        ModifyEntityCmd {
            id,
            entity_body: valid_birth_certificate_entity_json(),
            entity_type: "BirthCertificate".to_string(),
            modified_by: "Admin".to_string(),
        }
    }
    #[test]
    fn test_create_birth_certificate_entity() {
        // env_logger::init();
        SimpleTestHarness::given([
            def_created_valid_birth_certificate_event(),
            def_validated_valid_birth_certificate_event(),
            get_def_activated_birth_certificate_event(),
        ])
        .when(create_birth_certificate_entity_cmd())
        .then_assert(|events| {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            // debug!("EntityCreated: {:#?}", event);
            if let DomainEvent::EntityCreated {
                registry_def_id: def_id,
                created_by,
                ..
            } = event
            {
                assert_eq!(def_id, &generate_id_from_title("BirthCertificate"));
                assert_eq!(created_by, "Admin");
            } else {
                assert!(
                    matches!(event, DomainEvent::EntityCreated { .. }),
                    "Event is not of type DomainEvent::EntityCreated"
                );
            }
        });
    }

    #[test]
    fn test_modify_birth_certificate_entity() {
        let modify_birth_certificate_cmd = modify_birth_certificate_entity_cmd();
        SimpleTestHarness::given([
            def_created_valid_birth_certificate_event(),
            def_validated_valid_birth_certificate_event(),
            get_def_activated_birth_certificate_event(),
            entity_created_birth_certificate_event(),
        ])
        .when(modify_birth_certificate_cmd)
        .then_assert(|events| {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            // debug!("EntityCreated: {:#?}", event);
            if let DomainEvent::EntityUpdated {
                registry_def_id: def_id,
                updated_by,
                ..
            } = event
            {
                assert_eq!(def_id, &generate_id_from_title("BirthCertificate"));
                assert_eq!(updated_by, "Admin");
            } else {
                assert!(
                    matches!(event, DomainEvent::EntityCreated { .. }),
                    "Event is not of type DomainEvent::EntityCreated"
                );
            }
        });
    }
}
