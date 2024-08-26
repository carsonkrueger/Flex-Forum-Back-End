use core::time;

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
    offset: i64,
    alpha: f32,
    lambda: f32,
    epochs: usize,
    k: usize,
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

    println!("{:?}", join_result);

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

    let id_start = 1000;

    let u = Array2::random((u_count as usize, k), Uniform::new(0.0, 1.0));
    let v = Array2::random((v_count as usize, k), Uniform::new(0.0, 1.0));
    let mut a_obs = Array3::zeros((u_count as usize, v_count as usize, observations));

    for i in 0..join_result.len() {
        let u_index = (join_result[i].user_id - id_start) as usize;
        let v_index = (join_result[i].post_id - id_start) as usize;
        a_obs[(u_index, v_index, 0)] = join_result[i].is_liked as f32;
        a_obs[(u_index, v_index, 1)] = join_result[i].is_following as f32;
    }

    println!("{:?}", a_obs);

    NDArrayAppState {
        user_embeddings: u,
        post_embeddings: v,
        interactions_actual: a_obs,
        offset: id_start,
        alpha,
        epochs,
        k,
        lambda,
    }
}

impl NDArrayAppState {
    pub fn train(&mut self, alpha: f32, lambda: f32, epochs: usize, k: usize) {
        // let emb_shape = models.user_embeddings.shape();
        let a_shape = self.interactions_actual.shape();
        let start = std::time::Instant::now();

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

                        if (error > 0.0) {
                            println!("{}.{} : {}", u_idx, p_idx, error);
                        } else {
                            println!("{}.{}", u_idx, p_idx);
                        }

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

        println!("Training finished. Time elapsed: {:?}", start.elapsed());
        println!("User embedding: {:?}\n", self.user_embeddings);
        println!("Post embedding: {:?}\n", self.post_embeddings);
        println!("Actual: {:?}\n", self.interactions_actual);
    }

    pub fn predict(&self, user_id: i64, post_id: i64) -> f32 {
        let u_index = user_id - self.offset as i64;
        let p_index = post_id - self.offset as i64;
        self.user_embeddings
            .row(u_index as usize)
            .dot(&self.post_embeddings.row(p_index as usize))
    }

    pub fn add_user(&mut self) -> Result<(), ShapeError> {
        let new_row = Array1::<f32>::random(self.k, Uniform::new(0.0, 1.0));
        self.user_embeddings.push_row(new_row.view())?;
        self.add_user_interaction()
    }

    pub fn add_post(&mut self) -> Result<(), ShapeError> {
        let new_row = Array1::<f32>::random(self.k, Uniform::new(0.0, 1.0));
        self.post_embeddings.push_row(new_row.view())?;
        self.add_post_interaction()
    }

    fn add_user_interaction(&mut self) -> Result<(), ShapeError> {
        let shape = self.interactions_actual.shape();
        let new_interaction = Array2::<f32>::zeros((shape[0], shape[2]));
        self.interactions_actual
            .push(Axis(0), new_interaction.view())
    }

    fn add_post_interaction(&mut self) -> Result<(), ShapeError> {
        let shape = self.interactions_actual.shape();
        let new_interaction = Array2::<f32>::zeros((shape[1], shape[2]));
        self.interactions_actual
            .push(Axis(1), new_interaction.view())
    }
}
