use lib_models::error::ModelResult;
use sqlb::{HasFields, SqlxBindable};
use sqlx::{postgres::PgRow, FromRow, Pool, Postgres, Transaction};

pub trait DbBmc {
    const TABLE: &'static str;
}

/// Creates a row with the table given, returning the id of the inserted row.
pub async fn create<MC: DbBmc, D: HasFields>(data: D, db: &Pool<Postgres>) -> ModelResult<i64> {
    let fields = data.not_none_fields();

    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one::<_, (i64,)>(db)
        .await?;

    Ok(id)
}

pub async fn create_with_transaction<MC: DbBmc, D: HasFields>(
    data: D,
    db: &mut Transaction<'_, Postgres>,
) -> ModelResult<i64> {
    let fields = data.not_none_fields();

    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one::<_, (i64,)>(&mut **db)
        .await?;

    Ok(id)
}

// Gets the first row with the given id.
pub async fn get_one<MC, E, K>(column: &str, key: K, db: &Pool<Postgres>) -> ModelResult<Option<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
    K: SqlxBindable + Send + Sync,
{
    let entity = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .and_where_eq(column, key)
        .limit(1)
        .fetch_optional(db)
        .await?;

    Ok(entity)
}

pub async fn get_one_with_both<MC, E, K1, K2>(
    column_1: &str,
    key_1: K1,
    column_2: &str,
    key_2: K2,
    db: &Pool<Postgres>,
) -> ModelResult<Option<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
    K1: SqlxBindable + Send + Sync,
    K2: SqlxBindable + Send + Sync,
{
    let entity = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .and_where_eq(column_1, key_1)
        .and_where_eq(column_2, key_2)
        .limit(1)
        .fetch_optional(db)
        .await?;

    Ok(entity)
}

pub async fn get_all<MC, E>(db: &Pool<Postgres>, order_by: Option<&str>) -> ModelResult<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let entities = match order_by {
        Some(ob) => {
            sqlb::select()
                .table(MC::TABLE)
                .columns(E::field_names())
                .order_by(ob)
                .fetch_all(db)
                .await?
        }
        None => {
            sqlb::select()
                .table(MC::TABLE)
                .columns(E::field_names())
                .fetch_all(db)
                .await?
        }
    };

    Ok(entities)
}

pub async fn get_all_with_transaction<MC, E>(
    db: &mut Transaction<'_, Postgres>,
    order_by: Option<&str>,
) -> ModelResult<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let entities = match order_by {
        Some(ob) => {
            sqlb::select()
                .table(MC::TABLE)
                .columns(E::field_names())
                .order_by(ob)
                .fetch_all(&mut **db)
                .await?
        }
        None => {
            sqlb::select()
                .table(MC::TABLE)
                .columns(E::field_names())
                .fetch_all(&mut **db)
                .await?
        }
    };

    Ok(entities)
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
pub async fn delete<MC: DbBmc, T: SqlxBindable + Send + Sync>(
    column: &str,
    key: T,
    db: &Pool<Postgres>,
) -> ModelResult<u64> {
    let rows_affected = sqlb::delete()
        .table(MC::TABLE)
        .and_where_eq(column, key)
        .exec(db)
        .await?;

    Ok(rows_affected)
}

// Deletes row with the given id, returning the number of rows affected.
pub async fn delete_with_both<MC: DbBmc, K, K2>(
    column: &str,
    key: K,
    column_2: &str,
    key_2: K2,
    db: &Pool<Postgres>,
) -> ModelResult<u64>
where
    K: SqlxBindable + Send + Sync,
    K2: SqlxBindable + Send + Sync,
{
    let rows_affected = sqlb::delete()
        .table(MC::TABLE)
        .and_where_eq(column, key)
        .and_where_eq(column_2, key_2)
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

pub async fn count_where<MC: DbBmc>(
    db: &Pool<Postgres>,
    column: &str,
    operator: &str,
    value: &str,
) -> ModelResult<i64> {
    let count = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COUNT(id) FROM {} WHERE {} {} $1",
        MC::TABLE,
        column,
        operator,
    ))
    .bind(value)
    .fetch_one(db)
    .await?;
    // Ok(count)
    Ok(count)
}

pub async fn count<MC: DbBmc>(db: &Pool<Postgres>) -> ModelResult<i64> {
    let count = sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(id) FROM {};", MC::TABLE,))
        .fetch_one(db)
        .await?;
    Ok(count)
}
