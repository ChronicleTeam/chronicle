mod entries;
mod fields;
mod tables;

use crate::{model::data::DataTable, Id};
use sqlx::{Acquire, Postgres};
pub use {entries::*, fields::*, tables::*};

// All SELECT statements lock selected rows during the transaction.
// A regular connection will lock only for the duration of the function.

pub async fn get_data_table(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
) -> sqlx::Result<Option<DataTable>> {
    let mut tx = connection.begin().await?;

    let (data_table_name,): (String,) = sqlx::query_as(
        r#"
            SELECT data_table_name
            FROM meta_table
            WHERE table_id = $1
            FOR UPDATE
        "#,
    )
    .bind(table_id)
    .fetch_one(tx.as_mut())
    .await?;

    let fields = get_fields_options(tx.as_mut(), table_id).await?;

    todo!()
}
