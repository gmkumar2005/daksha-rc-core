use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

mod user;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ValidateDefRequest {
    #[schema(example = "4b736e56-8c99-c1c0-bd55-16175ec63f76")]
    #[validate(length(
        min = 36,
        max = 36,
        message = "id is required and must be 36 characters long"
    ))]
    pub id: String,
}
