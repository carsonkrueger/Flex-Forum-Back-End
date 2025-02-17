use chrono::NaiveDateTime;
use itertools::Itertools;
use serde::Serialize;

use crate::{
    model::{
        base::BaseModelTrait,
        schema::FlexForumDbConnection,
        schemas::{
            post_management::{
                following::is_following,
                likes::{get_num_likes, is_liked, LikePost},
                posts::{get_ten_older, get_ten_unseen_older, Posts},
                seen_posts::seen,
            },
            user_management::users::{Users, UsersModelTrait},
        },
    },
    route::{error::RouteResult, AppState},
};

#[derive(Serialize, Debug)]
pub struct GetPostSummary {
    #[serde(flatten)]
    content_model: Posts,
    num_likes: usize,
    is_liked: bool,
    is_following: bool,
}

pub trait PostsServiceTrait {
    fn sort_by_predicted(user_id: i64, posts: &mut Vec<Posts>, num_taken: usize, s: &AppState);

    async fn get_ten_posts<BM: BaseModelTrait, PS: PostsServiceTrait>(
        username: &str,
        created_at: &NaiveDateTime,
        s: &AppState,
        conn: &mut FlexForumDbConnection,
    ) -> RouteResult<Vec<GetPostSummary>>;
}

pub struct PostsService;

impl PostsServiceTrait for PostsService {
    fn sort_by_predicted(user_id: i64, posts: &mut Vec<Posts>, num_taken: usize, s: &AppState) {
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

    async fn get_ten_posts<BM: BaseModelTrait, PS: PostsServiceTrait>(
        username: &str,
        created_at: &NaiveDateTime,
        s: &AppState,
        conn: &mut FlexForumDbConnection,
    ) -> RouteResult<Vec<GetPostSummary>> {
        let mut posts = get_ten_unseen_older(&s.pool, &created_at, username).await?;
        let user = Users::get_user_by_username::<BM>(username, conn)
            .await?
            .unwrap();
        // .unwrap_or(Err(RouteError::Unauthorized)?);

        if posts.len() > 0 {
            PS::sort_by_predicted(user.id, &mut posts, 3, &s);

            // Mark all posts as seen so that they do not get recommended again.
            // Will likely change in the future so that interactions will only count as seen, or number of times recommended.
            for p in &posts {
                seen(&s.pool, username, p.id).await?;
            }
        }
        // if the posts length is 0 then they have seen all recommended posts, so just give them older already seen content again
        else {
            posts = get_ten_older(&s.pool, &created_at).await?;
        }

        let mut post_cards: Vec<GetPostSummary> = Vec::with_capacity(posts.len());

        for i in 0..posts.len() {
            let post_id = posts[i].id;
            let num_likes = get_num_likes::<BM>(conn, post_id).await? as usize;
            let like = LikePost {
                post_id,
                username: username.to_string(),
            };
            let is_liked = is_liked::<BM>(conn, like.post_id, username).await?;
            let is_following = is_following::<BM>(conn, username, &posts[i].username).await?;
            let card = GetPostSummary {
                content_model: posts[i].to_owned(),
                is_liked,
                num_likes,
                is_following,
            };
            post_cards.push(card);
        }

        Ok(post_cards)
    }
}
