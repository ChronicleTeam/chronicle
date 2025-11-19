use crate::{
    AppConfig, AppState, Id, api,
    auth::{self, AppAuthSession},
    db,
    error::{ApiResult, IntoAnyhow},
    init_layers,
    model::{
        Cell,
        access::{AccessRole, AccessRoleCheck, Resource},
        data::{FieldIdentifier, FieldKind, TableIdentifier},
        users::User,
    },
};
use aide::openapi::OpenApi;
use axum::{
    Json, Router,
    http::header::SET_COOKIE,
    response::IntoResponse,
    routing::{get, post},
};
use axum_test::{TestRequest, TestResponse, TestServer};
use chrono::DateTime;
use sqlx::{Acquire, PgExecutor, PgPool, Postgres};
use std::{collections::HashMap, fmt::Debug};

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

pub async fn test_insert_cell(
    executor: impl PgExecutor<'_> + Copy,
    table_id: Id,
    field_id: Id,
    test_value: Cell,
) -> bool {
    let table_ident = TableIdentifier::new(table_id, "data_table");
    let field_ident = FieldIdentifier::new(field_id);

    println!("{table_ident}.{field_ident}: Iserting: {:?}", test_value);
    test_value
        .bind(sqlx::query(&format!(
            r#"INSERT INTO {table_ident} ({field_ident}) VALUES ($1)"#
        )))
        .execute(executor)
        .await
        .is_ok()
}

pub fn field_tests() -> Vec<(FieldKind, Cell)> {
    const TIMESTAMP: i64 = 1761696082;
    vec![
        (
            FieldKind::Text { is_required: true },
            Cell::String("😀😀😀😀".into()),
        ),
        (
            FieldKind::Integer {
                is_required: false,
                range_start: Some(1),
                range_end: None,
            },
            Cell::Integer(10),
        ),
        (
            FieldKind::Float {
                is_required: true,
                range_start: None,
                range_end: Some(1.0),
            },
            Cell::Float(0.5),
        ),
        (
            FieldKind::Money {
                is_required: true,
                range_start: Some(1_000.into()),
                range_end: Some(1_000_000.into()),
            },
            Cell::Decimal(500_000.into()),
        ),
        (FieldKind::Progress { total_steps: 100 }, Cell::Integer(50)),
        (
            FieldKind::DateTime {
                is_required: true,
                range_start: Some(DateTime::from_timestamp_secs(TIMESTAMP).unwrap()),
                range_end: None,
            },
            Cell::DateTime(DateTime::from_timestamp_secs(TIMESTAMP + 10).unwrap()),
        ),
        (
            FieldKind::WebLink { is_required: true },
            Cell::String("https://example.com".into()),
        ),
        (FieldKind::Checkbox, Cell::Boolean(true)),
        (
            FieldKind::Enumeration {
                is_required: true,
                values: HashMap::from_iter([
                    (0, "Scheduled".into()),
                    (1, "In Progress".into()),
                    (2, "Completed".into()),
                ]),
                default_value: 0,
            },
            Cell::Integer(1),
        ),
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

pub async fn test_access_control<F>(
    conn: impl Acquire<'_, Database = Postgres>,
    resource: Resource,
    resource_id: Id,
    user_id: Id,
    required: AccessRole,
    request: F,
)
where
    F: Fn() -> TestRequest,
{
    let mut conn = conn.acquire().await.unwrap();
    for access_role in [
        None,
        Some(AccessRole::Viewer),
        Some(AccessRole::Editor),
        Some(AccessRole::Owner),
    ] {
        db::delete_many_access(conn.as_mut(), resource, resource_id, [user_id]).await.unwrap();
        if let Some(access_role) = access_role {
            db::create_access(conn.as_mut(), resource, resource_id, user_id, access_role).await.unwrap();
        }
        let expected = access_role.check(required).into_response().status();
        let actual = request().await.status_code();
        assert_eq!(expected, actual);
    }
}
