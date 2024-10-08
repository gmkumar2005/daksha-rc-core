/// TODO 1. Create offset_store
/// 1. Write Repository api for offset store
/// 2. Write a SimpleReadSideActor which can send messages to any subscriber
/// 3. Write a LoggingActor which receives messages and logs them

use crate::schema_def::SchemaDef;
use actix::prelude::*;
use async_trait::async_trait;
use cqrs_es::{EventEnvelope, Query};
use mockall::predicate::*;
use mockall::*;
use std::sync::Arc;


pub struct SimpleReadSideProcessor {
    event_processor: Addr<EventProcessorActor>,
}


#[async_trait]
impl Query<SchemaDef> for SimpleReadSideProcessor {
    async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<SchemaDef>]) {
        let process_events = ProcessEvents {
            aggregate_id: aggregate_id.to_string(),
            events: events.to_vec(),
        };
        self.event_processor.send(process_events).await.unwrap();
    }
}


pub struct ProcessEvents {
    pub aggregate_id: String,
    pub events: Vec<EventEnvelope<SchemaDef>>,
}

impl Message for ProcessEvents {
    type Result = ();
}

pub struct GetOffsetCount;

impl Message for GetOffsetCount {
    type Result = u64;
}

impl Handler<GetOffsetCount> for EventProcessorActor {
    type Result = u64;

    fn handle(&mut self, _: GetOffsetCount, _: &mut Self::Context) -> u64 {
        self.offset_count
    }
}

#[automock]
#[async_trait]
pub trait OffsetStoreRepository {
    async fn update_offset(&self, new_offset: u64);
    async fn get_offset(&self) -> u64;
}

pub struct InMemOffsetStoreRepository {
    pub offset_count: u64,
    pub threshold: u64,
}

pub struct EventProcessorActor {
    offset_store: Arc<dyn OffsetStoreRepository>,
    offset_count: u64,
    offset_threshold: u64,
}

impl EventProcessorActor {
    pub async fn new(offset_store: Arc<dyn OffsetStoreRepository>, offset_threshold: u64) -> Self {
        let offset_count = {
            let count = offset_store.get_offset().await;
            if count == 0 { 1 } else { count }
        };
        Self {
            offset_store,
            offset_count,
            offset_threshold,
        }
    }
}

impl Actor for EventProcessorActor {
    type Context = Context<Self>;
}

impl Handler<ProcessEvents> for EventProcessorActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: ProcessEvents, _: &mut Self::Context) -> ResponseFuture<()> {
        let event_count = msg.events.len() as u64;

        let current_offset = self.offset_count;
        let offset_store = Arc::clone(&self.offset_store);
        if self.offset_count % self.offset_threshold == 0 {
            self.offset_count += event_count;
            Box::pin(async move {
                offset_store.update_offset(current_offset).await;
            })
        } else {
            self.offset_count += event_count;
            Box::pin(async {})
        }


    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema_def_events::SchemaDefEvent;
    use cqrs_es::EventEnvelope;
    use hamcrest2::prelude::*;
    use std::collections::HashMap;
    use tokio::task::LocalSet;
    #[tokio::test]
    async fn test_dispatch() {
        // Create a LocalSet
        let local = LocalSet::new();

        // Run the test within the LocalSet
        local.run_until(async {
            let mut mock_repo = MockOffsetStoreRepository::new();
            mock_repo.expect_update_offset().times(1).returning(|_| ());
            mock_repo.expect_get_offset().times(1).returning(|| 0);

            // Create a mock OffsetStoreRepository
            let offset_store = Arc::new(mock_repo);
            let event_processor = EventProcessorActor::new(offset_store,2).await.start();
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