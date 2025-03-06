use sqlx::PgExecutor;

use crate::{db::Relation, Id};



pub async fn check_dashboard_relation(
    executor: impl PgExecutor<'_>,
    user_id: Id,
    dashboard_id: Id,
) -> sqlx::Result<Relation> {
    sqlx::query_scalar::<_, Id>(
        r#"
            SELECT user_id
            FROM dashboard
            WHERE dashboard_id = $1
        "#,
    )
    .bind(dashboard_id)
    .fetch_optional(executor)
    .await
    .map(|id| match id {
        None => Relation::Absent,
        Some(id) if id == user_id => Relation::Owned,
        Some(_) => Relation::NotOwned,
    })
}