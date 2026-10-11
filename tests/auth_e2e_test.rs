mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use common::{parse_json_response, setup_test_app};
use serde_json::{Value, json};
use tower::ServiceExt;

use src::models::auth::AuthResponse;
use src::models::user::User;

#[tokio::test]
async fn test_user_registration_and_duplicate_conflict() {
    let app = setup_test_app().await;

    let payload = json!({
        "name": "Alice Tester",
        "email": "alice@example.com",
        "password": "strongPassword123"
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/users")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let created_user: User = parse_json_response(res).await;
    assert_eq!(created_user.name, "Alice Tester");
    assert_eq!(created_user.email, "alice@example.com");

    let duplicate_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/users")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(duplicate_res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_login_and_protected_me_route() {
    let app = setup_test_app().await;

    let register_payload = json!({
        "name": "Bob Tester",
        "email": "bob@example.com",
        "password": "mypassword123"
    });
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/users")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(register_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let wrong_login_payload = json!({
        "email": "bob@example.com",
        "password": "wrongpassword"
    });
    let fail_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(wrong_login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(fail_res.status(), StatusCode::UNAUTHORIZED);

    let correct_login_payload = json!({
        "email": "bob@example.com",
        "password": "mypassword123"
    });
    let login_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(correct_login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_res.status(), StatusCode::OK);

    let auth_data: AuthResponse = parse_json_response(login_res).await;
    assert!(!auth_data.access_token.is_empty());
    assert!(!auth_data.refresh_token.is_empty());
    assert_eq!(auth_data.token_type, "Bearer");
    assert_eq!(auth_data.expires_in, 900);

    let unauth_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unauth_res.status(), StatusCode::UNAUTHORIZED);

    let me_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/me")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", auth_data.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(me_res.status(), StatusCode::OK);
    let me_user: User = parse_json_response(me_res).await;
    assert_eq!(me_user.email, "bob@example.com");
}

#[tokio::test]
async fn test_refresh_token_rotation_and_logout() {
    let app = setup_test_app().await;

    let register_payload = json!({
        "name": "Charlie",
        "email": "charlie@example.com",
        "password": "password123"
    });
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/users")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(register_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let login_payload = json!({
        "email": "charlie@example.com",
        "password": "password123"
    });
    let login_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let initial_auth: AuthResponse = parse_json_response(login_res).await;

    let refresh_payload = json!({
        "refresh_token": initial_auth.refresh_token
    });
    let refresh_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/refresh")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(refresh_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refresh_res.status(), StatusCode::OK);
    let new_auth: AuthResponse = parse_json_response(refresh_res).await;
    assert_ne!(new_auth.refresh_token, initial_auth.refresh_token);
    assert!(!new_auth.access_token.is_empty());

    let me_check = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/me")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", new_auth.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me_check.status(), StatusCode::OK);

    let reuse_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/refresh")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(refresh_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reuse_res.status(), StatusCode::UNAUTHORIZED);

    let logout_payload = json!({
        "refresh_token": new_auth.refresh_token
    });
    let logout_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/logout")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(logout_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(logout_res.status(), StatusCode::OK);
    let logout_body: Value = parse_json_response(logout_res).await;
    assert_eq!(logout_body["message"], "Successfully logged out");

    let post_logout_refresh = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/refresh")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(logout_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(post_logout_refresh.status(), StatusCode::UNAUTHORIZED);
}
