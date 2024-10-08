#[cfg(test)]
mod tests_dispatch {
    use actix::Actor;
    use anyhow::{anyhow, Result};
    use async_trait::async_trait;
    use cqrs_es::{EventEnvelope, Query};
    use definitions_manager_lib::read_side_processor::*;
    use definitions_manager_lib::schema_def_events::SchemaDefEvent;
    use hamcrest2::prelude::*;
    use mockall::mock;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::task::LocalSet;
    #[tokio::test]
    async fn offset_count_should_increase_after_dispatch() {
        mock! {
            pub OffsetStoreRepository {
            }

            #[async_trait]
            impl OffsetStoreRepository for OffsetStoreRepository {
                type Item = u64;
                type Error =anyhow::Error;

                async fn update_offset(&self, new_offset: u64) -> Result<(), anyhow::Error>;
                async fn get_offset(&self) -> Result<Option<u64>, anyhow::Error>;
            }
    }
        // Create a LocalSet
        let local = LocalSet::new();

        // Run the test within the LocalSet
        local.run_until(async {
            let mut mock_repo = MockOffsetStoreRepository::new();
            mock_repo.expect_update_offset().times(1).returning(|_| Ok(()));
            mock_repo.expect_get_offset().times(1).returning(|| Ok(Some(0)));

            // Create a mock OffsetStoreRepository
            let offset_store = Arc::new(mock_repo);
            let event_processor = EventProcessorActor::new(offset_store, 2).await.start();
            let event_processor_clone = event_processor.clone();
            // Create an instance of SimpleReadSideProcessor
            let processor = SimpleReadSideProcessor {
                event_processor,
            };


            // Create a mock event
            let event_1 = EventEnvelope {
                aggregate_id: "test_aggregate_1".to_string(),
                sequence: 1,
                payload: SchemaDefEvent::DefCreated {
                    os_id: "".to_string(),
                    schema: "".to_string(),
                },
                metadata: HashMap::new(),
            };
            let event_2 = EventEnvelope {
                aggregate_id: "test_aggregate_2".to_string(),
                sequence: 1,
                payload: SchemaDefEvent::DefCreated {
                    os_id: "".to_string(),
                    schema: "".to_string(),
                },
                metadata: HashMap::new(),
            };
            let offset_count = event_processor_clone.send(GetOffsetCount).await.unwrap();
            assert_that!(offset_count, is(equal_to(1)));

            processor.dispatch("test_aggregate", &[event_1]).await;
            let offset_count = event_processor_clone.send(GetOffsetCount).await.unwrap();
            assert_that!(offset_count, is(equal_to(2)));

            processor.dispatch("test_aggregate", &[event_2]).await;
            let offset_count = event_processor_clone.send(GetOffsetCount).await.unwrap();
            assert_that!(offset_count, is(equal_to(3)));
        }).await;
    }

    #[tokio::test]
    async fn offset_count_should_increase_when_repo_fails() {
        mock! {
            pub OffsetStoreRepository {
            }

            #[async_trait]
            impl OffsetStoreRepository for OffsetStoreRepository {
                type Item = u64;
                type Error =anyhow::Error;

                async fn update_offset(&self, new_offset: u64) -> Result<(), anyhow::Error>;
                async fn get_offset(&self) -> Result<Option<u64>, anyhow::Error>;
            }
    }
        // Create a LocalSet
        let local = LocalSet::new();

        // Run the test within the LocalSet
        local.run_until(async {
            let mut mock_repo = MockOffsetStoreRepository::new();
            mock_repo.expect_update_offset().times(1).returning(|_| Err(anyhow!("Offset update failed")));
            mock_repo.expect_get_offset().times(1).returning(|| Err(anyhow!("Get offset failed")));

            // Create a mock OffsetStoreRepository
            let offset_store = Arc::new(mock_repo);
            let event_processor = EventProcessorActor::new(offset_store, 2).await.start();
            let event_processor_clone = event_processor.clone();
            // Create an instance of SimpleReadSideProcessor
            let processor = SimpleReadSideProcessor {
                event_processor,
            };


            // Create a mock event
            let event_1 = EventEnvelope {
                aggregate_id: "test_aggregate_1".to_string(),
                sequence: 1,
                payload: SchemaDefEvent::DefCreated {
                    os_id: "".to_string(),
                    schema: "".to_string(),
                },
                metadata: HashMap::new(),
            };
            let event_2 = EventEnvelope {
                aggregate_id: "test_aggregate_2".to_string(),
                sequence: 1,
                payload: SchemaDefEvent::DefCreated {
                    os_id: "".to_string(),
                    schema: "".to_string(),
                },
                metadata: HashMap::new(),
            };
            let offset_count = event_processor_clone.send(GetOffsetCount).await.unwrap();
            assert_that!(offset_count, is(equal_to(1)));

            processor.dispatch("test_aggregate", &[event_1]).await;
            let offset_count = event_processor_clone.send(GetOffsetCount).await.unwrap();
            assert_that!(offset_count, is(equal_to(2)));

            processor.dispatch("test_aggregate", &[event_2]).await;
            let offset_count = event_processor_clone.send(GetOffsetCount).await.unwrap();
            assert_that!(offset_count, is(equal_to(3)));
        }).await;
    }


}

