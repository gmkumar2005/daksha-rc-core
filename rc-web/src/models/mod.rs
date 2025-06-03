use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

mod user;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ValidateDefRequest {
    #[validate(length(
        min = 36,
        max = 36,
        message = "id is required and must be 36 characters long"
    ))]
    pub id: String,
}
