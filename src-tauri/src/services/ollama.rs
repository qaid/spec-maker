use reqwest::Client;
use serde::{Deserialize, Serialize};

const OLLAMA_BASE_URL: &str = "http://localhost:11434";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            model: "llama3.1:8b".to_string(),
            temperature: 0.7,
            max_tokens: Some(4096),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    options: ChatOptions,
}

#[derive(Debug, Serialize)]
struct ChatOptions {
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub done: bool,
}

pub struct OllamaService {
    client: Client,
    config: OllamaConfig,
}

impl OllamaService {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String, String> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            stream: false,
            options: ChatOptions {
                temperature: self.config.temperature,
                num_predict: self.config.max_tokens,
            },
        };

        let response = self
            .client
            .post(format!("{}/api/chat", OLLAMA_BASE_URL))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama API error: {}", response.status()));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(chat_response.message.content)
    }

    pub async fn check_connection(&self) -> Result<bool, String> {
        let response = self
            .client
            .get(format!("{}/api/tags", OLLAMA_BASE_URL))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        Ok(response.status().is_success())
    }
}
