use std::time::Instant;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::handlers::AppState;

pub async fn login_time_lock_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let response = next.run(req).await;

    let elapsed = start.elapsed();
    let target_duration = state.auth_service.time_lock_duration();
    if elapsed < target_duration {
        tokio::time::sleep(target_duration - elapsed).await;
    }

    response
}
