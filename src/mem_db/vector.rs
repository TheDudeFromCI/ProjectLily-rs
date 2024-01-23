use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder,
    SentenceEmbeddingsModel,
    SentenceEmbeddingsModelType,
};
use tch::Device;

use super::{MemoryDBError, RecalledMemory};

pub const EMBEDDING_DIM: usize = 384;

pub struct VectorDB {
    model: SentenceEmbeddingsModel,
    tree: KdTree<f32, String, [f32; EMBEDDING_DIM]>,
}

impl VectorDB {
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn simple_db() {
        let mut db = VectorDB::new().await.unwrap();

        db.add_memory("My favorite color is red.").unwrap();
        db.add_memory("I like apples.").unwrap();
        db.add_memory("The sky is blue.").unwrap();

        let results = db.search("fruit", 1).unwrap();
        assert_eq!(results[0].text, "I like apples.");
    }
}
