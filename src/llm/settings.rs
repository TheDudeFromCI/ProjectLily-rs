use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionSettings {
    pub model: Option<String>,
    pub context_length: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub min_p: f32,
    pub top_k: i32,
    pub seed: Option<u64>,
    pub stop_tokens: Vec<String>,
    pub max_tokens: i32,
    pub repeat_penalty: f32,
    pub repeat_last_n: i32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub logit_bias: HashMap<i32, f32>,
    pub system_message_prefix: String,
    pub system_message_suffix: String,
    pub user_message_prefix: String,
    pub user_message_suffix: String,
    pub assistant_message_prefix: String,
    pub assistant_message_suffix: String,
}

impl Default for CompletionSettings {
    fn default() -> Self {
        Self {
            model: None,
            context_length: 2048,
            temperature: 0.7,
            top_p: 1.0,
            min_p: 0.05,
            top_k: 40,
            seed: None,
            stop_tokens: vec![String::from("\n")],
            max_tokens: 128,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            logit_bias: HashMap::new(),
            system_message_prefix: String::from("### system\n"),
            system_message_suffix: String::from("\n"),
            user_message_prefix: String::from("### user\n"),
            user_message_suffix: String::from("\n"),
            assistant_message_prefix: String::from("### assistant\n"),
            assistant_message_suffix: String::from("\n"),
        }
    }
}
