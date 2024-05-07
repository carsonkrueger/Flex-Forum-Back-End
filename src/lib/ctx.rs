use uuid::Uuid;

#[derive(Clone)]
pub struct Ctx {
    user_id: Uuid,
    // username: String,
}

impl Ctx {
    pub fn user_id(&self) -> &Uuid {
        &self.user_id
    }
}
