use core::time;
use std::collections::HashMap;

use itertools::Itertools;
use ndarray::{Array1, Array2, Array3, ArrayBase, Axis, Dim, OwnedRepr, ShapeError};
use ndarray_rand::RandomExt;
use rand::distributions::Uniform;
use sqlx::{Pool, Postgres};

use crate::models::interactions_matrix_model::build_model;

#[derive(Debug)]
pub struct NDArrayAppState {
    pub user_embeddings: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>>,
    pub post_embeddings: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>>,
    pub interactions_actual: ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>,
    alpha: f32,
    lambda: f32,
    epochs: usize,
    k: usize,
    user_index_hashmap: HashMap<i64, usize>,
    post_index_hashmap: HashMap<i64, usize>,
    next_u_index: usize,
    next_p_index: usize,
}

/// -> (u, v, a predicted, a observed)
pub async fn load_models(
    pg_pool: &Pool<Postgres>,
    alpha: f32,
    lambda: f32,
    epochs: usize,
    k: usize,
    observations: usize,
) -> NDArrayAppState {
    let join_result = build_model(pg_pool)
        .await
        .expect("Could not query for interactions matrix model");

    // println!("{:?}", join_result);

    let u_count = join_result
        .iter()
        .unique_by(|q| q.user_id)
        .collect::<Vec<_>>()
        .len() as u64;
    let v_count = join_result
        .iter()
        .unique_by(|q| q.post_id)
        .collect::<Vec<_>>()
        .len() as u64;

    let u = Array2::random((u_count as usize, k), Uniform::new(0.0, 1.0));
    let v = Array2::random((v_count as usize, k), Uniform::new(0.0, 1.0));
    let mut a_obs = Array3::zeros((u_count as usize, v_count as usize, observations));

    let mut user_index_hashmap = HashMap::<i64, usize>::new();
    let mut post_index_hashmap = HashMap::<i64, usize>::new();
    let mut next_u_index = 0;
    let mut next_v_index = 0;

    for i in 0..join_result.len() {
        if (!user_index_hashmap.contains_key(&join_result[i].user_id)) {
            user_index_hashmap.insert(join_result[i].user_id, next_u_index);
            next_u_index += 1;
        }
        if (!post_index_hashmap.contains_key(&join_result[i].post_id)) {
            post_index_hashmap.insert(join_result[i].post_id, next_v_index);
            next_v_index += 1;
        }
        let u_index = user_index_hashmap[&join_result[i].user_id];
        let v_index = post_index_hashmap[&join_result[i].post_id];
        a_obs[(u_index, v_index, 0)] = join_result[i].is_liked as f32;
        a_obs[(u_index, v_index, 1)] = join_result[i].is_following as f32;
    }

    // println!("{:?}", a_obs);

    NDArrayAppState {
        user_embeddings: u,
        post_embeddings: v,
        interactions_actual: a_obs,
        alpha,
        epochs,
        k,
        lambda,
        user_index_hashmap,
        post_index_hashmap,
        next_u_index,
        next_p_index: next_v_index,
    }
}

impl NDArrayAppState {
    pub fn train(&mut self, alpha: f32, lambda: f32, epochs: usize, k: usize) {
        // let emb_shape = models.user_embeddings.shape();
        let a_shape = self.interactions_actual.shape();
        // let start = std::time::Instant::now();

        for _ in 0..epochs {
            for u_idx in 0..a_shape[0] {
                for p_idx in 0..a_shape[1] {
                    for int_idx in 0..a_shape[2] {
                        let predicted_interaction = self
                            .user_embeddings
                            .row(u_idx)
                            .dot(&self.post_embeddings.row(p_idx));
                        let error = self.interactions_actual[(u_idx, p_idx, int_idx)]
                            - predicted_interaction;

                        for f in 0..k {
                            self.user_embeddings[(u_idx, f)] += alpha
                                * (error * self.post_embeddings[(p_idx, f)]
                                    - lambda * self.user_embeddings[(u_idx, f)]);
                            self.post_embeddings[(p_idx, f)] += alpha
                                * (error * self.user_embeddings[(u_idx, f)]
                                    - lambda * self.post_embeddings[(p_idx, f)]);
                        }
                    }
                }
            }
        }

        // println!("Training finished. Time elapsed: {:?}", start.elapsed());
        // println!("User embedding: {:?}\n", self.user_embeddings);
        // println!("Post embedding: {:?}\n", self.post_embeddings);
        // println!("Actual: {:?}\n", self.interactions_actual);
    }

    pub fn predict(&self, user_id: i64, post_id: i64) -> f32 {
        let u_index = self.user_index_hashmap[&user_id];
        let p_index = self.post_index_hashmap[&post_id];
        self.user_embeddings
            .row(u_index)
            .dot(&self.post_embeddings.row(p_index))
    }

    /// returns the post ids in order of best to worst
    pub fn predict_all(&self, user_id: i64, post_ids: &[i64]) -> Vec<(i64, f32)> {
        let mut predictions = Vec::with_capacity(post_ids.len());

        for &post_id in post_ids {
            predictions.push((post_id, self.predict(user_id, post_id)));
        }

        predictions
    }

    pub fn add_user(&mut self, user_id: i64) -> Result<(), ShapeError> {
        let new_row = Array1::<f32>::random(self.k, Uniform::new(0.0, 1.0));
        self.user_index_hashmap.insert(user_id, self.next_u_index);
        self.next_u_index += 1;
        self.user_embeddings.push_row(new_row.view())?;
        self.add_user_interaction()
    }

    pub fn add_post(&mut self, post_id: i64) -> Result<(), ShapeError> {
        let new_row = Array1::<f32>::random(self.k, Uniform::new(0.0, 1.0));
        self.post_index_hashmap.insert(post_id, self.next_p_index);
        self.next_p_index += 1;
        self.post_embeddings.push_row(new_row.view())?;
        self.add_post_interaction()
    }

    fn add_user_interaction(&mut self) -> Result<(), ShapeError> {
        let shape = self.interactions_actual.shape();
        let new_interaction = Array2::<f32>::zeros((shape[1], shape[2]));
        self.interactions_actual
            .push(Axis(0), new_interaction.view())
    }

    fn add_post_interaction(&mut self) -> Result<(), ShapeError> {
        let shape = self.interactions_actual.shape();
        let new_interaction = Array2::<f32>::zeros((shape[0], shape[2]));
        self.interactions_actual
            .push(Axis(1), new_interaction.view())
    }
}
