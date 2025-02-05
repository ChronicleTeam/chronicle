use crate::Id;
use sqlx::PgExecutor;

pub async fn debug_get_user_id(executor: impl PgExecutor<'_>) -> sqlx::Result<Id> {
    Ok(
        sqlx::query_as::<_, (_,)>("SELECT user_id FROM app_user LIMIT 1;")
            .fetch_one(executor)
            .await?
            .0,
    )
}
