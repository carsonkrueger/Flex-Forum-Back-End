use sea_query::{Alias, Iden, IntoColumnRef, IntoIden, SimpleExpr, TableRef};
use sqlx::PgConnection;

pub type FlexForumDbConnection = PgConnection;

pub trait IntoSchemaTableRef {
    fn schema_table_ref() -> TableRef;
    fn key_col_ref() -> impl IntoColumnRef {
        Alias::new("id")
    }
}

pub trait IntoIteratorIden {
    type C: IntoIden + 'static;
    type IC: IntoIterator<Item = Self::C>;
    fn into_iterator_iden() -> Self::IC;
}

pub trait IntoIteratorExprVal {
    type IntoIter: IntoIterator<Item = SimpleExpr>;
    fn into_iterator_val(&self) -> Self::IntoIter;
}

#[derive(Clone)]
pub enum Schema {
    // MediaManagement,
    UserManagement,
    WorkoutManagement,
    PostManagement,
}

impl Iden for Schema {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        let name = match self {
            // Self::MediaManagement => "media_management",
            Self::UserManagement => "user_management",
            Self::WorkoutManagement => "workout_management",
            Self::PostManagement => "post_management",
        };
        write!(s, "{}", &name).expect("Iden unquoted - Schema");
    }
}
