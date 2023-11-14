use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json, body::{BoxBody, HttpBody, Body},
};
use cookie::{Cookie, CookieBuilder};

use crate::{SecretKey, ServerState};

pub async fn auth_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    if path == "/login" || path == "/login/" {
        return Ok(next.run(request).await);
    }

    let secret_key = match std::env::var("SECRET_KEY") {
        Ok(x) => x,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    let cookie_header = match request.headers().get("Cookie") {
        Some(x) => x,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let cookie_header = match cookie_header.to_str() {
        Ok(x) => x,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    let cookies = Cookie::split_parse(cookie_header);

    for cookie in cookies {
        if let Ok(cookie) = cookie {
            if cookie.name() == "Key" && cookie.value() == secret_key {
                let response = next.run(request).await;
                return Ok(response);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub async fn login(
    State(ServerState { key, .. }): State<ServerState>,
    Json(given_key): Json<String>,
) -> Result<Response, StatusCode> {
    if key != given_key {
        return Err(StatusCode::FORBIDDEN);
    }

    let cookie = CookieBuilder::new("Key", given_key).http_only(true).build();

    let body = axum::body::boxed(Body::default());

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Set-Cookie", cookie.to_string())
        .body(body)
        .unwrap())
}
