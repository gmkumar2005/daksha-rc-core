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
    async fn create_def(&self, id: &str, schema: &str) -> Result<(), SchemaValidationError>;
    async fn validate_def(&self, id: &str) -> Result<(), SchemaValidationError>;
    async fn activate_def(&self, id: &str) -> Result<(), SchemaValidationError>;
    async fn deactivate_def(&self, id: &str) -> Result<(), SchemaValidationError>;
    async fn create_and_validate_def(&self, _id: &str, _schema: &str) -> Result<(), SchemaValidationError>;
}


#[derive(Debug, Clone)]
pub struct SchemaValidationError;
impl SchemaDefServices {
    pub fn create_def(&self, id: &str, schema: &str) -> Result<(), SchemaValidationError> {
        Ok(())
    }

    pub fn validate_def(&self, id: &str) -> Result<(), SchemaValidationError> {
        Ok(())
    }

    pub fn activate_def(&self, id: &str) -> Result<(), SchemaValidationError> {
        Ok(())
    }

    pub fn deactivate_def(&self, id: &str) -> Result<(), SchemaValidationError> {
        Ok(())
    }
}