/// TODO 1. Create offset_store
/// 1. Write Repository api for offset store
/// 2. Write a SimpleReadSideActor which can send messages to any subscriber
/// 3. Write a LoggingActor which receives messages and logs them

use crate::schema_def::SchemaDef;
use actix::prelude::*;
use anyhow::Result;
use async_trait::async_trait;
use cqrs_es::{EventEnvelope, Query};
use log::warn;
use mockall::predicate::*;
use std::sync::Arc;


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


pub struct EventProcessorActor {
    offset_store: Arc<dyn ProjectionOffsetStoreRepository>,
    offset_count: u64,
    offset_threshold: u64,
    projection_name: String,
    projection_key: String,
}

impl EventProcessorActor {
    pub async fn new(offset_store: Arc<dyn ProjectionOffsetStoreRepository>, offset_threshold: u64,
                     projection_name: &str,projection_key: &str) -> Self {
        let offset_count = match offset_store.read_record(projection_name,projection_key).await {
            Ok(Some(ProjectionOffsetStore { current_offset, .. })) => {
                let parsed_offset = current_offset.parse::<u64>().unwrap_or(0);
                if parsed_offset > 0 {
                    parsed_offset
                } else {
                    0
                }
            },
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
            projection_name: projection_name.to_string(),
            projection_key: projection_key.to_string(),
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
        let projection_name = self.projection_name.to_string();
        let projection_key = self.projection_key.to_string();
        if self.offset_count % self.offset_threshold == 0 {
            self.offset_count += event_count;
            Box::pin(async move {
                offset_store.update_record(&projection_name,&projection_key,&current_offset.to_string()).await.map_err(|e| {
                    warn!("Failed to update offset: {:?}", e);
                }).ok();
            })
        } else {
            self.offset_count += event_count;
            Box::pin(async {})
        }
    }
}

/// Begin repository

#[derive(Debug, sqlx::FromRow, PartialEq, Clone, Eq)]
pub struct ProjectionOffsetStore {
    pub projection_name: String,
    pub projection_key: String,
    pub current_offset: String,
    pub manifest:String,
    pub mergeable: bool,
    pub last_updated: i64,
}
#[async_trait]
pub trait ProjectionOffsetStoreRepository{
    async fn insert_record(&self, record: ProjectionOffsetStore ) -> Result<(), anyhow::Error>;
    async fn read_record(&self, projection_name: &str, projection_key: &str) -> Result<Option<ProjectionOffsetStore>, anyhow::Error>;
    async fn update_record(&self, projection_name: &str, projection_key: &str,current_offset: &str) -> Result<(), anyhow::Error>;
}
// impl Interface for dyn ProjectionOffsetStoreRepository {}