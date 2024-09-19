// use actix_web::{dev::Payload, web, Error, FromRequest, HttpRequest, HttpResponse};
// use futures::future::{ready, Ready};
// use serde::Deserialize;
// use std::collections::HashMap;
// use std::future::Future;
// use std::pin::Pin;
// use std::task::{Context, Poll};
// use actix_web::web::Bytes;
// use crate::app::SchemaApiCommand;
//
// pub struct CommandExtractor(pub HashMap<String, String>, pub SchemaApiCommand);
//
// impl FromRequest for CommandExtractor {
//     type Error = Error;
//     type Future = Ready<Result<Self, Self::Error>>;
//
//     fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
//         let mut metadata = HashMap::new();
//         metadata.insert("time".to_string(), chrono::Utc::now().to_rfc3339());
//         metadata.insert("uri".to_string(), req.uri().to_string());
//         if let Some(user_agent) = req.headers().get("User-Agent") {
//             if let Ok(value) = user_agent.to_str() {
//                 metadata.insert("User-Agent".to_string(), value.to_string());
//             }
//         }
//         // Parse and deserialize the request body as the command payload.
//         let command: SchemaApiCommand = serde_json::from_slice(payload).unwrap();
//         ready(Ok(CommandExtractor(metadata, command)))
//     }
// }