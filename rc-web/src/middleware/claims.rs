use crate::ErrorMessage;
use actix_web::{
    dev::Payload,
    error::{ErrorInternalServerError, ResponseError},
    http::{StatusCode, Uri},
    web::Data,
    Error, FromRequest, HttpRequest, HttpResponse,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::extractors::AuthenticationError;
use anyhow::Context;
use derive_more::Display;
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, Validation,
};
use serde::Deserialize;
use shuttle_runtime::SecretStore;
use std::{collections::HashSet, future::Future, pin::Pin};

// Enum to represent different types of client errors
#[derive(Debug, Display)]
enum ClientError {
    #[display("authentication")]
    Authentication(
        AuthenticationError<actix_web_httpauth::headers::www_authenticate::bearer::Bearer>,
    ),
    #[display("decode")]
    Decode(jsonwebtoken::errors::Error),
    #[display("not_found")]
    NotFound(String),
    #[display("unsupported_algorithm")]
    UnsupportedAlgorithm(AlgorithmParameters),
}

impl ResponseError for ClientError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Authentication(_) => StatusCode::UNAUTHORIZED,
            Self::Decode(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::UnsupportedAlgorithm(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Self::Authentication(_) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("authentication_error".to_string()),
                error_description: Some(
                    "Authentication failed. Invalid or missing token.".to_string(),
                ),
                message: "Unauthorized".to_string(),
            }),
            Self::Decode(_) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("invalid_token".to_string()),
                error_description: Some(
                    "Failed to decode token. Ensure it's a valid JWT.".to_string(),
                ),
                message: "Bad credentials".to_string(),
            }),
            Self::NotFound(msg) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("not_found".to_string()),
                error_description: Some(msg.to_string()),
                message: "Token not found".to_string(),
            }),
            Self::UnsupportedAlgorithm(alg) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("unsupported_algorithm".to_string()),
                error_description: Some(format!(
                    "Unsupported encryption algorithm: expected RSA, but got {:?}",
                    alg
                )),
                message: "Bad credentials".to_string(),
            }),
        }
    }
}

// Claims structure for holding user permissions
#[derive(Debug, Deserialize)]
pub struct Claims {
    pub permissions: Option<HashSet<String>>,
}

impl Claims {
    // Helper function to validate permissions
    pub fn validate_permissions(&self, required_permissions: &HashSet<String>) -> bool {
        self.permissions
            .as_ref()
            .is_some_and(|permissions| permissions.is_superset(required_permissions))
    }
}

// Extractor implementation for Claims
impl FromRequest for Claims {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        //  let config = req.app_data::<web::Data<JwtConfig>>();
        // let auth_config = match req.app_data::<Data<BTreeMap<String, String>>>() {
        //     Some(config) => config.clone(),
        //     None => {
        //         return Box::pin(async {
        //             Err(ErrorInternalServerError("Missing Secrets configuration"))
        //         });
        //     }
        // };

        let auth_config = match req.app_data::<Data<SecretStore>>() {
            Some(config) => config.clone(),
            None => {
                return Box::pin(async {
                    Err(ErrorInternalServerError("Missing Secrets configuration"))
                });
            }
        };
        let auth_extractor = BearerAuth::extract(req);
        Box::pin(async move {
            let credentials = auth_extractor.await.map_err(ClientError::Authentication)?;
            let token = credentials.token();
            let header = decode_header(token).map_err(ClientError::Decode)?;
            let kid = header.kid.ok_or_else(|| {
                ClientError::NotFound("kid not found in token header".to_string())
            })?;
            let domain = auth_config
                .get("AUTH0_DOMAIN")
                .context("AUTH0_DOMAIN was not found")
                .map_err(|e| ClientError::NotFound(format!("Invalid JWKS format: {:?}", e)))?;
            let audience = auth_config
                .get("AUTH0_AUDIENCE")
                .context("AUTH0_AUDIENCE was not found")
                .map_err(|e| ClientError::NotFound(format!("Invalid JWKS format: {:?}", e)))?;
            // Fetch JWKS (JSON Web Key Set)
            let jwks_uri = Uri::builder()
                .scheme("https")
                .authority(domain.as_str())
                .path_and_query("/.well-known/jwks.json")
                .build()
                .map_err(|e| ClientError::NotFound(format!("Failed to build JWKS URI: {e}")))?;

            let jwks: JwkSet = reqwest::get(jwks_uri.to_string())
                .await
                .map_err(|e| ClientError::NotFound(format!("Failed to fetch JWKS: {:?}", e)))?
                .json()
                .await
                .map_err(|e| ClientError::NotFound(format!("Invalid JWKS format: {:?}", e)))?;

            let jwk = jwks
                .find(&kid)
                .ok_or_else(|| ClientError::NotFound("No JWK found for kid".to_string()))?;
            match jwk.clone().algorithm {
                AlgorithmParameters::RSA(ref rsa) => {
                    let mut validation = Validation::new(Algorithm::RS256);
                    validation.set_audience(&[audience]);
                    validation.set_issuer(&[format!("https://{}/", domain)]);
                    let key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                        .map_err(ClientError::Decode)?;
                    let token_data =
                        decode::<Claims>(token, &key, &validation).map_err(ClientError::Decode)?;
                    Ok(token_data.claims)
                }
                alg => Err(ClientError::UnsupportedAlgorithm(alg).into()),
            }
        })
    }
}

// #[cfg(test)]
mod tests {
    use actix_web::dev::Payload;
    use actix_web::http::header;
    use actix_web::{test, web, FromRequest};
    use std::collections::BTreeMap;

    use crate::middleware::claims::Claims;

    // #[actix_web::test]
    #[allow(dead_code)]
    async fn test_claims_from_request_success() {
        // Construct a dummy JWT token header with the correct kid
        let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6InFFWXNKVHRUU0tUU1hpQndjT2U4ciJ9.eyJpc3MiOiJodHRwczovL2Rha3NoYS51cy5hdXRoMC5jb20vIiwic3ViIjoieTdkSnN6SE9aMGJiQWJiSlpBc2FLY1IzQnZTVVI4OWJAY2xpZW50cyIsImF1ZCI6Imh0dHBzOi8vZGFrc2hhLnVzLmF1dGgwLmNvbS9hcGkvdjIvIiwiaWF0IjoxNzQ3OTM2ODk3LCJleHAiOjE3NDgwMjMyOTcsImd0eSI6ImNsaWVudC1jcmVkZW50aWFscyIsImF6cCI6Ink3ZEpzekhPWjBiYkFiYkpaQXNhS2NSM0J2U1VSODliIn0.E00qQvo2KtVLsK_e2vfb-npMWheK-ss_3Gnz28hj878AfhFkG8c_qAJNaStLedjYLGB4F54ZiIm2FN5Y1ST80wbeooLQS1fZ-hWoNZfcvpctGsbGtVUN9Nsb7VtB8J43qAB-f8nL59BxDSM5mds2ZxDaS1FMa-eozpdl27rLZyBaLMWEXgI44HE3XEUDhv62Db1VIizDZNM2k3ibH6IjuLPJg2PykalKwOaQpko7EhXCLjVAiovF5XrGwdxXpWJRkDs0ZIGzqH_kR0pVHsew-Y_SvUxxFuDHTe8Wi0XqBnjBtz6BRNsBF-jYgn_EeYBN2FQ3JefcqSonoBvBQU2pIA";
        // let token = "FAKE_TOKEN";
        // You'd want a real signed token for a full test, but for extractor logic, this is fine.

        // Prepare secret store
        let mut secrets = BTreeMap::new();
        // Use mockito's server url for domain
        secrets.insert(
            "AUTH0_DOMAIN".to_string(),
            "daksha.us.auth0.com".to_string(),
        );
        secrets.insert(
            "AUTH0_AUDIENCE".to_string(),
            "https://daksha.us.auth0.com/api/v2/".to_string(),
        );

        // Prepare test request with Bearer header
        let app_data = web::Data::new(secrets);
        let req = test::TestRequest::default()
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .app_data(app_data)
            .to_http_request();

        // Act: Call Claims::from_request
        let mut payload = Payload::None;
        let extract = Claims::from_request(&req, &mut payload);
        // Note: Since decoding will fail (fake token etc.) expect an error at this step.
        let result = extract.await;
        println!("Claims is {:#?}", result);
        // Assert: Should error due to invalid token
        assert!(result.is_ok());
    }
}
