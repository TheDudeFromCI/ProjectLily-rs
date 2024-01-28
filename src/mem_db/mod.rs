mod embedding;
mod error;
mod io;
mod log;

use rusqlite::Connection;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder,
    SentenceEmbeddingsModel,
    SentenceEmbeddingsModelType,
};
use tch::Device;

use self::embedding::{MemoryEmbedding, RecalledMemory, VectorMemory};
pub use self::error::MemoryDBError;
use self::log::MessageLog;
use crate::llm::CompletionSettings;
use crate::prompt::ChatMessage;

pub struct MemoryDB {
    log: MessageLog,
    conn: Connection,
    embeddings_model: SentenceEmbeddingsModel,
}

impl MemoryDB {
    pub async fn new() -> Result<Self, MemoryDBError> {
        let model = tokio::task::spawn_blocking(|| {
            SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
                .with_device(Device::Cpu)
                .create_model()
        })
        .await??;

        let mut s = Self {
            log: MessageLog::new(),
            conn: io::open_connection()?,
            embeddings_model: model,
        };

        for message in io::get_recent_logs(&s.conn, 4096)? {
            s.log.add_message(message);
        }

        Ok(s)
    }

    fn embed(&self, text: &str) -> Result<MemoryEmbedding, MemoryDBError> {
        let embeddings = self.embeddings_model.encode(&[text])?;
        let tensor = embeddings[0].as_slice();
        let embed = MemoryEmbedding::try_from_slice(tensor)?;
        Ok(embed)
    }

    pub fn update_pre_prompt(&mut self, pre_prompt: String, tokens: usize) {
        self.log.update_pre_prompt(pre_prompt, tokens);
    }

    pub fn add_vector_memory(&mut self, content: &str) -> Result<(), MemoryDBError> {
        let embedding = self.embed(content)?;
        let memory = VectorMemory::new(content.to_string(), embedding);
        io::save_vector_mem(&self.conn, &memory)?;
        Ok(())
    }

    pub fn search_vector_memory(
        &self,
        query: &str,
        count: usize,
        max_distance: f32,
    ) -> Result<Vec<RecalledMemory>, MemoryDBError> {
        let search = self.embed(query)?;
        io::recall_memories(&self.conn, &search, count, max_distance)
    }

    pub fn add_log_memory(&mut self, message: ChatMessage) -> Result<(), MemoryDBError> {
        io::append_to_log(&self.conn, &message)?;
        self.log.add_message(message);
        Ok(())
    }

    pub fn get_log_prompt(&self, settings: &CompletionSettings) -> String {
        self.log.format(settings)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn simple_db() {
        let mut db = MemoryDB::new().await.unwrap();

        db.add_vector_memory("My favorite color is red.").unwrap();
        db.add_vector_memory("I like apples.").unwrap();
        db.add_vector_memory("The sky is blue.").unwrap();

        let results = db.search_vector_memory("fruit", 1, 10.0).unwrap();
        assert_eq!(results[0].memory, "I like apples.");
    }
}
