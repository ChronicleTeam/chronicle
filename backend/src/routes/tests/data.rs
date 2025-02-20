use super::{post_request, response_body};
use crate::{
    config::Config,
    routes::{create_app, ApiState},
    setup_tracing,
};
use axum::{http::StatusCode, Router};
use clap::Parser;
use serde_json::{json, Value};
use sqlx::PgPool;

async fn create_table(app: &mut Router, name: &str, description: &str) -> i64 {
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

    let table_id = response_body(response)
        .await
        .get("table_id")
        .unwrap()
        .as_i64()
        .unwrap();

    table_id
}

#[sqlx::test]
async fn test_create_table(pool: PgPool) {
    setup_tracing();
    let mut app = create_app(ApiState {
        config: Config::parse().into(),
        pool: pool.clone(),
    });

    let name = "Test Table";
    let description = "This is a test table";

    let table_id = create_table(&mut app, name, description).await;

    let (name_db, description_db): (String, String) = sqlx::query_as(
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

    assert_eq!(name, name_db);
    assert_eq!(description, description_db);
}

async fn create_field(app: &mut Router, table_id: i64, name: &str, options: Value) -> i64 {
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

    let field_id = response_body(response)
        .await
        .get("field_id")
        .unwrap()
        .as_i64()
        .unwrap();
    field_id
}

#[sqlx::test]
async fn test_create_field(pool: PgPool) {
    setup_tracing();
    let mut app = create_app(ApiState {
        config: Config::parse().into(),
        pool: pool.clone(),
    });

    let table_id = create_table(&mut app, "Test Table", "This is a test table").await;

    let data_table_name: String = sqlx::query_scalar(
        r#"
            SELECT data_table_name
            FROM meta_table
            WHERE table_id = $1        
        "#,
    )
    .bind(table_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let (schema, data_table_name) = data_table_name.split_once(".").unwrap();

    let name = "Test Field";
    let options = json!({"type": "Text", "is_required": true});

    let field_id = create_field(&mut app, table_id, name, options.clone()).await;

    let (name_db, options_db, data_field_name): (String, Value, String) = sqlx::query_as(
        r#"
            SELECT
                name,
                options,
                data_field_name
            FROM meta_field
            WHERE field_id = $1
        "#,
    )
    .bind(field_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(name, name_db);
    assert_eq!(options, options_db);

    let data_field_names_db: Vec<String> = sqlx::query_scalar(
        r#"
            SELECT column_name
            FROM information_schema.columns
            WHERE table_name = $1 AND table_schema = $2
        "#,
    )
    .bind(data_table_name)
    .bind(schema)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert!(data_field_names_db
        .iter()
        .find(|n| **n == data_field_name)
        .is_some());

    assert!(false)
}
