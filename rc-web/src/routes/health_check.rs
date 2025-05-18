use crate::HEALTH;
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
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
    HttpResponse::Ok().body("Hello world!")
}

/// Simple endpoint to test if the server is running
#[utoipa::path(
    path = "/echo",
    request_body = String,
    tag = HEALTH,
    responses(
        (status = 200, description = "Echo", body = String)
    )
)]
#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
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
    HttpResponse::Ok().body("ok")
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
        Ok(_) => HttpResponse::Ok().body("ready"),
        Err(_) => HttpResponse::ServiceUnavailable().body("db unavailable"),
    }
}
