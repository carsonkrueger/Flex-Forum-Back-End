use sqlb::HasFields;
use sqlx::{postgres::PgRow, FromRow, Pool, Postgres};

use super::ModelResult;

pub trait DbBmc {
    const TABLE: &'static str;
}

/// Creates a row with the table given, returning the id of the inserted row.
pub async fn create<MC: DbBmc, D: HasFields>(data: D, db: &Pool<Postgres>) -> ModelResult<String> {
    let fields = data.not_none_fields();

    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["username"])
        .fetch_one::<_, (String,)>(db)
        .await?;

    Ok(id)
}

// Gets the first row with the given id.
pub async fn get_one<MC, E>(id: i64, db: &Pool<Postgres>) -> ModelResult<Option<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let entity = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .and_where_eq("id", id)
        .limit(1)
        .fetch_optional(db)
        .await?;

    Ok(entity)
}

// Updates the given fields with the given id, returning the number of rows affected.
pub async fn update<MC: DbBmc, E: HasFields>(
    id: i64,
    data: E,
    db: &Pool<Postgres>,
) -> ModelResult<u64> {
    let fields = data.not_none_fields();

    let rows_affected = sqlb::update()
        .table(MC::TABLE)
        .data(fields)
        .and_where_eq("id", id)
        .exec(db)
        .await?;

    Ok(rows_affected)
}

// Deletes row with the given id, returning the number of rows affected.
pub async fn delete<MC: DbBmc>(username: &str, db: &Pool<Postgres>) -> ModelResult<u64> {
    let rows_affected = sqlb::delete()
        .table(MC::TABLE)
        .and_where_eq("username", username)
        .exec(db)
        .await?;

    Ok(rows_affected)
}

const MAX_LIMIT: i64 = 64;

pub async fn list<MC, E>(
    mut limit: i64,
    offset: i64,
    column: &str,
    like_str: &str,
    db: &Pool<Postgres>,
) -> ModelResult<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    limit = limit.clamp(0, MAX_LIMIT);

    let entity = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .and_where(column, "LIKE", format!("%{}%", like_str))
        .limit(limit)
        .offset(offset)
        .fetch_all(db)
        .await?;

    Ok(entity)
}
