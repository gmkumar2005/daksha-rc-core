/// TODO 1. Create offset_store
/// 1. Write Repository api for offset store
/// 2. Write a SimpleReadSideActor which can send messages to any subscriber
/// 3. Write a LoggingActor which receives messages and logs them

use crate::schema_def::SchemaDef;
use actix::prelude::*;
use anyhow::{Result};
use async_trait::async_trait;
use cqrs_es::{EventEnvelope, Query};
use mockall::predicate::*;
use std::sync::Arc;
use log::warn;

pub struct SimpleReadSideProcessor {
    pub event_processor: Addr<EventProcessorActor>,
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

// #[automock]
#[async_trait]
pub trait OffsetStoreRepository {
    type Item;
    type Error;
    async fn update_offset(&self, new_offset: u64)-> Result<(), Self::Error>;
    async fn get_offset(&self) -> Result<Option<Self::Item>, Self::Error>;
}

pub struct EventProcessorActor {
    offset_store: Arc<dyn OffsetStoreRepository<Error=anyhow::Error, Item=u64>>,
    offset_count: u64,
    offset_threshold: u64,
}

impl EventProcessorActor {
    pub async fn new(offset_store: Arc<dyn OffsetStoreRepository<Error=anyhow::Error, Item=u64>>, offset_threshold: u64) -> Self {
        let offset_count = match offset_store.get_offset().await {
            Ok(Some(count)) if count > 0 => count,
            Ok(_) => {
                warn!("Offset count is zero or not found, defaulting to 1");
                1
            }
            Err(e) => {
                warn!("Failed to get offset: {:?}", e);
                1
            }
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
                offset_store.update_offset(current_offset).await.map_err(|e| {
                    warn!("Failed to update offset: {:?}", e);
                }).ok();
            })
        } else {
            self.offset_count += event_count;
            Box::pin(async {})
        }


    }
}

