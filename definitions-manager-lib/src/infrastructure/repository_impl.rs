use async_trait::async_trait;
use crate::read_side_processor::OffsetStoreRepository;
#[derive(Debug)]
pub struct InMemOffsetStoreRepository {
    pub offset_count: u64,
    pub threshold: u64,
}

#[async_trait]
impl OffsetStoreRepository for InMemOffsetStoreRepository  {
    type Item = u64;
    type Error = anyhow::Error;
    async fn update_offset(&self, new_offset: u64)-> Result<(), Self::Error>{
        println!("Updating offset to {}", new_offset);
        Ok(())
    }
    async fn get_offset(&self) -> Result<Option<Self::Item>, Self::Error>{
        println!("Getting offset");
        Ok(Some(self.offset_count))
    }
}

#[derive(Debug)]
pub struct SqlOffsetStoreRepository {
    pub offset_count: u64,
    pub threshold: u64,
}