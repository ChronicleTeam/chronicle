mod crud;
mod utils;

use crate::{
    config::Config,
    routes::{create_app, ApiState},
    setup_tracing,
};
use axum::{http::StatusCode, Router};
use clap::Parser;
use crud::create_table;
use serde_json::json;
use sqlx::PgPool;
use std::{env, usize};
use tower::ServiceExt;

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

#[sqlx::test]
async fn test_create_field(pool: PgPool) {
    setup_tracing();
    let app = create_app(ApiState {
        config: Config::parse().into(),
        pool: pool.clone(),
    });

    let table_id = create_table(&mut app, "Test Table", "This is a test table");

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

    let field_id = create_field(&mut app, table_id, name, options.clone());

    let (name_db, options_db, data_field_name): (String, Value) = sqlx::query_as(
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
            WHERE table_name = $1 && table_schema = $2;
        "#,
    )
    .bind(data_table_name)
    .bind(schema)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert!(data_field_names_db
        .iter()
        .find(|n| n == data_field_name)
        .is_some());
}
