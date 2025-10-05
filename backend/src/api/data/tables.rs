use super::AppState;
use crate::{
    Id,
    auth::AppAuthSession,
    db,
    error::{ApiError, ApiResult, IntoAnyhow},
    io,
    model::data::{CreateTable, CreateTableData, FieldMetadata, Table, TableData, UpdateTable},
};
use aide::{NoApi, axum::ApiRouter};
use axum::{
    Json,
    extract::{Multipart, Path, State},
    routing::{get, patch, post},
};
use axum_login::AuthSession;
use itertools::Itertools;
use std::io::Cursor;
use umya_spreadsheet::{
    reader::{self, xlsx},
    writer,
};

const MISSING_MULTIPART_FIELD: &str = "Missing multipart field";

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        "/tables",
        ApiRouter::new()
            .route("/", post(create_table).get(get_tables))
            .route("/{table-id}", patch(update_table).delete(delete_table))
            .route("/{table-id}/children", get(get_table_children))
            .route("/{table-id}/data", get(get_table_data))
            .route("/excel", post(import_table_from_excel))
            .route("/{table-id}/excel", post(export_table_to_excel))
            .route("/csv", post(import_table_from_csv))
            .route("/{table-id}/csv", post(export_table_to_csv)),
    )
}

/// Create an empty user table.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
///
async fn create_table(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Json(create_table): Json<CreateTable>,
) -> ApiResult<Json<Table>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let table = db::create_table(&db, user_id, create_table).await?;

    Ok(Json(table))
}

/// Update a table's meta data.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to that table
/// - [ApiError::NotFound]: Table not found
///
async fn update_table(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(table_id): Path<Id>,
    Json(update_table): Json<UpdateTable>,
) -> ApiResult<Json<Table>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_table_relation(&db, user_id, table_id)
        .await?
        .to_api_result()?;

    let table = db::update_table(&db, table_id, update_table).await?;

    Ok(Json(table))
}

/// Delete a table, including all fields and entries.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to that table
/// - [ApiError::NotFound]: Table not found
///
async fn delete_table(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(table_id): Path<Id>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_table_relation(&db, user_id, table_id)
        .await?
        .to_api_result()?;

    db::delete_table(&db, table_id).await?;

    Ok(())
}

/// Get all tables belonging to the user.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
///
async fn get_tables(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
) -> ApiResult<Json<Vec<Table>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let tables = db::get_tables(&db, user_id).await?;

    Ok(Json(tables))
}

/// Get all table children for the specified table.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to that table
/// - [ApiError::NotFound]: Table not found
///
async fn get_table_children(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<Vec<Table>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_table_relation(&db, user_id, table_id)
        .await?
        .to_api_result()?;

    let tables = db::get_table_children(&db, table_id).await?;

    Ok(Json(tables))
}

/// Get all the meta data, fields, and entries of a table.
///
/// Used for displaying the table in the user interface.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to that table
/// - [ApiError::NotFound]: Table not found
///
async fn get_table_data(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<TableData>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_table_relation(&db, user_id, table_id)
        .await?
        .to_api_result()?;

    let data_table = db::get_table_data(&db, table_id).await?;

    Ok(Json(data_table))
}

/// Takes an Excel file and attempts to convert it into an table.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::BadRequest]: Multipart has zero fields
///
async fn import_table_from_excel(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    mut multipart: Multipart,
) -> ApiResult<Json<Vec<TableData>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let Some(field) = multipart.next_field().await.unwrap() else {
        return Err(ApiError::BadRequest("".into()));
    };

    let data = field.bytes().await.anyhow()?;
    let spreadsheet = xlsx::read_reader(Cursor::new(data), true).anyhow()?;

    let create_tables = io::import_table_from_excel(spreadsheet);

    let mut tx = db.begin().await?;

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
            None,
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

/// Converts the specified table into an Excel file.
///
/// Can optionally take an input Excel file in which to add the table to.
/// Otherwise, provide an empty multipart field.
///
/// # TODO
/// Add support for child tables.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to that table
/// - [ApiError::NotFound]: Table not found
/// - [ApiError::BadRequest]: Multipart has zero fields
///
async fn export_table_to_excel(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(table_id): Path<Id>,
    mut multipart: Multipart,
) -> ApiResult<Vec<u8>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_table_relation(&db, user_id, table_id)
        .await?
        .to_api_result()?;

    let mut spreadsheet = if let Some(field) = multipart.next_field().await.anyhow()? {
        let data = field.bytes().await.anyhow()?;
        if data.is_empty() {
            umya_spreadsheet::new_file_empty_worksheet()
        } else {
            reader::xlsx::read_reader(Cursor::new(data), true).anyhow()?
        }
    } else {
        return Err(ApiError::BadRequest(MISSING_MULTIPART_FIELD.into()));
    };

    let mut buffer = Vec::new();
    let data = Cursor::new(&mut buffer);

    io::export_table_to_excel(&mut spreadsheet, db::get_table_data(&db, table_id).await?);

    writer::xlsx::write_writer(&spreadsheet, data).anyhow()?;

    Ok(buffer)
}

/// Takes an CSV file and attempts to convert it into an table.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::BadRequest]: Multipart has zero fields
///
async fn import_table_from_csv(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    mut multipart: Multipart,
) -> ApiResult<Json<TableData>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let Some(field) = multipart.next_field().await.unwrap() else {
        return Err(ApiError::BadRequest(MISSING_MULTIPART_FIELD.into()));
    };

    let name = field.file_name().unwrap_or("CSV Import").to_string();
    let data = field.bytes().await.anyhow()?;
    let csv_reader = csv::Reader::from_reader(Cursor::new(data));

    let create_table = io::import_table_from_csv(csv_reader, &name).anyhow()?;

    let mut tx = db.begin().await?;

    let table = db::create_table(tx.as_mut(), user_id, create_table.table).await?;
    let fields = db::create_fields(tx.as_mut(), table.table_id, create_table.fields).await?;
    let entries = db::create_entries(
        tx.as_mut(),
        table.table_id,
        None,
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

/// Converts the specified table into an CSV file.
///
/// # TODO
/// Add support for child tables.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to that table
/// - [ApiError::NotFound]: Table not found
///
async fn export_table_to_csv(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Vec<u8>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    db::check_table_relation(&db, user_id, table_id)
        .await?
        .to_api_result()?;

    let mut buffer = Vec::new();
    let csv_writer = csv::Writer::from_writer(Cursor::new(&mut buffer));

    io::export_table_to_csv(csv_writer, db::get_table_data(&db, table_id).await?).anyhow()?;

    Ok(buffer)
}
