use crate::routes::{definition_routes, entity_routes};
use actix_web::{web, Scope};

pub fn routes() -> Scope {
    web::scope("/api")
        .service(web::scope("/v1/entity").service(entity_routes::routes()))
        .service(web::scope("/v1/schema").service(definition_routes::routes()))
}
