use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use src::docs::ApiDoc;
use src::docs::schemas::ApiSchemas;
use utoipa::OpenApi;

mod common;
use common::setup_test_app;

#[tokio::test]
async fn test_scalar_ui_serves_html() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .expect("Missing content-type header")
        .to_str()
        .unwrap();
    assert!(
        content_type.contains("text/html"),
        "Expected text/html, got {}",
        content_type
    );

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert!(
        body_str.contains("scalar") || body_str.contains("@scalar"),
        "HTML body should contain scalar reference"
    );
}

#[tokio::test]
async fn test_openapi_spec_json_endpoint() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api-docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let spec: Value = serde_json::from_slice(&body_bytes).expect("Valid JSON spec");

    // Validação de versão e paths
    assert!(spec.get("openapi").is_some());
    let paths = spec["paths"].as_object().expect("paths object");
    assert!(paths.contains_key("/login"), "Missing /login");
    assert!(paths.contains_key("/refresh"), "Missing /refresh");
    assert!(paths.contains_key("/logout"), "Missing /logout");
    assert!(paths.contains_key("/profile"), "Missing /profile");
    assert!(paths.contains_key("/users"), "Missing /users");
    assert!(paths.contains_key("/users/{id}"), "Missing /users/{{id}}");

    // Validação dos schemas registrados via schemas.rs
    let schemas = spec["components"]["schemas"]
        .as_object()
        .expect("schemas object");
    assert!(schemas.contains_key("LoginDto"), "Missing LoginDto schema");
    assert!(
        schemas.contains_key("AuthResponse"),
        "Missing AuthResponse schema"
    );
    assert!(schemas.contains_key("User"), "Missing User schema");
    assert!(
        schemas.contains_key("UserProfile"),
        "Missing UserProfile schema"
    );
    assert!(
        schemas.contains_key("ErrorResponse"),
        "Missing ErrorResponse schema"
    );

    // Validação do security scheme configurado via security.rs
    let security_schemes = spec["components"]["securitySchemes"]
        .as_object()
        .expect("securitySchemes object");
    assert!(
        security_schemes.contains_key("bearer_auth"),
        "Missing bearer_auth security scheme"
    );
    assert_eq!(security_schemes["bearer_auth"]["type"], "http");
    assert_eq!(security_schemes["bearer_auth"]["scheme"], "bearer");
    assert_eq!(security_schemes["bearer_auth"]["bearerFormat"], "JWT");
}

#[test]
fn test_srp_schemas_module_isolation() {
    // Validação unitária de SRP: ApiSchemas possui os componentes registrados de forma isolada
    let schemas_doc = ApiSchemas::openapi();
    let components = schemas_doc.components.expect("Components must be present");
    assert!(components.schemas.contains_key("LoginDto"));
    assert!(components.schemas.contains_key("AuthResponse"));
    assert!(components.schemas.contains_key("RefreshTokenDto"));
    assert!(components.schemas.contains_key("User"));
    assert!(components.schemas.contains_key("ErrorResponse"));
}

#[test]
fn test_srp_build_spec_merges_schemas_and_paths() {
    let full_spec = ApiDoc::build_spec();
    let components = full_spec.components.expect("Merged components must exist");
    assert!(components.schemas.contains_key("LoginDto"));
    assert!(components.security_schemes.contains_key("bearer_auth"));
    assert!(full_spec.paths.paths.contains_key("/login"));
    assert!(full_spec.paths.paths.contains_key("/profile"));
}
