use sqlb::HasFields;
use sqlx::{Database, Pool, Postgres};
use uuid::Uuid;

use crate::lib::error::Result;

pub trait DbBmc {
    const TABLE: &'static str;
}

pub async fn create<MC: DbBmc, D: HasFields>(data: D, db: &Pool<Postgres>) -> Result<Uuid> {
    let fields = data.not_none_fields();
    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one::<_, (Uuid,)>(db)
        .await?;
    Ok(id)
}
