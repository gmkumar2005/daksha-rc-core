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
#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
/// Liveness probe: Kubernetes checks if the container is running
#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

/// Readiness probe: Kubernetes checks if the container is ready to serve requests
/// Readiness probe with DB check
#[get("/readyz")]
async fn readyz(db_pool: web::Data<PgPool>) -> impl Responder {
    // Run a simple query to check DB connectivity (adapt query to your DB as needed)
    match sqlx::query("SELECT 1").execute(db_pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().body("ready"),
        Err(_) => HttpResponse::ServiceUnavailable().body("db unavailable"),
    }
}
