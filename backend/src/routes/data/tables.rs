use super::ApiState;
use crate::{
    db,
    error::{ApiError, ApiResult, IntoAnyhow},
    io,
    model::data::{CreateTable, CreateTableData, FieldMetadata, Table, TableData, UpdateTable},
    users::AuthSession,
    Id,
};
use axum::{
    extract::{Multipart, Path, State},
    routing::{get, patch, post},
    Json, Router,
};
use axum_login::AuthUser;
use itertools::Itertools;
use tracing::info;
use std::io::Cursor;
use umya_spreadsheet::{
    reader::{self, xlsx},
    writer,
};

// const TABLE_NAME_CONFLICT: ErrorMessage =
//     ErrorMessage::new_static("name", "Table name already used");

pub fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables",
        Router::new()
            .route("/", post(create_table).get(get_tables))
            .route("/{table-id}", patch(update_table).delete(delete_table))
            .route("/{table-id}/children", get(get_table_children))
            .route("/{table-id}/data", get(get_table_data))
            .route("/excel", post(import_table_from_excel))
            .route("/{table-id}/excel", get(export_table_to_excel))
            .route("/csv", post(import_table_from_csv))
            .route("/{table-id}/csv", get(export_table_to_csv)),
    )
}

/// Create an empty user table.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
///
async fn create_table(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    Json(create_table): Json<CreateTable>,
) -> ApiResult<Json<Table>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.id();

    let table = db::create_table(&pool, user_id, create_table).await?;

    Ok(Json(table))
}

/// Update a table's meta data.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table
/// - [`ApiError::NotFound`]: Table not found
///
async fn update_table(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(update_table): Json<UpdateTable>,
) -> ApiResult<Json<Table>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.id();

    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let table = db::update_table(&pool, table_id, update_table).await?;

    Ok(Json(table))
}

/// Delete a table, including all fields and entries.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table
/// - [`ApiError::NotFound`]: Table not found
///
async fn delete_table(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.id();

    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    db::delete_table(&pool, table_id).await?;

    Ok(())
}

/// Get all tables belonging to the user.
///
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
///
async fn get_tables(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
) -> ApiResult<Json<Vec<Table>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.id();

    let tables = db::get_tables(&pool, user_id).await?;
    
    Ok(Json(tables))
}

async fn get_table_children(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<Vec<Table>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.id();

    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let tables = db::get_table_children(&pool, table_id).await?;

    Ok(Json(tables))
}

/// Get all the meta data, fields, and entries of a table.
///
/// Used for displaying the table in the user interface.
async fn get_table_data(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<TableData>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.id();

    info!("User is logged in");

    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let data_table = db::get_table_data(&pool, table_id).await?;

    Ok(Json(data_table))
}

async fn import_table_from_excel(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    mut multipart: Multipart,
) -> ApiResult<Json<Vec<TableData>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.id();

    let Some(field) = multipart.next_field().await.unwrap() else {
        return Err(ApiError::BadRequest);
    };

    let data = field.bytes().await.into_anyhow()?;
    let spreadsheet = xlsx::read_reader(Cursor::new(data), true).into_anyhow()?;

    let create_tables = io::import_table_from_excel(spreadsheet);

    let mut tx = pool.begin().await?;

    let mut tables = Vec::new();

    for CreateTableData {
        table,
        fields,
        entries,
    } in create_tables
    {
        let table = db::create_table(tx.as_mut(), user_id, table).await?;
        let fields = db::create_fields(tx.as_mut(), table.table_id, fields).await?;
        let entries = db::create_entries(
            tx.as_mut(),
            table.table_id,
            fields
                .iter()
                .map(|field| FieldMetadata::from_field(field.clone()))
                .collect_vec(),
            entries,
        )
        .await?;
        tables.push(TableData {
            table,
            fields,
            entries,
            children: Vec::new(),
        })
    }

    tx.commit().await?;

    Ok(Json(tables))
}

async fn export_table_to_excel(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    mut multipart: Multipart,
) -> ApiResult<Vec<u8>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.id();

    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let mut spreadsheet = if let Some(field) = multipart.next_field().await.into_anyhow()? {
        let data = field.bytes().await.into_anyhow()?;
        if data.is_empty() {
            umya_spreadsheet::new_file_empty_worksheet()
        } else {
            reader::xlsx::read_reader(Cursor::new(data), true).into_anyhow()?
        }
    } else {
        return Err(ApiError::BadRequest);
    };

    let mut buffer = Vec::new();
    let data = Cursor::new(&mut buffer);

    io::export_table_to_excel(&mut spreadsheet, db::get_table_data(&pool, table_id).await?);

    writer::xlsx::write_writer(&spreadsheet, data).into_anyhow()?;

    Ok(buffer)
}

async fn import_table_from_csv(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    mut multipart: Multipart,
) -> ApiResult<Json<TableData>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.id();

    let Some(field) = multipart.next_field().await.unwrap() else {
        return Err(ApiError::BadRequest);
    };

    let name = field.file_name().unwrap_or("CSV Import").to_string();
    let data = field.bytes().await.into_anyhow()?;
    let csv_reader = csv::Reader::from_reader(Cursor::new(data));

    let create_table = io::import_table_from_csv(csv_reader, &name).into_anyhow()?;

    let mut tx = pool.begin().await?;

    let table = db::create_table(tx.as_mut(), user_id, create_table.table).await?;
    let fields = db::create_fields(tx.as_mut(), table.table_id, create_table.fields).await?;
    let entries = db::create_entries(
        tx.as_mut(),
        table.table_id,
        fields
            .iter()
            .map(|field| FieldMetadata::from_field(field.clone()))
            .collect_vec(),
        create_table.entries,
    )
    .await?;

    tx.commit().await?;

    Ok(Json(TableData {
        table,
        fields,
        entries,
        children: Vec::new(),
    }))
}

async fn export_table_to_csv(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Vec<u8>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.id();
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let mut buffer = Vec::new();
    let csv_writer = csv::Writer::from_writer(Cursor::new(&mut buffer));

    io::export_table_to_csv(csv_writer, db::get_table_data(&pool, table_id).await?)
        .into_anyhow()?;

    Ok(buffer)
}
