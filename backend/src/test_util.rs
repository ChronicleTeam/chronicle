use crate::{
        AppConfig, AppState, api,
        auth::{self, AppAuthSession},
        error::{ApiResult, IntoAnyhow},
        init_layers,
        model::users::User,
    };
    use aide::openapi::OpenApi;
    use axum::{
        Json, Router,
        http::header::SET_COOKIE,
        routing::{get, post},
    };
    use axum_test::TestServer;
    use password_auth::generate_hash;
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

    pub async fn create_user(
        executor: impl PgExecutor<'_>,
        username: &str,
        password: &str,
        is_admin: bool,
    ) -> User {
        sqlx::query_as(
            r#"
                INSERT INTO app_user (username, password_hash, is_admin)
                VALUES ($1, $2, $3)
                RETURNING *
            "#,
        )
        .bind(username)
        .bind(generate_hash(password))
        .bind(is_admin)
        .fetch_one(executor)
        .await
        .unwrap()
    }

    pub async fn login_session(server: &mut TestServer, user: &User) {
        server.save_cookies();
        let response = server.post("/test/login").json(user).await;
        response.assert_status_ok();
        response.assert_contains_header(SET_COOKIE);
    }