use super::AppState;
use crate::{
    auth::AppAuthSession,
    db,
    error::{ApiError, ApiResult, IntoAnyhow},
    io,
    model::{
        access::{AccessRole, AccessRoleCheck, Resource},
        data::{
            CreateTable, CreateTableData, FieldMetadata, GetTable, GetTableData, SelectTable,
            Table, TableData, UpdateTable,
        },
    },
};
use aide::{
    NoApi,
    axum::{
        ApiRouter,
        routing::{get_with, patch_with, post_with},
    },
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
                post_with(create_table, docs::create_table).get_with(get_tables, docs::get_tables),
            )
            .api_route(
                "/{table_id}",
                patch_with(update_table, docs::update_table)
                    .delete_with(delete_table, docs::delete_table),
            )
            .api_route(
                "/{table_id}/children",
                get_with(get_table_children, docs::get_table_children),
            )
            .api_route(
                "/{table_id}/data",
                get_with(get_table_data, docs::get_table_data),
            )
            .api_route(
                "/excel",
                post_with(import_table_from_excel, docs::import_table_from_excel),
            )
            .api_route(
                "/{table_id}/excel",
                post_with(export_table_to_excel, docs::export_table_to_excel),
            )
            .api_route(
                "/csv",
                post_with(import_table_from_csv, docs::import_table_from_csv),
            )
            .api_route(
                "/{table_id}/csv",
                post_with(export_table_to_csv, docs::export_table_to_csv),
            ),
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
        db::get_access_role(&db, Resource::Table, parent_table_id, user_id)
            .await?
            .check(AccessRole::Owner)?;
    }

    let table = db::create_table(tx.as_mut(), create_table).await?;
    db::create_access(
        tx.as_mut(),
        Resource::Table,
        table.table_id,
        user_id,
        AccessRole::Owner,
    )
    .await?;

    tx.commit().await?;
    Ok(Json(table))
}

async fn update_table(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
    Json(update_table): Json<UpdateTable>,
) -> ApiResult<Json<Table>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    let table = db::update_table(tx.as_mut(), table_id, update_table).await?;

    tx.commit().await?;
    Ok(Json(table))
}

async fn delete_table(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    db::delete_table(tx.as_mut(), table_id).await?;

    tx.commit().await?;
    Ok(())
}

async fn get_tables(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
) -> ApiResult<Json<Vec<GetTable>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let tables = db::get_tables(&db, user_id).await?;

    Ok(Json(tables))
}

async fn get_table_children(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
) -> ApiResult<Json<Vec<Table>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_access_role(&db, Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Viewer)?;

    let tables = db::get_table_children(&db, table_id).await?;

    Ok(Json(tables))
}

async fn get_table_data(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
) -> ApiResult<Json<GetTableData>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let access_role = db::get_access_role(&db, Resource::Table, table_id, user_id).await?;
    access_role.check(AccessRole::Viewer)?;

    let table_data = db::get_table_data(&db, table_id).await?;

    Ok(Json(GetTableData {
        table_data,
        access_role: access_role.unwrap(),
    }))
}

async fn import_table_from_excel(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    mut multipart: Multipart,
) -> ApiResult<Json<Vec<GetTableData>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let Some(field) = multipart.next_field().await.anyhow()? else {
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
        db::create_access(
            tx.as_mut(),
            Resource::Table,
            table.table_id,
            user_id,
            AccessRole::Owner,
        )
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
        tables.push(GetTableData {
            table_data: TableData {
                table,
                fields,
                entries,
                children: Vec::new(),
            },
            access_role: AccessRole::Owner,
        })
    }

    tx.commit().await?;
    Ok(Json(tables))
}

async fn export_table_to_excel(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
    mut multipart: Multipart,
) -> ApiResult<Vec<u8>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_access_role(&db, Resource::Table, table_id, user_id)
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

async fn import_table_from_csv(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    mut multipart: Multipart,
) -> ApiResult<Json<GetTableData>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let Some(field) = multipart.next_field().await.anyhow()? else {
        return Err(ApiError::BadRequest(MISSING_MULTIPART_FIELD.into()));
    };

    let name = field.file_name().unwrap_or("CSV Import").to_string();
    let data = field.bytes().await.anyhow()?;
    let csv_reader = csv::Reader::from_reader(Cursor::new(data));

    let create_table = io::import_table_from_csv(csv_reader, &name).anyhow()?;

    let mut tx = db.begin().await?;
    let table = db::create_table(tx.as_mut(), create_table.table).await?;
    db::create_access(
        tx.as_mut(),
        Resource::Table,
        table.table_id,
        user_id,
        AccessRole::Owner,
    )
    .await?;
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

    Ok(Json(GetTableData {
        table_data: TableData {
            table,
            fields,
            entries,
            children: Vec::new(),
        },
        access_role: AccessRole::Owner,
    }))
}

async fn export_table_to_csv(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
) -> ApiResult<Vec<u8>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    db::get_access_role(&db, Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Viewer)?;

    let mut buffer = Vec::new();
    let csv_writer = csv::Writer::from_writer(Cursor::new(&mut buffer));

    io::export_table_to_csv(csv_writer, db::get_table_data(&db, table_id).await?).anyhow()?;

    Ok(buffer)
}

#[cfg_attr(coverage_nightly, coverage(off))]
mod docs {
    use crate::{
        docs::{TABLES_TAG, TransformOperationExt, template},
        model::{
            access::{AccessRole, Resource},
            data::{GetTable, GetTableData, Table},
        },
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;

    const TABLE_OWNER: [(Resource, AccessRole); 1] = [(Resource::Table, AccessRole::Owner)];
    const TABLE_VIEWER: [(Resource, AccessRole); 1] = [(Resource::Table, AccessRole::Viewer)];

    fn tables<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        template::<R>(op, summary, description, true, TABLES_TAG)
    }

    pub fn select_tables<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        tables::<R>(op, summary, description).response_description::<404, ()>("Table not found")
    }

    pub fn create_table(op: TransformOperation) -> TransformOperation {
        tables::<Json<Table>>(op, "create_table", "Create an empty user table.")
            .response_description::<404, ()>("Parent table not found")
    }

    pub fn update_table(op: TransformOperation) -> TransformOperation {
        select_tables::<Json<Table>>(op, "update_table", "Update a table's meta data.")
            .required_access(TABLE_OWNER)
    }

    pub fn delete_table(op: TransformOperation) -> TransformOperation {
        select_tables::<()>(
            op,
            "delete_table",
            "Delete a table, including all fields and entries.",
        )
        .required_access(TABLE_OWNER)
    }

    pub fn get_tables(op: TransformOperation) -> TransformOperation {
        tables::<Json<Vec<GetTable>>>(op, "get_tables", "Get all tables viewable to the user.")
    }

    pub fn get_table_children(op: TransformOperation) -> TransformOperation {
        select_tables::<Json<Vec<Table>>>(
            op,
            "get_table_children",
            "Get all table children for the specified table.",
        )
        .required_access(TABLE_VIEWER)
    }

    pub fn get_table_data(op: TransformOperation) -> TransformOperation {
        select_tables::<Json<GetTableData>>(
            op,
            "get_table_data",
            "Get all the meta data, fields, and entries of a table.",
        )
        .required_access(TABLE_VIEWER)
    }

    pub fn import_table_from_excel(op: TransformOperation) -> TransformOperation {
        tables::<Json<Vec<GetTableData>>>(
            op,
            "import_table_from_excel",
            "Takes an Excel file and attempts to convert it into a table.",
        )
        .response_description::<400, ()>("Multipart has zero fields")
    }

    pub fn export_table_to_excel(op: TransformOperation) -> TransformOperation {
        select_tables::<Vec<u8>>(
            op,
            "export_table_to_excel",
            "Converts the specified table into an Excel file. Can optionally take an input Excel file in which to add the table to.",
        )
        .required_access(TABLE_VIEWER)
        .response_description::<400, ()>("Multipart has zero fields")
    }

    pub fn import_table_from_csv(op: TransformOperation) -> TransformOperation {
        tables::<Json<GetTableData>>(
            op,
            "import_table_from_csv",
            "Takes a CSV file and attempts to convert it into a table.",
        )
        .response_description::<400, ()>("Multipart has zero fields")
    }

    pub fn export_table_to_csv(op: TransformOperation) -> TransformOperation {
        select_tables::<Vec<u8>>(
            op,
            "export_table_to_csv",
            "Converts the specified table into a CSV file.",
        )
        .required_access(TABLE_VIEWER)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use std::{collections::HashMap, io::Cursor};

    use crate::{
        Id, db,
        model::{
            Cell,
            access::{AccessRole, Resource},
            data::{
                CreateField, CreateTable, Entry, Field, FieldKind, FieldMetadata, GetTable, Table,
                UpdateTable,
            },
        },
        setup_tracing, test_util,
    };
    use axum::body::Bytes;
    use axum_test::{TestResponse, multipart};
    use itertools::Itertools;
    use serde_json::{Value, json};
    use sqlx::PgPool;

    #[sqlx::test]
    async fn create_table(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let path = "/api/tables";

        let parent_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "parent".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;

        let create_table = CreateTable {
            parent_id: None,
            name: "test".into(),
            description: "description".into(),
        };

        server
            .post(path)
            .json(&create_table)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;

        let response = server.post(path).json(&create_table).await;
        response.assert_status_ok();
        let table_1: Table = response.json();
        assert_eq!(create_table.parent_id, table_1.parent_id);
        assert_eq!(create_table.name, table_1.name);
        assert_eq!(create_table.description, table_1.description);
        let table_2: Table = sqlx::query_as(r#"SELECT * FROM meta_table WHERE table_id = $1"#)
            .bind(table_1.table_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(table_1, table_2);

        let access_role = db::get_access_role(&db, Resource::Table, table_1.table_id, user.user_id)
            .await?
            .unwrap();
        assert_eq!(access_role, AccessRole::Owner);

        let create_table = CreateTable {
            parent_id: Some(parent_id),
            name: "test".into(),
            description: "description".into(),
        };
        test_util::test_access_control(
            &db,
            Resource::Table,
            parent_id,
            user.user_id,
            AccessRole::Owner,
            async || server.post(&path).json(&create_table).await,
        )
        .await;

        let response = server.post(path).json(&create_table).await;
        response.assert_status_ok();
        let table_1: Table = response.json();
        assert_eq!(create_table.parent_id, table_1.parent_id);
        assert_eq!(create_table.name, table_1.name);
        assert_eq!(create_table.description, table_1.description);
        let table_2: Table = sqlx::query_as(r#"SELECT * FROM meta_table WHERE table_id = $1"#)
            .bind(table_1.table_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(table_1, table_2);

        server
            .post(&path)
            .json(&CreateTable {
                parent_id: Some(1000),
                name: "test".into(),
                description: "description".into(),
            })
            .await
            .assert_status_not_found();
        Ok(())
    }

    #[sqlx::test]
    async fn update_table(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "A".into(),
                description: "B".into(),
            },
        )
        .await?
        .table_id;
        let path = format!("/api/tables/{table_id}");

        let update_table = UpdateTable {
            name: "C".into(),
            description: "D".into(),
        };

        server
            .patch(&path)
            .json(&update_table)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Owner,
            async || server.patch(&path).json(&update_table).await,
        )
        .await;

        server
            .patch("/api/tables/1000")
            .json(&update_table)
            .await
            .assert_status_not_found();

        let update_table = UpdateTable {
            name: "E".into(),
            description: "F".into(),
        };
        let response = server.patch(&path).json(&update_table).await;
        response.assert_status_ok();
        let table_1: Table = response.json();
        assert_eq!(update_table.name, table_1.name);
        assert_eq!(update_table.description, table_1.description);

        let table_2: Table = sqlx::query_as(r#"SELECT * FROM meta_table WHERE table_id = $1"#)
            .bind(table_1.table_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(table_1, table_2);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_table(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "A".into(),
                description: "B".into(),
            },
        )
        .await?
        .table_id;
        let path = format!("/api/tables/{table_id}");

        server.delete(&path).await.assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Owner,
            async || server.delete(&path).await,
        )
        .await;

        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "C".into(),
                description: "D".into(),
            },
        )
        .await
        .unwrap()
        .table_id;
        db::create_access(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Owner,
        )
        .await?;
        let path = format!("/api/tables/{table_id}");

        server
            .delete("/api/tables/1000")
            .await
            .assert_status_not_found();

        server.delete(&path).await.assert_status_ok();

        let not_exists: bool = sqlx::query_scalar(
            r#"SELECT NOT EXISTS (SELECT 1 FROM meta_table WHERE table_id = $1)"#,
        )
        .bind(table_id)
        .fetch_one(&db)
        .await?;
        assert!(not_exists);

        server.delete(&path).await.assert_status_not_found();
        Ok(())
    }

    #[sqlx::test]
    async fn get_tables(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let user = db::create_user(&db, "test".into(), "".into(), false).await?;

        let mut tables_1 = Vec::new();
        for (idx, access_role) in [AccessRole::Viewer, AccessRole::Editor, AccessRole::Owner]
            .into_iter()
            .enumerate()
        {
            let table = db::create_table(
                &db,
                CreateTable {
                    parent_id: None,
                    name: idx.to_string(),
                    description: idx.to_string(),
                },
            )
            .await?;
            db::create_access(
                &db,
                Resource::Table,
                table.table_id,
                user.user_id,
                access_role,
            )
            .await?;
            tables_1.push(GetTable { table, access_role });
        }

        let path = "/api/tables";

        server.get(&path).await.assert_status_unauthorized();

        test_util::login_session(&mut server, &user).await;

        let response = server.get(&path).await;
        response.assert_status_ok();
        let tables_2: Vec<GetTable> = response.json();
        test_util::assert_eq_vec(tables_1, tables_2, |t| t.table.table_id);
        Ok(())
    }

    #[sqlx::test]
    async fn get_table_children(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let parent_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "parent".into(),
                description: "parent".into(),
            },
        )
        .await?
        .table_id;

        let mut tables_1 = Vec::new();
        for idx in 0..3 {
            let table = db::create_table(
                &db,
                CreateTable {
                    parent_id: Some(parent_id),
                    name: idx.to_string(),
                    description: idx.to_string(),
                },
            )
            .await?;
            tables_1.push(table);
        }

        let path = format!("/api/tables/{parent_id}/children");

        server.get(&path).await.assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            parent_id,
            user.user_id,
            AccessRole::Viewer,
            async || server.get(&path).await,
        )
        .await;

        server
            .get("/api/tables/1000/children")
            .await
            .assert_status_not_found();

        let response = server.get(&path).await;
        response.assert_status_ok();
        let tables_2: Vec<Table> = response.json();
        test_util::assert_eq_vec(tables_1, tables_2, |t| t.table_id);
        Ok(())
    }

    #[sqlx::test]
    async fn get_table_data(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let table_1 = db::create_table(
            &db,
            CreateTable {
                name: "Test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?;

        let field_1 = db::create_field(
            &db,
            table_1.table_id,
            CreateField {
                name: "Test".into(),
                field_kind: FieldKind::Checkbox,
            },
        )
        .await?;
        let entries_1 = db::create_entries(
            &db,
            table_1.table_id,
            None,
            vec![FieldMetadata::from_field(field_1.clone())],
            vec![
                vec![Cell::Boolean(false)],
                vec![Cell::Boolean(true)],
                vec![Cell::Boolean(true)],
                vec![Cell::Boolean(false)],
                vec![Cell::Boolean(true)],
                vec![Cell::Boolean(false)],
                vec![Cell::Boolean(false)],
            ],
        )
        .await?;
        let path = format!("/api/tables/{}/data", table_1.table_id);

        server.get(&path).await.assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_1.table_id,
            user.user_id,
            AccessRole::Viewer,
            async || server.get(&path).await,
        )
        .await;

        let access_role_1 = AccessRole::Owner;

        server
            .get("/api/tables/1000/children")
            .await
            .assert_status_not_found();

        let response = server.get(&path).await;
        response.assert_status_ok();
        let mut get_table_data: Value = response.json();
        let access_role_2: AccessRole =
            serde_json::from_value(get_table_data.get_mut("access_role").unwrap().take()).unwrap();
        let mut table_data = get_table_data.get_mut("table_data").unwrap().take();
        let table_2: Table =
            serde_json::from_value(table_data.get_mut("table").unwrap().take()).unwrap();
        let (field_2,): (Field,) =
            serde_json::from_value::<Vec<_>>(table_data.get_mut("fields").unwrap().take())
                .unwrap()
                .into_iter()
                .collect_tuple()
                .unwrap();
        let entries_2: Vec<Value> =
            serde_json::from_value(table_data.get_mut("entries").unwrap().take()).unwrap();
        let entries_2: Vec<_> = entries_2
            .into_iter()
            .map(|mut entry| {
                let mut cells = entry.get_mut("cells").unwrap().take();
                Ok(Entry {
                    entry_id: serde_json::from_value(entry.get_mut("entry_id").unwrap().take())?,
                    parent_id: serde_json::from_value(entry.get_mut("parent_id").unwrap().take())?,
                    created_at: serde_json::from_value(
                        entry.get_mut("created_at").unwrap().take(),
                    )?,
                    updated_at: serde_json::from_value(
                        entry.get_mut("updated_at").unwrap().take(),
                    )?,
                    cells: HashMap::from_iter([(
                        field_2.field_id,
                        Cell::Boolean(
                            cells
                                .get_mut(field_2.field_id.to_string())
                                .unwrap()
                                .as_bool()
                                .unwrap(),
                        ),
                    )]),
                })
            })
            .try_collect::<_, _, anyhow::Error>()
            .unwrap();
        let children = table_data.get_mut("children").unwrap().take();

        assert_eq!(access_role_1, access_role_2);
        assert_eq!(table_1, table_2);
        assert_eq!(field_1, field_2);
        test_util::assert_eq_vec(entries_1, entries_2, |e| e.entry_id);
        assert_eq!(children, json!([]));
        Ok(())
    }

    async fn test_import(db: PgPool, path: String, part: multipart::Part) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;

        let excel_form = multipart::MultipartForm::new().add_part("file", part.clone());
        server
            .post(&path)
            .multipart(excel_form)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;

        let excel_form = multipart::MultipartForm::new().add_part("file", part);
        let response = server.post(&path).multipart(excel_form).await;
        response.assert_status_ok();
        let mut get_table_data: Value = response.json();
        if let Some(a) = get_table_data.as_array_mut() {
            get_table_data = a[0].take();
        }
        println!("{get_table_data:?}");
        let access_role_1: AccessRole =
            serde_json::from_value(get_table_data.get_mut("access_role").unwrap().take()).unwrap();
        assert_eq!(access_role_1, AccessRole::Owner);

        let mut table_data_1 = get_table_data.get_mut("table_data").unwrap().take();

        let table_1: Table =
            serde_json::from_value(table_data_1.get_mut("table").unwrap().take()).unwrap();

        let fields_1: Vec<Field> =
            serde_json::from_value(table_data_1.get_mut("fields").unwrap().take()).unwrap();

        let (age_field, name_field) = fields_1
            .iter()
            .sorted_by_key(|f| f.name.to_owned())
            .collect_tuple()
            .unwrap();

        let entries_1: Vec<Value> =
            serde_json::from_value(table_data_1.get_mut("entries").unwrap().take()).unwrap();
        println!("{:?}", entries_1);
        let mut entries_1: Vec<_> = entries_1
            .into_iter()
            .map(|mut entry| {
                let mut cells = entry.get_mut("cells").unwrap().take();
                Ok(Entry {
                    entry_id: serde_json::from_value(entry.get_mut("entry_id").unwrap().take())?,
                    parent_id: serde_json::from_value(entry.get_mut("parent_id").unwrap().take())?,
                    created_at: serde_json::from_value(
                        entry.get_mut("created_at").unwrap().take(),
                    )?,
                    updated_at: serde_json::from_value(
                        entry.get_mut("updated_at").unwrap().take(),
                    )?,
                    cells: HashMap::from_iter([
                        (
                            name_field.field_id,
                            Cell::String(
                                cells
                                    .get_mut(name_field.field_id.to_string())
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .into(),
                            ),
                        ),
                        (
                            age_field.field_id,
                            Cell::String(
                                cells
                                    .get_mut(age_field.field_id.to_string())
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .into(),
                            ),
                        ),
                    ]),
                })
            })
            .try_collect::<_, _, anyhow::Error>()
            .unwrap();

        let children_1 = table_data_1.get_mut("children").unwrap().take();
        assert_eq!(children_1, json!([]));

        let access_role_2 =
            db::get_access_role(&db, Resource::Table, table_1.table_id, user.user_id)
                .await?
                .unwrap();
        assert_eq!(access_role_1, access_role_2);

        let table_data_2 = db::get_table_data(&db, table_1.table_id).await?;

        assert_eq!(table_1, table_data_2.table);
        test_util::assert_eq_vec(fields_1.clone(), table_data_2.fields, |f| f.field_id);
        test_util::assert_eq_vec(entries_1.clone(), table_data_2.entries, |e| e.entry_id);
        assert!(table_data_2.children.is_empty());

        entries_1.sort_by_key(|e| {
            let Cell::String(v) = e.cells[&name_field.field_id].clone() else {
                panic!()
            };
            v
        });

        assert_eq!(
            entries_1[0].cells[&name_field.field_id],
            Cell::String("Alice".into())
        );
        assert_eq!(
            entries_1[0].cells[&age_field.field_id],
            Cell::String("30".into())
        );
        assert_eq!(
            entries_1[1].cells[&name_field.field_id],
            Cell::String("Bob".into())
        );
        assert_eq!(
            entries_1[1].cells[&age_field.field_id],
            Cell::String("25".into())
        );
        Ok(())
    }

    async fn test_export<F>(db: PgPool, path_fn: F) -> anyhow::Result<Bytes>
    where
        F: Fn(Id) -> String,
    {
        let mut server = test_util::server(db.clone()).await;
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "My Table".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let name_field = db::create_field(
            &db,
            table_id,
            CreateField {
                name: "name".into(),
                field_kind: FieldKind::Text { is_required: true },
            },
        )
        .await?;
        let age_field = db::create_field(
            &db,
            table_id,
            CreateField {
                name: "age".into(),
                field_kind: FieldKind::Text { is_required: true },
            },
        )
        .await?;
        db::create_entries(
            &db,
            table_id,
            None,
            vec![
                FieldMetadata::from_field(name_field.clone()),
                FieldMetadata::from_field(age_field.clone()),
            ],
            vec![
                vec![Cell::String("Alice".into()), Cell::Integer(30)],
                vec![Cell::String("Bob".into()), Cell::Integer(25)],
            ],
        )
        .await?;

        let get_multipart =
            || multipart::MultipartForm::new().add_part("name", multipart::Part::bytes(Vec::new()));

        let path = path_fn(table_id);
        server
            .post(&path)
            .multipart(get_multipart())
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Viewer,
            async || server.post(&path).multipart(get_multipart()).await,
        )
        .await;

        server
            .post(&path_fn(1000))
            .multipart(get_multipart())
            .await
            .assert_status_not_found();

        let response = server.post(&path).multipart(get_multipart()).await;
        response.assert_status_ok();
        Ok(response.into_bytes())
    }

    #[sqlx::test]
    async fn import_table_from_excel(db: PgPool) -> anyhow::Result<()> {
        let mut book = umya_spreadsheet::new_file();
        let sheet = book.get_active_sheet_mut();
        sheet.get_cell_mut("A1").set_value("name");
        sheet.get_cell_mut("B1").set_value("age");
        sheet.get_cell_mut("A2").set_value("Alice");
        sheet.get_cell_mut("B2").set_value("30");
        sheet.get_cell_mut("A3").set_value("Bob");
        sheet.get_cell_mut("B3").set_value("25");

        let mut buf = Cursor::new(Vec::new());
        umya_spreadsheet::writer::xlsx::write_writer(&book, &mut buf)?;
        let excel_part = multipart::Part::bytes(buf.into_inner())
            .file_name("import.xlsx")
            .mime_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");

        test_import(db, "/api/tables/excel".into(), excel_part)
            .await
            .unwrap();
        Ok(())
    }

    #[sqlx::test]
    async fn export_table_to_excel(db: PgPool) -> anyhow::Result<()> {
        let excel_bytes = test_export(db, |table_id| format!("/api/tables/{table_id}/excel"))
            .await
            .unwrap();
        let book =
            umya_spreadsheet::reader::xlsx::read_reader(Cursor::new(excel_bytes), true).unwrap();
        let sheet = book.get_sheet_by_name("My Table").unwrap();
        for row in 1..=sheet.get_highest_row() {
            for col in 1..=sheet.get_highest_column() {
                if let Some(cell) = sheet.get_cell((col, row)) {
                    println!("{:?} = {:?}", (col, row), cell.get_value());
                } else {
                    println!("{:?} = <empty>", (col, row));
                }
            }
        }
        // assert_eq!(sheet.get_cell("A1").unwrap().get_value(), "name");
        // assert_eq!(sheet.get_cell("B1").unwrap().get_value(), "age");
        assert_eq!(sheet.get_cell("A2").unwrap().get_value(), "Alice");
        assert_eq!(sheet.get_cell("B2").unwrap().get_value(), "30");
        assert_eq!(sheet.get_cell("A3").unwrap().get_value(), "Bob");
        assert_eq!(sheet.get_cell("B3").unwrap().get_value(), "25");
        Ok(())
    }

    #[sqlx::test]
    async fn import_table_from_csv(db: PgPool) -> anyhow::Result<()> {
        let csv_data = "name,age\nAlice,30\nBob,25\n";
        let csv_part = multipart::Part::bytes(csv_data)
            .file_name("import.csv")
            .mime_type("text/csv");
        test_import(db, "/api/tables/csv".into(), csv_part)
            .await
            .unwrap();
        Ok(())
    }

    #[sqlx::test]
    async fn export_table_to_csv(db: PgPool) -> anyhow::Result<()> {
        let csv_bytes = test_export(db, |table_id| format!("/api/tables/{table_id}/csv"))
            .await
            .unwrap();
        let csv_output = String::from_utf8(csv_bytes.into())?;
        let expected_csv_1 = "name,age\nAlice,30\nBob,25\n";
        let expected_csv_2 = "name,age\nBob,25\nAlice,30\n";
        println!("\"{}\"", csv_output);
        assert!(csv_output == expected_csv_1 || csv_output == expected_csv_2);
        Ok(())
    }
}
