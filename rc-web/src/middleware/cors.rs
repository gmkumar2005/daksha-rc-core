use actix_cors::Cors;
use actix_web::http::header;

pub fn cors(client_origin_url: &str) -> Cors {
    Cors::default()
        .allowed_origin(client_origin_url)
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .supports_credentials()
        .max_age(86_400)
}
