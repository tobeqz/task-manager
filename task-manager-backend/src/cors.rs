use axum::{
    body::{boxed, Body, HttpBody},
    http::{HeaderValue, Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn cors_middleware<B>(request: Request<B>, next: Next<B>) -> Response {
    if request.method() != Method::OPTIONS {
        let mut response = next.run(request).await;
        response.headers_mut().append(
            "Access-Control-Allow-Origin",
            HeaderValue::from_str("http://localhost:5173").unwrap(),
        );

        return response
    }

    Response::builder()
        .status(204)
        .header("Access-Control-Allow-Origin", "http://localhost:5173")
        .header("Access-Control-Allow-Methods", "*")
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Max-Age", "86400")
        .body(boxed(Body::empty()))
        .unwrap()
}
