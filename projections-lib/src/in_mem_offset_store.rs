use crate::offset_store::LastOffset;
use crate::{Offset, PersistenceId};
use actix::prelude::*;
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntityId, EntityType, Offset, PersistenceId};
    use actix_rt;

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
}