use async_trait::async_trait;

pub struct SchemaDefServices {
    pub services: Box<dyn SchemaDefServicesApi>,
}

impl SchemaDefServices {
    pub fn new(services: Box<dyn SchemaDefServicesApi>) -> Self {
        Self { services }
    }
}

#[async_trait]
pub trait SchemaDefServicesApi: Sync + Send {
    async fn get_user_id(&self, user_id: &str) -> Result<(), SchemaValidationError>;
}


#[derive(Debug, Clone)]
pub struct SchemaValidationError;
