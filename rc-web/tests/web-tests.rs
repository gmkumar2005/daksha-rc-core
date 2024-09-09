#[cfg(test)]
mod tests {
    use actix_web::{App, FromRequest, Handler, Responder, Route, test};
    use actix_web::{body, body::MessageBody as _, rt::pin, web};
    use actix_web::test::TestRequest;
    use actix_web::http::header::ContentType;
    use super::*;
    use rc_web::main::hello;
    #[actix_web::test]
    async fn test_index_get() {
        let app = test::init_service(App::new().service(hello)).await;
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body = resp.into_body();
        let body_bytes = body::to_bytes(body).await.unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        println!("Body: {:?}", body_string);
        assert_eq!(body_string, "Hello, Actix web!");

    }
}