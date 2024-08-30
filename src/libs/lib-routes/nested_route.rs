use axum::Router;

pub trait NestedRoute<S> {
    const PATH: &'static str;
    fn router() -> Router<S>;
}
