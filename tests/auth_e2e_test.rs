mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use common::{parse_json_response, setup_test_app};
use serde_json::{Value, json};
use tower::ServiceExt;

use src::models::auth::AuthResponse;
use src::models::user::{User, UserProfile};

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
async fn test_login_and_protected_profile_route() {
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
                .uri("/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unauth_res.status(), StatusCode::UNAUTHORIZED);

    let profile_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/profile")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", auth_data.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(profile_res.status(), StatusCode::OK);
    let profile_user: User = parse_json_response(profile_res).await;
    assert_eq!(profile_user.email, "bob@example.com");
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
                .uri("/profile")
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

#[tokio::test]
async fn test_exclusive_profile_route_and_get_user_by_id() {
    let app = setup_test_app().await;

    let register_payload = json!({
        "name": "David Profile",
        "email": "david@example.com",
        "password": "secretPassword123"
    });
    let create_res = app
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
    assert_eq!(create_res.status(), StatusCode::CREATED);
    let created_user: User = parse_json_response(create_res).await;

    let login_payload = json!({
        "email": "david@example.com",
        "password": "secretPassword123"
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
    assert_eq!(login_res.status(), StatusCode::OK);
    let auth_data: AuthResponse = parse_json_response(login_res).await;

    // 1. Profile route without bearer token should fail
    let no_token_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(no_token_res.status(), StatusCode::UNAUTHORIZED);

    // 2. Profile route with invalid bearer token should fail
    let invalid_token_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/profile")
                .header(header::AUTHORIZATION, "Bearer invalid-token-string")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_token_res.status(), StatusCode::UNAUTHORIZED);

    // 3. Profile route with valid bearer token returns full data (without password), including token and refresh token
    let profile_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/profile")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", auth_data.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(profile_res.status(), StatusCode::OK);

    let raw_json: Value = parse_json_response(profile_res).await;
    assert_eq!(raw_json["id"], created_user.id);
    assert_eq!(raw_json["name"], "David Profile");
    assert_eq!(raw_json["email"], "david@example.com");
    assert_eq!(raw_json["token"], auth_data.access_token);
    assert_eq!(raw_json["refresh_token"], auth_data.refresh_token);
    assert!(raw_json.get("password").is_none());

    let typed_profile: UserProfile = serde_json::from_value(raw_json.clone()).unwrap();
    assert_eq!(typed_profile.id, created_user.id);
    assert_eq!(typed_profile.name, "David Profile");
    assert_eq!(typed_profile.email, "david@example.com");
    assert_eq!(typed_profile.token, auth_data.access_token);
    assert_eq!(
        typed_profile.refresh_token,
        Some(auth_data.refresh_token.clone())
    );

    // 4. get_user_by_id (/users/{id}) returns ONLY id, name, email (no password, no token, no refresh_token)
    let get_user_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/users/{}", created_user.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_user_res.status(), StatusCode::OK);

    let user_json: Value = parse_json_response(get_user_res).await;
    assert_eq!(user_json["id"], created_user.id);
    assert_eq!(user_json["name"], "David Profile");
    assert_eq!(user_json["email"], "david@example.com");
    assert!(user_json.get("password").is_none());
    assert!(user_json.get("token").is_none());
    assert!(user_json.get("refresh_token").is_none());
}

#[tokio::test]
async fn test_token_version_revocation_on_logout() {
    let app = setup_test_app().await;

    // 1. Create and login user
    let register_payload = json!({
        "name": "Security User",
        "email": "security@example.com",
        "password": "securePassword123"
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
        "email": "security@example.com",
        "password": "securePassword123"
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
    assert_eq!(login_res.status(), StatusCode::OK);
    let auth_data: AuthResponse = parse_json_response(login_res).await;

    // 2. Profile request with active token works (200 OK)
    let profile_before = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/profile")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", auth_data.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(profile_before.status(), StatusCode::OK);

    // 3. User performs logout
    let logout_payload = json!({
        "refresh_token": auth_data.refresh_token
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

    // 4. Accessing /profile with the SAME bearer token now returns 401 Unauthorized
    // because token_version was incremented on logout!
    let profile_after = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/profile")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", auth_data.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(profile_after.status(), StatusCode::UNAUTHORIZED);

    let error_body: Value = parse_json_response(profile_after).await;
    assert_eq!(
        error_body["error"],
        "Token has been revoked or session invalidated"
    );
}


