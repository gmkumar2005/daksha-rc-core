#[cfg(test)]
mod read_side_dispatch {
    use actix::Actor;
    use anyhow::{Error, Result};
    use async_trait::async_trait;
    use cqrs_es::{EventEnvelope, Query};
    use definitions_core::read_side_processor::ProjectionOffsetStoreRepository;
    use definitions_core::read_side_processor::{EventProcessorActor, GetOffsetCount, ProjectionOffsetStore, SimpleReadSideProcessor};
    use definitions_core::schema_def::SchemaDef;
    use definitions_core::schema_def_events::SchemaDefEvent;
    // use hamcrest2::{assert_that, equal_to, is};
    use hamcrest2::prelude::*;
    use mockall::mock;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::task::LocalSet;

    #[tokio::test]
    async fn dispatch_using_projection_offset_store() {
        mock! {
             pub ProjectionOffsetStoreRepository  {
        }
        #[async_trait]
         impl ProjectionOffsetStoreRepository  for ProjectionOffsetStoreRepository  {
            async fn insert_record(&self, record: ProjectionOffsetStore ) -> Result<(), Error>;
            async fn read_record(&self, projection_name: &str, projection_key: &str) -> Result<Option<ProjectionOffsetStore>, anyhow::Error>;
            async fn update_record(&self, projection_name: &str, projection_key: &str,current_offset: &str)-> Result<(), Error>;}
        }
        let offset_store_record_1 = ProjectionOffsetStore {
            projection_name: "test_projection".to_string(),
            projection_key: "test_key".to_string(),
            current_offset: "1".to_string(),
            manifest: "".to_string(),
            mergeable: false,
            last_updated: 0,
        };
        // Create a LocalSet
        let local = LocalSet::new();

        local.run_until(async {
            let expected_offset_record = offset_store_record_1.clone();
            let mut mock_repo = MockProjectionOffsetStoreRepository::new();
            // mock_repo.expect_insert_record().times(1).returning(|_| Ok(()));
            mock_repo.expect_read_record().times(2).returning(move |_, _| Ok(Some(offset_store_record_1.clone())));
            mock_repo.expect_update_record().times(1).returning(|_, _, _| Ok(()));
            let actual_offset_record = mock_repo
                .read_record("test_projection", "test_key")
                .await.unwrap().unwrap();
            assert_that!(actual_offset_record, is(equal_to(expected_offset_record)));
            let offset_store = Arc::new(mock_repo);
            let event_processor = EventProcessorActor::new(offset_store, 2, "SimpleProjection", "1").await.start();
            let event_processor_clone = event_processor.clone();
            let processor = SimpleReadSideProcessor { event_processor };
            let offset_count = event_processor_clone.send(GetOffsetCount).await.unwrap();
            // offset_count should be always initiated to 1
            assert_that!(offset_count, is(equal_to(1)));
            // Create a mock event
            let event_1: EventEnvelope<SchemaDef> = EventEnvelope {
                aggregate_id: "test_aggregate_1".to_string(),
                sequence: 1,
                payload: SchemaDefEvent::DefCreated {
                    os_id: "".to_string(),
                    schema: "".to_string(),
                },
                metadata: HashMap::new(),
            };
            let event_2: EventEnvelope<SchemaDef> = EventEnvelope {
                aggregate_id: "test_aggregate_2".to_string(),
                sequence: 1,
                payload: SchemaDefEvent::DefCreated {
                    os_id: "".to_string(),
                    schema: "".to_string(),
                },
                metadata: HashMap::new(),
            };
            processor.dispatch("test_aggregate", &[event_1]).await;
            let offset_count = event_processor_clone.send(GetOffsetCount).await.unwrap();
            assert_that!(offset_count, is(equal_to(2)));

            processor.dispatch("test_aggregate", &[event_2]).await;
            let offset_count = event_processor_clone.send(GetOffsetCount).await.unwrap();
            assert_that!(offset_count, is(equal_to(3)));
        }).await;
    }
}

