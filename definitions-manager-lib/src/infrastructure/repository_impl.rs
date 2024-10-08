use async_trait::async_trait;
use crate::read_side_processor::OffsetStoreRepository;
#[derive(Debug)]
pub struct InMemOffsetStoreRepository {
    pub offset_count: u64,
    pub threshold: u64,
}

#[async_trait]
impl OffsetStoreRepository for InMemOffsetStoreRepository {
    async fn update_offset(&self, new_offset: u64) {
        println!("Updating offset to {}", new_offset);
    }

    async fn get_offset(&self) -> u64 {
        self.offset_count
    }
}

#[derive(Debug)]
pub struct SqlOffsetStoreRepository {
    pub offset_count: u64,
    pub threshold: u64,
}