use crate::{Offset, PersistenceId};

pub type LastOffset = (Vec<PersistenceId>, Offset);

pub enum Command {
    GetLastOffset {
    },
    GetOffset {
        persistence_id: PersistenceId,
    },
    SaveOffset {
        persistence_id: PersistenceId,
        offset: Offset,
    },
}
