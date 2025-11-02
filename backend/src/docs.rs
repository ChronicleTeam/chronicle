use aide::{
    OperationOutput,
    axum::ApiRouter,
    openapi::{ApiKeyLocation, OpenApi, SecurityScheme},
    swagger::Swagger,
    transform::{TransformOpenApi, TransformOperation},
};
use axum::{Extension, Json, Router, routing::get};
use itertools::Itertools;
use std::{fs::File, io::BufWriter, sync::Arc};

use crate::{AppState, model::access::AccessRole};

// Tags to seperate API doc endpoints into categories.

pub const AUTHENTICATION_TAG: &str = "Authentication";
pub const USERS_TAG: &str = "Users";
pub const ACCESS_TAG: &str = "Access";

pub const TABLES_TAG: &str = "Tables";
pub const FIELDS_TAG: &str = "Fields";
pub const ENTRIES_TAG: &str = "Entries";

pub const DASHBOARDS_TAG: &str = "Dashboards";
pub const CHARTS_TAG: &str = "Charts";
pub const AXES_TAG: &str = "Axes";

pub const SECURITY_SCHEME: &str = "cookieAuth";

pub trait TransformOperationExt {
    fn response_description<const N: u16, R: OperationOutput>(self, description: &str) -> Self;

    fn required_access<'a>(self, role: impl IntoIterator<Item = (&'a str, AccessRole)>) -> Self;
}

impl TransformOperationExt for TransformOperation<'_> {
    fn response_description<const N: u16, R: OperationOutput>(self, description: &str) -> Self {
        self.response_with::<N, R, _>(|r| r.description(description))
    }

    fn required_access<'a>(self, roles: impl IntoIterator<Item = (&'a str, AccessRole)>) -> Self {
        let roles = roles
            .into_iter()
            .map(|(prefix, role)| {
                let role = match role {
                    AccessRole::Viewer => "Viewer",
                    AccessRole::Editor => "Editor",
                    AccessRole::Owner => "Owner",
                };
                format!("{prefix}: {role}")
            })
            .join(", ");

        self.response_description::<403, ()>(&format!("Required access roles: {roles}"))
    }
}

pub fn template<'a, R: OperationOutput>(
    mut op: TransformOperation<'a>,
    summary: &'a str,
    description: &'a str,
    secure: bool,
    tag: &str,
) -> TransformOperation<'a> {
    if secure {
        op = op
            .response_description::<401, ()>("User is not authenticated")
            .security_requirement(SECURITY_SCHEME)
    }
    op.summary(summary)
        .description(description)
        .response_description::<200, R>("Success")
        .tag(tag)
}

/// Initialize API documentation endpoints.
pub fn init(app: ApiRouter<AppState>) -> anyhow::Result<Router<AppState>> {
    let mut api_docs = OpenApi::default();

    let app = app.finish_api_with(&mut api_docs, api_docs_transform);

    if cfg!(debug_assertions) {
        let file = File::create("./api.json")?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &api_docs)?;
    }

    Ok(app.merge(router()).layer(Extension(Arc::new(api_docs))))
}

/// Settings for the OpenApi documentation.
fn api_docs_transform(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Chronicle back-end")
        .summary("Application for managing tabular data and creating dashboards.")
        .security_scheme(
            SECURITY_SCHEME,
            SecurityScheme::ApiKey {
                name: "id".into(),
                location: ApiKeyLocation::Cookie,
                description: Some("Session cookie".into()),
                extensions: Default::default(),
            },
        )
        .security_requirement(SECURITY_SCHEME)
}

fn router() -> Router<AppState> {
    Router::new()
        .route("/docs/api.json", get(serve_docs))
        .route(
            "/docs",
            get(Swagger::new("/docs/api.json")
                .with_title("Chronicle API")
                .axum_handler()),
        )
}

/// Get the OpenApi JSON documentation.
async fn serve_docs(Extension(api_docs): Extension<Arc<OpenApi>>) -> Json<Arc<OpenApi>> {
    Json(api_docs)
}
