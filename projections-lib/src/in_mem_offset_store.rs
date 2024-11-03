use crate::offset_store::LastOffset;
use crate::{Offset, PersistenceId};
use actix::prelude::*;
use futures_util::stream;
use std::pin::Pin;


pub struct InMemOffsetStore {
    last_offsets: Vec<LastOffset>,
}

impl InMemOffsetStore {
    pub fn new() -> Self {
        Self {
            last_offsets: Vec::new(),
        }
    }
}

impl Actor for InMemOffsetStore {
    type Context = Context<Self>;
}

pub struct GetLastOffset;

impl Message for GetLastOffset {
    type Result = Option<LastOffset>;
}

pub struct GetOffset {
    pub persistence_id: PersistenceId,
}

impl Message for GetOffset {
    type Result = Option<Offset>;
}

pub struct SaveOffset {
    pub persistence_id: PersistenceId,
    pub offset: Offset,
}

impl Message for SaveOffset {
    type Result = ();
}


impl Handler<GetLastOffset> for InMemOffsetStore {
    type Result = MessageResult<GetLastOffset>;

    fn handle(&mut self, _: GetLastOffset, _: &mut Self::Context) -> Self::Result {
        let last_offset = self.last_offsets.last().cloned();
        MessageResult(last_offset)
    }
}

impl Handler<GetOffset> for InMemOffsetStore {
    type Result = MessageResult<GetOffset>;

    fn handle(&mut self, msg: GetOffset, _: &mut Self::Context) -> Self::Result {
        let offset = self.last_offsets.iter()
            .find(|(ids, _)| ids.contains(&msg.persistence_id))
            .map(|(_, offset)| offset.clone());
        MessageResult(offset)
    }
}

impl Handler<SaveOffset> for InMemOffsetStore {
    type Result = MessageResult<SaveOffset>;

    fn handle(&mut self, msg: SaveOffset, _: &mut Self::Context) -> Self::Result {
        self.last_offsets.push((vec![msg.persistence_id], msg.offset));
        MessageResult(())
    }
}

/// start of my Actor
// Define a message type
#[derive(Message)]
#[rtype(result = "()")]
struct PingMessage(String);
// Define the actor
struct PingActor {
    local_ping_stream: Option<Pin<Box<dyn Stream<Item=PingMessage>>>>,
    // local_ping_stream: Option<Box<dyn Stream<Item = PingMessage> + Unpin>>,

}


impl PingActor {
    fn new(stream: Pin<Box<dyn Stream<Item=PingMessage>>>) -> Self {
        Self {
            local_ping_stream: Some(stream),
        }
    }
}
impl Actor for PingActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Context<Self>) {
        if let Some(stream) = self.local_ping_stream.take() {
            // Self::add_stream(stream, ctx);
            ctx.add_stream(stream);
            println!("Stream local_ping_stream added");
        }
    }
}

// Implement StreamHandler for processing the ping message stream
impl StreamHandler<PingMessage> for PingActor {
    fn handle(&mut self, msg: PingMessage, _ctx: &mut Self::Context) {
        println!("Received: {}", msg.0);
    }
    fn finished(&mut self, ctx: &mut Self::Context) {
        println!("Stream processing finished");
        System::current().stop();
    }
}


#[derive(Message)]
#[rtype(result = "()")]
struct AddStreamMessage;

impl Handler<AddStreamMessage> for PingActor {
    type Result = ();

    fn handle(&mut self, _: AddStreamMessage, ctx: &mut Self::Context) {
        let ping_stream = stream::iter(vec![
            PingMessage("Ping 1".into()),
            PingMessage("Ping 2".into()),
            PingMessage("Ping 3".into()),
        ]);
        Self::add_stream(ping_stream, ctx);
    }
}

/// end of my Actor
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntityId, EntityType, Offset, PersistenceId};
    use actix_rt;
    use futures_util::{stream, SinkExt};
    use futures_util::stream::once;

    #[actix_rt::test]
    async fn test_basic_ops() {
        let volatile_store = InMemOffsetStore::new().start();

        let last_offset = volatile_store.send(GetLastOffset).await.unwrap();
        assert_eq!(last_offset, None);

        let persistence_id =
            PersistenceId::new(EntityType::from("entity-type"), EntityId::from("entity-id"));
        let offset = volatile_store.send(GetOffset { persistence_id: persistence_id.clone() }).await.unwrap();
        assert_eq!(offset, None);

        let offset = Offset::Sequence(10);
        volatile_store.send(SaveOffset { persistence_id: persistence_id.clone(), offset }).await.unwrap();
        let offset = volatile_store.send(GetOffset { persistence_id }).await.unwrap();
        assert_eq!(offset, Some(Offset::Sequence(10)));

        let persistence_id =
            PersistenceId::new(EntityType::from("entity-type"), EntityId::from("entity-id"));
        let offset = Offset::Sequence(10);
        volatile_store.send(SaveOffset { persistence_id: persistence_id.clone(), offset }).await.unwrap();
        let last_offset = volatile_store.send(GetLastOffset).await.unwrap();
        let expected_last_offset = Some((vec![persistence_id], Offset::Sequence(10)));
        assert_eq!(last_offset, expected_last_offset);
    }

    #[actix_rt::test]
    async fn test_ping_stream_handler() {
        let ping_msg_stream =
            once(async { PingMessage("Ping Single 1".into()) });


        // Create a stream of ping messages
        let ping_stream = stream::iter(vec![
            PingMessage("Ping 1".into()),
            PingMessage("Ping 2".into()),
            PingMessage("Ping 3".into()),
        ]);
        let mut addr_1 = PingActor::new(Box::pin(ping_msg_stream));
        addr_1.start();


        // Allow some time for the actor to process the stream
        actix_rt::time::sleep(std::time::Duration::from_secs(1)).await;
        System::current().stop();
    }
}