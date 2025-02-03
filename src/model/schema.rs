use sea_query::{Alias, ColumnRef, Iden, IntoColumnRef, IntoIden, TableRef};

pub trait IntoSchemaTableRef {
    fn schema_table_ref() -> TableRef;
    fn key_col_ref() -> ColumnRef {
        let i = Alias::new("id");
        ColumnRef::Column(i.into_iden())
    }
}

pub trait IntoIteratorColumnRef {
    type C: IntoColumnRef;
    type IC: IntoIterator<Item = Self::C>;
    fn into_iterator_column_ref() -> Self::IC;
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
