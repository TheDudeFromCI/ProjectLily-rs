use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder,
    SentenceEmbeddingsModel,
    SentenceEmbeddingsModelType,
};
use rust_bert::RustBertError;
use tch::Device;
use thiserror::Error;
use tokio::task::JoinError;

pub const EMBEDDING_DIM: usize = 384;

pub struct MemoryDB {
    model: SentenceEmbeddingsModel,
    tree: KdTree<f32, String, [f32; EMBEDDING_DIM]>,
}

#[derive(Debug)]
pub struct RecalledMemory {
    pub text: String,
    pub score: f32,
}

impl MemoryDB {
    pub async fn new() -> Result<Self, MemoryDBError> {
        let model = tokio::task::spawn_blocking(|| {
            SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
                .with_device(Device::Cpu)
                .create_model()
        })
        .await??;

        Ok(Self {
            model,
            tree: KdTree::new(EMBEDDING_DIM),
        })
    }

    fn embed(&self, text: &str) -> Result<[f32; EMBEDDING_DIM], MemoryDBError> {
        let embeddings = self.model.encode(&[text])?;
        let tensor = embeddings[0].as_slice();
        let embed: [f32; EMBEDDING_DIM] =
            tensor
                .try_into()
                .map_err(|_| MemoryDBError::WrongEmbeddingSize {
                    expected: EMBEDDING_DIM,
                    actual: tensor.len(),
                })?;

        Ok(embed)
    }

    pub fn add_memory(&mut self, text: &str) -> Result<(), MemoryDBError> {
        let embedding = self.embed(text)?;
        self.tree.add(embedding, text.to_owned())?;
        Ok(())
    }

    pub fn search(&self, query: &str, count: usize) -> Result<Vec<RecalledMemory>, MemoryDBError> {
        let query_embedding = self.embed(query)?;
        let nearest = self
            .tree
            .nearest(&query_embedding, count, &squared_euclidean)?;

        Ok(nearest
            .iter()
            .map(|(distance, data)| RecalledMemory {
                text: data.to_string(),
                score: distance.sqrt(),
            })
            .collect())
    }
}

#[derive(Debug, Error)]
pub enum MemoryDBError {
    #[error("Failed to create model: {0}")]
    FailedToCreateModel(#[from] RustBertError),
    #[error("Wrong embedding size: expected: {expected}, actual: {actual}")]
    WrongEmbeddingSize { expected: usize, actual: usize },
    #[error("Failed to search KD Tree: {0}")]
    KdTreeError(#[from] kdtree::ErrorKind),
    #[error("Failed to spawn blocking task: {0}")]
    AsyncError(#[from] JoinError),
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn simple_db() {
        let mut db = MemoryDB::new().await.unwrap();

        db.add_memory("My favorite color is red.").unwrap();
        db.add_memory("I like apples.").unwrap();
        db.add_memory("The sky is blue.").unwrap();

        let results = db.search("fruit", 1).unwrap();
        assert_eq!(results[0].text, "I like apples.");
    }
}
