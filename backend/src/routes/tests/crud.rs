use axum::{http::StatusCode, Router};
use serde_json::{json, Value};

use crate::routes::tests::utils::response_to_json;

use super::post_request;

pub async fn create_table(app: &mut Router, name: &str, description: &str) -> i64 {
    let response = post_request(
        app,
        "/api/tables",
        json!(
            {
                "name": name,
                "description": description
            }
        ),
    )
    .await;

    println!("{:?}", response);

    assert_eq!(response.status(), StatusCode::OK);

    let table_id = response_to_json(response)
        .await
        .get("table_id")
        .unwrap()
        .as_i64()
        .unwrap();

    table_id
}

pub async fn create_field(app: &mut Router, table_id: i64, name: &str, options: Value) -> i64 {
    let response = post_request(
        app,
        &format!("/api/tables/{table_id}/fields"),
        json!(
            {
                "name": name,
                "options": options,
            }
        ),
    )
    .await;

    println!("{:?}", response);

    assert_eq!(response.status(), StatusCode::OK);

    let field_id = response_to_json(response)
        .await
        .get("field_id")
        .unwrap()
        .as_i64()
        .unwrap();
    field_id
}
