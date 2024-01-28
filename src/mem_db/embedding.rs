use super::MemoryDBError;

pub const EMBEDDING_DIM: usize = 384;
pub const EMBEDDING_DIM_BYTES: usize = EMBEDDING_DIM * 4;

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEmbedding([f32; EMBEDDING_DIM]);
impl MemoryEmbedding {
    pub fn empty() -> Self {
        Self::from_slice([0f32; EMBEDDING_DIM])
    }

    pub fn from_slice(array: [f32; EMBEDDING_DIM]) -> Self {
        Self(array)
    }

    pub fn try_from_slice(array: &[f32]) -> Result<Self, MemoryDBError> {
        array
            .try_into()
            .map_err(|_| MemoryDBError::WrongEmbeddingSize {
                expected: EMBEDDING_DIM,
                actual: array.len(),
            })
            .map(Self)
    }

    pub fn as_bytes(&self) -> [u8; EMBEDDING_DIM_BYTES] {
        let mut bytes = [0u8; EMBEDDING_DIM_BYTES];
        for (i, f) in self.0.iter().enumerate() {
            let f_bytes = f.to_ne_bytes();
            bytes[i * 4 .. (i + 1) * 4].copy_from_slice(&f_bytes);
        }
        bytes
    }
}

pub struct VectorMemory {
    text: String,
    embedding: MemoryEmbedding,
}

impl VectorMemory {
    pub fn new(text: String, embedding: MemoryEmbedding) -> Self {
        Self { text, embedding }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn embedding(&self) -> &MemoryEmbedding {
        &self.embedding
    }
}

#[derive(Debug, Clone)]
pub struct RecalledMemory {
    pub memory: String,
    pub distance: f32,
}
