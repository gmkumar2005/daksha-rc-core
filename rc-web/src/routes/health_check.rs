use crate::middleware::claims::Claims;
use crate::HEALTH;
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use chrono::Utc;
use log::debug;
use serde_json::{json, Value};
use sqlx::PgPool;

pub fn routes() -> Scope {
    web::scope("")
        // .service(handlers::admin)
        .service(hello)
        .service(echo)
        .service(healthz)
        .service(readyz)
}
/// Simple hello
#[utoipa::path(
    get,
    path = "/hello",
    tag = HEALTH,
    responses(
        (status = 200, description = "Say hello", body = String)
    )
)]
#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("Hello world!")
}

/// Simple endpoint to test if the server is running
#[utoipa::path(
    path = "/echo",
     request_body(
        content = Value,
        content_type = "application/json",
        example = json!({
                "Hello": 123,
                "Message": [1, 2, 3]
            })

        ),
        tag = HEALTH,
        responses(
            (status = 200, description = "Echo", body = Value, content_type = "application/json")
        ),
    security(
           ("bearer_auth" = []),
       )
)]
#[post("/echo")]
async fn echo(req_body: web::Json<Value>, claims: Claims) -> impl Responder {
    debug!("Claims: {:?}", claims);
    let response = json!({
        "version": "1.0.0",
        "timestamp": Utc::now().to_rfc3339(),
        "data": req_body.into_inner()
    });

    HttpResponse::Ok().json(response)
}
/// Liveness probe
#[utoipa::path(
    get,
    path = "/healthz",
    tag = HEALTH,
    responses(
        (status = 200, description = "Health check", body = String)
    )
)]
#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("healthy")
}

/// Readiness probe with DB check
#[utoipa::path(
    get,
    path = "/readyz",
    tag = HEALTH,
    responses(
        (status = 200, description = "Readiness ", body = String)
    )
)]
#[get("/readyz")]
async fn readyz(db_pool: web::Data<PgPool>) -> impl Responder {
    // Run a simple query to check DB connectivity (adapt query to your DB as needed)
    match sqlx::query("SELECT 1").execute(db_pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok()
            .content_type("text/plain; charset=utf-8")
            .body("ready"),
        Err(_) => HttpResponse::ServiceUnavailable().body("db unavailable"),
    }
}
