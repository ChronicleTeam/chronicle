use super::AppState;
use crate::{
    auth::AppAuthSession,
    db,
    docs::{self, TABLES_TAG, TransformOperationExt},
    error::{ApiError, ApiResult, IntoAnyhow},
    io,
    model::{
        data::{
            CreateTable, CreateTableData, FieldMetadata, GetTable, SelectTable, Table, TableData,
            UpdateTable,
        },
        users::{AccessRole, AccessRoleCheck},
    },
};
use aide::{
    NoApi, OperationOutput,
    axum::{
        ApiRouter,
        routing::{get_with, patch_with, post_with},
    },
    transform::TransformOperation,
};
use axum::{
    Json,
    extract::{Multipart, Path, State},
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
            .api_route(
                "/",
                post_with(create_table, create_table_docs).get_with(get_tables, get_tables_docs),
            )
            .api_route(
                "/{table-id}",
                patch_with(update_table, update_table_docs)
                    .delete_with(delete_table, delete_table_docs),
            )
            .api_route(
                "/{table-id}/children",
                get_with(get_table_children, get_table_children_docs),
            )
            .api_route(
                "/{table-id}/data",
                get_with(get_table_data, get_table_data_docs),
            )
            .api_route(
                "/excel",
                post_with(import_table_from_excel, import_table_from_excel_docs),
            )
            .api_route(
                "/{table-id}/excel",
                post_with(export_table_to_excel, export_table_to_excel_docs),
            )
            .api_route(
                "/csv",
                post_with(import_table_from_csv, import_table_from_csv_docs),
            )
            .api_route(
                "/{table-id}/csv",
                post_with(export_table_to_csv, export_table_to_csv_docs),
            ), // .api_route(
               //     "/{table-id}/access",
               //     post_with(create_table_access, create_table_access_docs)
               //         .patch_with(update_table_access, update_table_access_docs),
               // ),
    )
}

async fn create_table(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Json(create_table): Json<CreateTable>,
) -> ApiResult<Json<Table>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    if let Some(parent_table_id) = create_table.parent_id {
        db::get_table_access(&db, user_id, parent_table_id)
            .await?
            .check(AccessRole::Owner)?;
    }

    let table = db::create_table(tx.as_mut(), create_table).await?;
    db::create_table_access(tx.as_mut(), [(user_id, AccessRole::Owner)], table.table_id).await?;

    tx.commit().await?;
    Ok(Json(table))
}

fn create_table_docs(op: TransformOperation) -> TransformOperation {
    tables_docs::<Json<Table>>(op, "create_table", "Create an empty user table.")
}

async fn update_table(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
    Json(update_table): Json<UpdateTable>,
) -> ApiResult<Json<Table>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_table_access(tx.as_mut(), user_id, table_id)
        .await?
        .check(AccessRole::Editor)?;

    let table = db::update_table(tx.as_mut(), table_id, update_table).await?;

    tx.commit().await?;
    Ok(Json(table))
}

fn update_table_docs(op: TransformOperation) -> TransformOperation {
    select_tables_docs::<Json<Table>>(op, "update_table", "Update a table's meta data.")
}

async fn delete_table(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_table_access(tx.as_mut(), user_id, table_id)
        .await?
        .check(AccessRole::Owner)?;

    db::delete_table(tx.as_mut(), table_id).await?;

    tx.commit().await?;
    Ok(())
}

fn delete_table_docs(op: TransformOperation) -> TransformOperation {
    select_tables_docs::<()>(
        op,
        "delete_table",
        "Delete a table, including all fields and entries.",
    )
}

async fn get_tables(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
) -> ApiResult<Json<Vec<GetTable>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let tables = db::get_tables(&db, user_id).await?;

    Ok(Json(tables))
}

fn get_tables_docs(op: TransformOperation) -> TransformOperation {
    tables_docs::<Json<Vec<Table>>>(op, "get_tables", "Get all tables belonging to the user.")
}

async fn get_table_children(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
) -> ApiResult<Json<Vec<Table>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_table_access(&db, user_id, table_id)
        .await?
        .check(AccessRole::Viewer)?;

    let tables = db::get_table_children(&db, table_id).await?;

    Ok(Json(tables))
}

fn get_table_children_docs(op: TransformOperation) -> TransformOperation {
    select_tables_docs::<Json<Vec<Table>>>(
        op,
        "get_table_children",
        "Get all table children for the specified table.",
    )
}

async fn get_table_data(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
) -> ApiResult<Json<TableData>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_table_access(&db, user_id, table_id)
        .await?
        .check(AccessRole::Viewer)?;

    let data_table = db::get_table_data(&db, table_id).await?;

    Ok(Json(data_table))
}

fn get_table_data_docs(op: TransformOperation) -> TransformOperation {
    select_tables_docs::<Json<TableData>>(
        op,
        "get_table_data",
        "Get all the meta data, fields, and entries of a table.",
    )
}

async fn import_table_from_excel(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    mut multipart: Multipart,
) -> ApiResult<Json<Vec<TableData>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let Some(field) = multipart.next_field().await.unwrap() else {
        return Err(ApiError::BadRequest(MISSING_MULTIPART_FIELD.into()));
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
        let table = db::create_table(tx.as_mut(), table).await?;
        db::create_table_access(tx.as_mut(), [(user_id, AccessRole::Owner)], table.table_id)
            .await?;
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

fn import_table_from_excel_docs(op: TransformOperation) -> TransformOperation {
    tables_docs::<Json<Vec<TableData>>>(
        op,
        "import_table_from_excel",
        "Takes an Excel file and attempts to convert it into a table.",
    )
    .response_description::<400, ()>("Multipart has zero fields")
}

async fn export_table_to_excel(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
    mut multipart: Multipart,
) -> ApiResult<Vec<u8>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_table_access(&db, user_id, table_id)
        .await?
        .check(AccessRole::Viewer)?;

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

fn export_table_to_excel_docs(op: TransformOperation) -> TransformOperation {
    select_tables_docs::<Vec<u8>>(
        op,
        "export_table_to_excel",
        "Converts the specified table into an Excel file. Can optionally take an input Excel file in which to add the table to.",
    )
    .response_description::<400, ()>("Multipart has zero fields")
}

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

    let table = db::create_table(tx.as_mut(), create_table.table).await?;
    db::create_table_access(tx.as_mut(), [(user_id, AccessRole::Owner)], table.table_id).await?;
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

fn import_table_from_csv_docs(op: TransformOperation) -> TransformOperation {
    tables_docs::<Json<TableData>>(
        op,
        "import_table_from_csv",
        "Takes a CSV file and attempts to convert it into a table.",
    )
    .response_description::<400, ()>("Multipart has zero fields")
}

async fn export_table_to_csv(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
) -> ApiResult<Vec<u8>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    db::get_table_access(&db, user_id, table_id)
        .await?
        .check(AccessRole::Viewer)?;

    let mut buffer = Vec::new();
    let csv_writer = csv::Writer::from_writer(Cursor::new(&mut buffer));

    io::export_table_to_csv(csv_writer, db::get_table_data(&db, table_id).await?).anyhow()?;

    Ok(buffer)
}

fn export_table_to_csv_docs(op: TransformOperation) -> TransformOperation {
    select_tables_docs::<Vec<u8>>(
        op,
        "export_table_to_csv",
        "Converts the specified table into a CSV file.",
    )
}

fn tables_docs<'a, R: OperationOutput>(
    op: TransformOperation<'a>,
    summary: &'a str,
    description: &'a str,
) -> TransformOperation<'a> {
    docs::template::<R>(op, summary, description, true, TABLES_TAG)
}

fn select_tables_docs<'a, R: OperationOutput>(
    op: TransformOperation<'a>,
    summary: &'a str,
    description: &'a str,
) -> TransformOperation<'a> {
    tables_docs::<R>(op, summary, description)
        .response_description::<403, ()>("User does not have access to that table")
        .response_description::<404, ()>("Table not found")
}
