use axum::{
    body::{to_bytes, Body},
    http::{header::CONTENT_TYPE, Method, Request, Response},
    Router,
};
use mime::APPLICATION_JSON;
use serde_json::Value;
use tower::{Service, ServiceExt};

mod data;

// #[test]
// fn load_dotenv() {
//     dotenvy::dotenv().unwrap();
//     let database_url = env::var("DATABASE_URL").unwrap();
//     println!("DATABASE_URL = {}", database_url);
//     assert_ne!(database_url, "");
// }

pub async fn post_request(app: &mut Router, uri: &str, body: Value) -> Response<Body> {
    let request = Request::builder()
        .uri(uri)
        .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
        .method(Method::POST)
        .body(Body::from(body.to_string()))
        .unwrap();

    ServiceExt::<Request<Body>>::ready(app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap()
}

pub async fn response_body(response: Response<Body>) -> Value {
    serde_json::from_slice::<Value>(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
        .unwrap()
}
