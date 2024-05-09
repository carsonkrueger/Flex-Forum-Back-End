use sqlb::HasFields;
use sqlx::{Pool, Postgres};

use super::{Error, Result};

pub trait DbBmc {
    const TABLE: &'static str;
}

/// Creates a row with the table given, returning the 'id' column
pub async fn create<MC: DbBmc, D: HasFields>(data: D, db: &Pool<Postgres>) -> Result<i64> {
    let fields = data.not_none_fields();

    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one::<_, (i64,)>(db)
        .await?;

    Ok(id)
}
