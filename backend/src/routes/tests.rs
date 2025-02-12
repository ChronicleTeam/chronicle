use std::{env, usize};

use crate::{
    config::Config,
    routes::{create_app, ApiState}, setup_tracing,
};
use axum::{
    body::{to_bytes, Body},
    http::{header::CONTENT_TYPE, Method, Request, Response, StatusCode},
};
use clap::Parser;
use mime::APPLICATION_JSON;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt}; // for `collect`

// #[test]
// fn load_dotenv() {
//     dotenvy::dotenv().unwrap();
//     let database_url = env::var("DATABASE_URL").unwrap();
//     println!("DATABASE_URL = {}", database_url);
//     assert_ne!(database_url, "");
// }

#[sqlx::test]
async fn test_create_table(pool: PgPool) {

    setup_tracing();

    let app = create_app(ApiState {
        config: Config::parse().into(),
        pool: pool.clone(),
    });

    let name = "Test Table";
    let description = "This is a test table";

    let request_body = Body::from(
        json!({
            "name": name,
            "description": description
        })
        .to_string(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tables")
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .method(Method::POST)
                .body(request_body)
                .unwrap(),
        )
        .await
        .unwrap();

    println!("{:?}", response);

    assert_eq!(response.status(), StatusCode::OK);

    let table_id: i64 = response_to_json(response)
        .await
        .get("table_id")
        .unwrap()
        .as_i64()
        .unwrap();

    let (name_sql, description_sql): (String, String) = sqlx::query_as(
        r#"
            SELECT 
                name,
                description
            FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(name_sql, name);
    assert_eq!(description_sql, description);
}

async fn response_to_json(response: Response<Body>) -> Value {
    serde_json::from_slice::<Value>(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
        .unwrap()
}
