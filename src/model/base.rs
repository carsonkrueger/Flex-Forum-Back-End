use super::{
    error::{ModelError, ModelResult},
    schema::{IntoIteratorExprVal, IntoIteratorIden, IntoSchemaTableRef},
};
use sea_query::{
    Expr, Func, IntoColumnRef, IntoIden, PostgresQueryBuilder, Query, SimpleExpr, Value,
};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, Executor, FromRow, Postgres};

const MAX_LIMIT: u64 = 64;

pub trait BaseModelTrait {
    async fn insert_returning<'e, M, C>(
        values: impl IntoIteratorExprVal,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<M>
    where
        M: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoSchemaTableRef,
        C: IntoIteratorIden;

    async fn get_one_with_key<'e, M, RM>(
        key: impl Into<SimpleExpr>,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<Option<RM>>
    where
        M: IntoSchemaTableRef,
        RM: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoIteratorIden;

    async fn get_one_with<'e, M, RM>(
        col: impl IntoColumnRef,
        val: impl Into<SimpleExpr>,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<Option<RM>>
    where
        M: IntoSchemaTableRef,
        RM: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoIteratorIden;

    async fn get_one_with_both<'e, M, RM>(
        col: impl IntoColumnRef,
        val: impl Into<Value>,
        col2: impl IntoColumnRef,
        val2: impl Into<Value>,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<Option<RM>>
    where
        M: IntoSchemaTableRef,
        RM: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoIteratorIden;

    async fn get_all<'e, M, RM>(
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<Vec<RM>>
    where
        M: IntoSchemaTableRef,
        RM: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoIteratorIden;

    async fn update<'e, M, T, VI>(
        values: VI,
        key: SimpleExpr,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<()>
    where
        M: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoSchemaTableRef,
        T: IntoIden,
        VI: IntoIterator<Item = (T, SimpleExpr)>;

    async fn delete_one_by_key<'e, M>(
        key: SimpleExpr,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<()>
    where
        M: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoSchemaTableRef;

    async fn delete_one_with<'e, M>(
        col: impl IntoColumnRef,
        val: SimpleExpr,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<()>
    where
        M: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoSchemaTableRef;

    async fn delete_one_with_both<'e, M>(
        col: impl IntoColumnRef,
        val: SimpleExpr,
        col2: impl IntoColumnRef,
        val2: SimpleExpr,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<()>
    where
        M: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoSchemaTableRef;

    async fn list<'e, M, RM>(
        col: impl IntoColumnRef,
        val: impl Into<SimpleExpr>,
        offset: u64,
        limit: u64,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<Vec<RM>>
    where
        M: IntoSchemaTableRef,
        RM: IntoIteratorIden + Send + Unpin + for<'fr> FromRow<'fr, PgRow>;

    async fn count_where<'e, M>(
        col: impl IntoColumnRef,
        operator: &str,
        val: impl Into<Value>,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<i64>
    where
        M: IntoSchemaTableRef;
}

pub struct BaseModel;

impl BaseModelTrait for BaseModel {
    async fn insert_returning<'e, M, C>(
        values: impl IntoIteratorExprVal,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<M>
    where
        M: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoSchemaTableRef,
        C: IntoIteratorIden,
    {
        let (sql, values) = Query::insert()
            .into_table(M::schema_table_ref())
            .columns(C::into_iterator_iden())
            .values(values.into_iterator_val())?
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        let model = sqlx::query_as_with::<Postgres, M, _>(&sql, values)
            .fetch_one(pool)
            .await?;

        Ok(model)
    }

    async fn get_one_with_key<'e, M, RM>(
        key: impl Into<SimpleExpr>,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<Option<RM>>
    where
        M: IntoSchemaTableRef,
        RM: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoIteratorIden,
    {
        let (sql, values) = Query::select()
            .from(M::schema_table_ref())
            .columns(RM::into_iterator_iden())
            .and_where(Expr::col(M::key_col_ref()).eq(key))
            .build_sqlx(PostgresQueryBuilder);

        let model = sqlx::query_as_with::<Postgres, RM, _>(&sql, values)
            .fetch_optional(pool)
            .await?;

        Ok(model)
    }

    async fn get_one_with<'e, M, RM>(
        col: impl IntoColumnRef,
        val: impl Into<SimpleExpr>,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<Option<RM>>
    where
        M: IntoSchemaTableRef,
        RM: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoIteratorIden,
    {
        let (sql, values) = Query::select()
            .from(M::schema_table_ref())
            .columns(RM::into_iterator_iden())
            .and_where(Expr::col(col).eq(val))
            .build_sqlx(PostgresQueryBuilder);

        let model = sqlx::query_as_with::<Postgres, RM, _>(&sql, values)
            .fetch_optional(pool)
            .await?;

        Ok(model)
    }

    /// Gets the first row with the matching column vals.
    async fn get_one_with_both<'e, M, RM>(
        col: impl IntoColumnRef,
        val: impl Into<Value>,
        col2: impl IntoColumnRef,
        val2: impl Into<Value>,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<Option<RM>>
    where
        M: IntoSchemaTableRef,
        RM: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoIteratorIden,
    {
        let (sql, values) = Query::select()
            .from(M::schema_table_ref())
            .columns(RM::into_iterator_iden())
            .and_where(Expr::col(col).eq(val.into()))
            .and_where(Expr::col(col2).eq(val2.into()))
            .build_sqlx(PostgresQueryBuilder);

        let model = sqlx::query_as_with::<Postgres, RM, _>(&sql, values)
            .fetch_optional(pool)
            .await?;

        Ok(model)
    }

    async fn get_all<'e, M, RM>(
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<Vec<RM>>
    where
        M: IntoSchemaTableRef,
        RM: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoIteratorIden,
    {
        let (sql, values) = Query::select()
            .from(M::schema_table_ref())
            .columns(RM::into_iterator_iden())
            .build_sqlx(PostgresQueryBuilder);

        let model = sqlx::query_as_with::<Postgres, RM, _>(&sql, values)
            .fetch_all(pool)
            .await?;

        Ok(model)
    }

    async fn update<'e, M, T, VI>(
        values: VI,
        key: SimpleExpr,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<()>
    where
        M: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoSchemaTableRef,
        T: IntoIden,
        VI: IntoIterator<Item = (T, SimpleExpr)>,
    {
        let (sql, values) = Query::update()
            .table(M::schema_table_ref())
            .values(values)
            .and_where(Expr::col(M::key_col_ref()).eq(key))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with::<Postgres, _>(&sql, values)
            .fetch_one(pool)
            .await?;

        Ok(())
    }

    async fn delete_one_by_key<'e, M>(
        key: SimpleExpr,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<()>
    where
        M: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoSchemaTableRef,
    {
        let (sql, _) = Query::delete()
            .from_table(M::schema_table_ref())
            .and_where(Expr::col(M::key_col_ref()).eq(key))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query::<Postgres>(&sql).execute(pool).await?;

        Ok(())
    }

    async fn delete_one_with<'e, M>(
        col: impl IntoColumnRef,
        val: SimpleExpr,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<()>
    where
        M: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoSchemaTableRef,
    {
        let (sql, values) = Query::delete()
            .from_table(M::schema_table_ref())
            .and_where(Expr::col(col).eq(val))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with::<Postgres, _>(&sql, values)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn delete_one_with_both<'e, M>(
        col: impl IntoColumnRef,
        val: SimpleExpr,
        col2: impl IntoColumnRef,
        val2: SimpleExpr,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<()>
    where
        M: Send + Unpin + for<'fr> FromRow<'fr, PgRow> + IntoSchemaTableRef,
    {
        let (sql, values) = Query::delete()
            .from_table(M::schema_table_ref())
            .and_where(Expr::col(col).eq(val))
            .and_where(Expr::col(col2).eq(val2))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with::<Postgres, _>(&sql, values)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn list<'e, M, RM>(
        col: impl IntoColumnRef,
        val: impl Into<SimpleExpr>,
        offset: u64,
        limit: u64,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<Vec<RM>>
    where
        M: IntoSchemaTableRef,
        RM: IntoIteratorIden + Send + Unpin + for<'fr> FromRow<'fr, PgRow>,
    {
        let limit = limit.min(MAX_LIMIT);

        let (sql, values) = Query::select()
            .from(M::schema_table_ref())
            .columns(RM::into_iterator_iden())
            .limit(limit)
            .offset(offset)
            .and_where(Expr::col(col).eq(val))
            .build_sqlx(PostgresQueryBuilder);

        let model = sqlx::query_as_with::<Postgres, RM, _>(&sql, values)
            .fetch_all(pool)
            .await?;

        Ok(model)
    }

    async fn count_where<'e, M>(
        col: impl IntoColumnRef,
        operator: &str,
        val: impl Into<Value>,
        pool: &mut crate::model::schema::FlexForumDbConnection,
    ) -> ModelResult<i64>
    where
        M: IntoSchemaTableRef,
    {
        let cond = match operator {
            "=" | "==" => Expr::col(col).eq(val),
            ">" => Expr::col(col).gt(val),
            ">=" => Expr::col(col).gte(val),
            "<" => Expr::col(col).lt(val),
            "<=" => Expr::col(col).lte(val),
            "<>" | "!=" => Expr::col(col).ne(val),
            // "like" => Expr::col(col).like(val),
            // "ilike" => Expr::col(col).ilike(val),
            _ => {
                return Err(ModelError::Sqlx(sqlx::Error::Protocol(
                    "Unsupported operator".into(),
                )))
            }
        };

        let (sql, values) = Query::select()
            .expr(Func::count(cond))
            .from(M::schema_table_ref()) // replace with the actual table name
            .build_sqlx(PostgresQueryBuilder);

        let count: i64 = sqlx::query_scalar_with(&sql, values)
            .fetch_one(pool)
            .await?;
        Ok(count)
    }
}
