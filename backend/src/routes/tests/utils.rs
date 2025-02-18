use axum::{body::{to_bytes, Body}, http::{header::CONTENT_TYPE, Method, Request}, Router, ServiceExt};
use mime::APPLICATION_JSON;
use serde_json::Value;

pub async fn json_post_request(app: &mut Router, uri: &str, value: Value)  {
    let request = Request::builder()
        .uri(uri)
        .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
        .method(Method::POST)
        .body(Body::from(value.to_string()))
        .unwrap();

    let t = ServiceExt::<Request<Body>>::ready(app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    t
}

pub async fn response_to_json(response: Response<Body>) -> Value {
    serde_json::from_slice::<Value>(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
        .unwrap()
}
