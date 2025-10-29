use std::{collections::HashMap, fmt::Debug};

use crate::{
    AppConfig, AppState, Id, api,
    auth::{self, AppAuthSession},
    error::{ApiResult, IntoAnyhow},
    init_layers,
    model::{
        Cell,
        data::{FieldIdentifier, FieldKind, TableIdentifier},
        users::User,
    },
};
use aide::openapi::OpenApi;
use axum::{
    Json, Router,
    http::header::SET_COOKIE,
    routing::{get, post},
};
use axum_test::TestServer;
use chrono::DateTime;
use sqlx::{PgExecutor, PgPool};

async fn login(mut session: AppAuthSession, Json(user): Json<User>) -> ApiResult<()> {
    session.login(&user).await.anyhow()?;
    Ok(())
}

async fn get_auth_user(session: AppAuthSession) -> Json<Option<User>> {
    Json(session.user.clone())
}

pub async fn server(db: PgPool) -> TestServer {
    dotenvy::from_filename("example.env").unwrap();
    let config: AppConfig = AppConfig::build().unwrap();
    let app = api::router().finish_api(&mut OpenApi::default());
    let app = app.nest(
        "/test",
        Router::new()
            .route("/login", post(login))
            .route("/user", get(get_auth_user)),
    );
    let app = auth::init(app, db.clone(), config.session_key)
        .await
        .unwrap();
    let app = init_layers(app, config.allowed_origin).unwrap();
    let server = TestServer::new(app.with_state(AppState { db })).unwrap();
    server
}

pub async fn login_session(server: &mut TestServer, user: &User) {
    server.save_cookies();
    let response = server.post("/test/login").json(user).await;
    response.assert_status_ok();
    response.assert_contains_header(SET_COOKIE);
}

#[derive(Debug, Clone)]
pub struct FieldKindTest {
    pub field_kind: FieldKind,
    pub test_value: Cell,
}

impl FieldKindTest {
    pub async fn test_insert(self, executor: impl PgExecutor<'_> + Copy, table_id: Id, field_id: Id) {
        let table_ident = TableIdentifier::new(table_id, "data_table");
        let field_ident = FieldIdentifier::new(field_id);

        let sql = format!(r#"INSERT INTO {table_ident} ({field_ident}) VALUES ($1)"#);
        let query = |value: Cell| value.bind(sqlx::query(&sql)).execute(executor);

        println!("FieldKind {:?}", self.field_kind);
        query(self.test_value).await.unwrap();
    }
}

pub fn field_kind_tests() -> Vec<FieldKindTest> {
    const TIMESTAMP: i64 = 1761696082;
    vec![
        FieldKindTest {
            field_kind: FieldKind::Text { is_required: true },
            test_value: Cell::String("😀😀😀😀".into()),
        },
        FieldKindTest {
            field_kind: FieldKind::Integer {
                is_required: false,
                range_start: Some(1),
                range_end: None,
            },
            test_value: Cell::Integer(10),
        },
        FieldKindTest {
            field_kind: FieldKind::Float {
                is_required: true,
                range_start: None,
                range_end: Some(1.0),
            },
            test_value: Cell::Float(0.5),
        },
        FieldKindTest {
            field_kind: FieldKind::Money {
                is_required: true,
                range_start: Some(1_000.into()),
                range_end: Some(1_000_000.into()),
            },
            test_value: Cell::Decimal(500_000.into()),
        },
        FieldKindTest {
            field_kind: FieldKind::Progress { total_steps: 100 },
            test_value: Cell::Integer(50),
        },
        FieldKindTest {
            field_kind: FieldKind::DateTime {
                is_required: true,
                range_start: Some(DateTime::from_timestamp_secs(TIMESTAMP).unwrap()),
                range_end: None,
            },
            test_value: Cell::DateTime(DateTime::from_timestamp_secs(TIMESTAMP + 10).unwrap()),
        },
        FieldKindTest {
            field_kind: FieldKind::WebLink { is_required: true },
            test_value: Cell::String("https://example.com".into()),
        },
        FieldKindTest {
            field_kind: FieldKind::Checkbox,
            test_value: Cell::Boolean(true),
        },
        FieldKindTest {
            field_kind: FieldKind::Enumeration {
                is_required: true,
                values: HashMap::from_iter([
                    (0, "Scheduled".into()),
                    (1, "In Progress".into()),
                    (2, "Completed".into()),
                ]),
                default_value: 0,
            },
            test_value: Cell::Integer(1),
        },
    ]
}

pub fn assert_eq_vec<T, F, K>(mut vec_1: Vec<T>, mut vec_2: Vec<T>, f: F) 
where 
    T: PartialEq + Debug,
    F: FnMut(&T) -> K + Copy,
    K: Ord,
{
    vec_1.sort_by_key(f);
    vec_2.sort_by_key(f);
    assert_eq!(vec_1, vec_2);
}