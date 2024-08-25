use tensorflow::Tensor;

#[derive(Debug)]
pub struct TensorAppState {
    pub user_embeddings: Tensor<f32>,
    pub post_embeddings: Tensor<f32>,
    pub interactions: Tensor<f32>,
}
