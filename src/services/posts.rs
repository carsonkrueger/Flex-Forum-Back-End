use itertools::Itertools;

use crate::{models::content_model::ContentModel, routes::AppState};

pub fn sort_by_predicted(
    posts: &mut Vec<ContentModel>,
    s: &AppState,
    num_taken: usize,
    user_id: i64,
) {
    let post_ids = posts.iter().map(|p| p.id).collect::<Vec<_>>();
    let predictions = s
        .ndarray_app_state
        .lock()
        .expect("err locking")
        .predict_all(user_id, &post_ids);

    let zipped = posts.iter().zip(predictions.iter()).collect::<Vec<_>>();
    *posts = zipped
        .iter()
        .sorted_by(|a, b| {
            b.1 .1
                .partial_cmp(&a.1 .1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|&(x, _)| x.clone())
        .take(num_taken)
        .collect::<Vec<_>>();
}
