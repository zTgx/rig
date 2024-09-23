//! Cohere API client and Rig integration
//!
//! # Example
//! ```
//! use rig::providers::cohere;
//!
//! let client = cohere::Client::new("YOUR_API_KEY");
//!
//! let command_r = client.completion_model(cohere::COMMAND_R);
//! ```
use std::collections::HashMap;

use crate::{
    agent::AgentBuilder,
    completion::{self, CompletionError},
    embeddings::{self, EmbeddingError, EmbeddingsBuilder},
    extractor::ExtractorBuilder,
    json_utils,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

// ================================================================
// Main Cohere Client
// ================================================================
const COHERE_API_BASE_URL: &str = "https://api.cohere.ai";

#[derive(Clone)]
pub struct Client {
    base_url: String,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(api_key: &str) -> Self {
        Self::from_url(api_key, COHERE_API_BASE_URL)
    }

    pub fn from_url(api_key: &str, base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            http_client: reqwest::Client::builder()
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        "Authorization",
                        format!("Bearer {}", api_key)
                            .parse()
                            .expect("Bearer token should parse"),
                    );
                    headers
                })
                .build()
                .expect("Cohere reqwest client should build"),
        }
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/{}", self.base_url, path).replace("//", "/");
        self.http_client.post(url)
    }

    pub fn embedding_model(
        &self,
        model: &CohereEmbeddingModel,
        input_type: &str,
    ) -> EmbeddingModel {
        EmbeddingModel::new(self.clone(), model, input_type)
    }

    pub fn embeddings(
        &self,
        model: &CohereEmbeddingModel,
        input_type: &str,
    ) -> EmbeddingsBuilder<EmbeddingModel> {
        EmbeddingsBuilder::new(self.embedding_model(model, input_type))
    }

    pub fn completion_model(&self, model: &str) -> CompletionModel {
        CompletionModel::new(self.clone(), model)
    }

    #[deprecated(
        since = "0.2.0",
        note = "Please use the `agent` method instead of the `model` method."
    )]
    pub fn model(&self, model: &str) -> AgentBuilder<CompletionModel> {
        AgentBuilder::new(self.completion_model(model))
    }

    pub fn agent(&self, model: &str) -> AgentBuilder<CompletionModel> {
        AgentBuilder::new(self.completion_model(model))
    }

    pub fn extractor<T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync>(
        &self,
        model: &str,
    ) -> ExtractorBuilder<T, CompletionModel> {
        ExtractorBuilder::new(self.completion_model(model))
    }

    #[deprecated(
        since = "0.2.0",
        note = "Please use the `agent` method instead of the `rag_agent` method."
    )]
    pub fn rag_agent(&self, model: &str) -> AgentBuilder<CompletionModel> {
        AgentBuilder::new(self.completion_model(model))
    }

    #[deprecated(
        since = "0.2.0",
        note = "Please use the `agent` method instead of the `tool_rag_agent` method."
    )]
    pub fn tool_rag_agent(&self, model: &str) -> AgentBuilder<CompletionModel> {
        AgentBuilder::new(self.completion_model(model))
    }

    #[deprecated(
        since = "0.2.0",
        note = "Please use the `agent` method instead of the `context_rag_agent` method."
    )]
    pub fn context_rag_agent(&self, model: &str) -> AgentBuilder<CompletionModel> {
        AgentBuilder::new(self.completion_model(model))
    }
}

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ApiResponse<T> {
    Ok(T),
    Err(ApiErrorResponse),
}

// ================================================================
// Cohere Embedding API
// ================================================================
#[derive(Debug, Clone)]
pub enum CohereEmbeddingModel {
    EmbedEnglishV3,
    EmbedEnglishLightV3,
    EmbedMultilingualV3,
    EmbedMultilingualLightV3,
    EmbedEnglishV2,
    EmbedEnglishLightV2,
    EmbedMultilingualV2,
}

impl std::str::FromStr for CohereEmbeddingModel {
    type Err = EmbeddingError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "embed-english-v3.0" => Ok(Self::EmbedEnglishV3),
            "embed-english-light-v3.0" => Ok(Self::EmbedEnglishLightV3),
            "embed-multilingual-v3.0" => Ok(Self::EmbedMultilingualV3),
            "embed-multilingual-light-v3.0" => Ok(Self::EmbedMultilingualLightV3),
            "embed-english-v2.0" => Ok(Self::EmbedEnglishV2),
            "embed-english-light-v2.0" => Ok(Self::EmbedEnglishLightV2),
            "embed-multilingual-v2.0" => Ok(Self::EmbedMultilingualV2),
            _ => Err(EmbeddingError::BadModel(s.to_string())),
        }
    }
}

impl std::fmt::Display for CohereEmbeddingModel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::EmbedEnglishLightV3 => write!(f, "embed-english-light-v3.0"),
            Self::EmbedEnglishV3 => write!(f, "embed-english-v3.0"),
            Self::EmbedMultilingualLightV3 => write!(f, "embed-multilingual-light-v3.0"),
            Self::EmbedMultilingualV3 => write!(f, "embed-multilingual-v3.0"),
            Self::EmbedEnglishV2 => write!(f, "embed-english-v2.0"),
            Self::EmbedEnglishLightV2 => write!(f, "embed-english-light-v2.0"),
            Self::EmbedMultilingualV2 => write!(f, "embed-multilingual-v2.0"),
        }
    }
}

#[derive(Deserialize)]
pub struct EmbeddingResponse {
    #[serde(default)]
    pub response_type: Option<String>,
    pub id: String,
    pub embeddings: Vec<Vec<f64>>,
    pub texts: Vec<String>,
    #[serde(default)]
    pub meta: Option<Meta>,
}

#[derive(Deserialize)]
pub struct Meta {
    pub api_version: ApiVersion,
    pub billed_units: BilledUnits,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Deserialize)]
pub struct ApiVersion {
    pub version: String,
    #[serde(default)]
    pub is_deprecated: Option<bool>,
    #[serde(default)]
    pub is_experimental: Option<bool>,
}

#[derive(Deserialize)]
pub struct BilledUnits {
    #[serde(default)]
    pub input_tokens: u32,
    #[serde(default)]
    pub output_tokens: u32,
    #[serde(default)]
    pub search_units: u32,
    #[serde(default)]
    pub classifications: u32,
}

#[derive(Clone)]
pub struct EmbeddingModel {
    client: Client,
    pub model: CohereEmbeddingModel,
    pub input_type: String,
}

impl embeddings::EmbeddingModel for EmbeddingModel {
    const MAX_DOCUMENTS: usize = 96;

    fn ndims(&self) -> usize {
        match self.model {
            CohereEmbeddingModel::EmbedEnglishV3 => 1024,
            CohereEmbeddingModel::EmbedEnglishLightV3 => 384,
            CohereEmbeddingModel::EmbedMultilingualV3 => 1024,
            CohereEmbeddingModel::EmbedMultilingualLightV3 => 384,
            CohereEmbeddingModel::EmbedEnglishV2 => 4096,
            CohereEmbeddingModel::EmbedEnglishLightV2 => 1024,
            CohereEmbeddingModel::EmbedMultilingualV2 => 768,
        }
    }

    async fn embed_documents(
        &self,
        documents: Vec<String>,
    ) -> Result<Vec<embeddings::Embedding>, EmbeddingError> {
        let response = self
            .client
            .post("/v1/embed")
            .json(&json!({
                "model": self.model.to_string(),
                "texts": documents,
                "input_type": self.input_type,
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<ApiResponse<EmbeddingResponse>>()
            .await?;

        match response {
            ApiResponse::Ok(response) => {
                if response.embeddings.len() != documents.len() {
                    return Err(EmbeddingError::DocumentError(format!(
                        "Expected {} embeddings, got {}",
                        documents.len(),
                        response.embeddings.len()
                    )));
                }

                Ok(response
                    .embeddings
                    .into_iter()
                    .zip(documents.into_iter())
                    .map(|(embedding, document)| embeddings::Embedding {
                        document,
                        vec: embedding,
                    })
                    .collect())
            }
            ApiResponse::Err(error) => Err(EmbeddingError::ProviderError(error.message)),
        }
    }
}

impl EmbeddingModel {
    pub fn new(client: Client, model: &CohereEmbeddingModel, input_type: &str) -> Self {
        Self {
            client,
            model: model.clone(),
            input_type: input_type.to_string(),
        }
    }
}

// ================================================================
// Cohere Completion API
// ================================================================
/// `command-r-plus` completion model
pub const COMMAND_R_PLUS: &str = "comman-r-plus";
/// `command-r` completion model
pub const COMMAND_R: &str = "command-r";
/// `command` completion model
pub const COMMAND: &str = "command";
/// `command-nightly` completion model
pub const COMMAND_NIGHTLY: &str = "command-nightly";
/// `command-light` completion model
pub const COMMAND_LIGHT: &str = "command-light";
/// `command-light-nightly` completion model
pub const COMMAND_LIGHT_NIGHTLY: &str = "command-light-nightly";

#[derive(Debug, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub generation_id: String,
    #[serde(default)]
    pub citations: Vec<Citation>,
    #[serde(default)]
    pub documents: Vec<Document>,
    #[serde(default)]
    pub is_search_required: Option<bool>,
    #[serde(default)]
    pub search_queries: Vec<SearchQuery>,
    #[serde(default)]
    pub search_results: Vec<SearchResult>,
    pub finish_reason: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default)]
    pub chat_history: Vec<ChatHistory>,
}

impl From<CompletionResponse> for completion::CompletionResponse<CompletionResponse> {
    fn from(response: CompletionResponse) -> Self {
        let CompletionResponse {
            text, tool_calls, ..
        } = &response;

        let model_response = if !tool_calls.is_empty() {
            completion::ModelChoice::ToolCall(
                tool_calls.first().unwrap().name.clone(),
                tool_calls.first().unwrap().parameters.clone(),
            )
        } else {
            completion::ModelChoice::Message(text.clone())
        };

        completion::CompletionResponse {
            choice: model_response,
            raw_response: response,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Citation {
    pub start: u32,
    pub end: u32,
    pub text: String,
    pub document_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Document {
    pub id: String,
    #[serde(flatten)]
    pub additional_prop: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub generation_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub search_query: SearchQuery,
    pub connector: Connector,
    pub document_ids: Vec<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub continue_on_failure: bool,
}

#[derive(Debug, Deserialize)]
pub struct Connector {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ChatHistory {
    pub role: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Parameter {
    pub description: String,
    pub r#type: String,
    pub required: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameter_definitions: HashMap<String, Parameter>,
}

impl From<completion::ToolDefinition> for ToolDefinition {
    fn from(tool: completion::ToolDefinition) -> Self {
        fn convert_type(r#type: &serde_json::Value) -> String {
            fn convert_type_str(r#type: &str) -> String {
                match r#type {
                    "string" => "string".to_owned(),
                    "number" => "number".to_owned(),
                    "integer" => "integer".to_owned(),
                    "boolean" => "boolean".to_owned(),
                    "array" => "array".to_owned(),
                    "object" => "object".to_owned(),
                    _ => "string".to_owned(),
                }
            }
            match r#type {
                serde_json::Value::String(r#type) => convert_type_str(r#type.as_str()),
                serde_json::Value::Array(types) => convert_type_str(
                    types
                        .iter()
                        .find(|t| t.as_str() != Some("null"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("string"),
                ),
                _ => "string".to_owned(),
            }
        }

        let maybe_required = tool
            .parameters
            .get("required")
            .and_then(|v| v.as_array())
            .map(|required| {
                required
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Self {
            name: tool.name,
            description: tool.description,
            parameter_definitions: tool
                .parameters
                .get("properties")
                .expect("Tool properties should exist")
                .as_object()
                .expect("Tool properties should be an object")
                .iter()
                .map(|(argname, argdef)| {
                    (
                        argname.clone(),
                        Parameter {
                            description: argdef
                                .get("description")
                                .expect("Argument description should exist")
                                .as_str()
                                .expect("Argument description should be a string")
                                .to_string(),
                            r#type: convert_type(
                                argdef.get("type").expect("Argument type should exist"),
                            ),
                            required: maybe_required.contains(&argname.as_str()),
                        },
                    )
                })
                .collect::<HashMap<_, _>>(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub message: String,
}

impl From<completion::Message> for Message {
    fn from(message: completion::Message) -> Self {
        Self {
            role: match message.role.as_str() {
                "system" => "SYSTEM".to_owned(),
                "user" => "USER".to_owned(),
                "assistant" => "CHATBOT".to_owned(),
                _ => "USER".to_owned(),
            },
            message: message.content,
        }
    }
}

#[derive(Clone)]
pub struct CompletionModel {
    client: Client,
    pub model: String,
}

impl CompletionModel {
    pub fn new(client: Client, model: &str) -> Self {
        Self {
            client,
            model: model.to_string(),
        }
    }
}

impl completion::CompletionModel for CompletionModel {
    type Response = CompletionResponse;

    async fn completion(
        &self,
        completion_request: completion::CompletionRequest,
    ) -> Result<completion::CompletionResponse<CompletionResponse>, CompletionError> {
        let request = json!({
            "model": self.model,
            "preamble": completion_request.preamble,
            "message": completion_request.prompt,
            "documents": completion_request.documents,
            "chat_history": completion_request.chat_history.into_iter().map(Message::from).collect::<Vec<_>>(),
            "temperature": completion_request.temperature,
            "tools": completion_request.tools.into_iter().map(ToolDefinition::from).collect::<Vec<_>>(),
        });

        let response = self
            .client
            .post("/v1/chat")
            .json(
                &if let Some(ref params) = completion_request.additional_params {
                    json_utils::merge(request.clone(), params.clone())
                } else {
                    request.clone()
                },
            )
            .send()
            .await?
            .error_for_status()?
            .json::<ApiResponse<CompletionResponse>>()
            .await?;

        match response {
            ApiResponse::Ok(completion) => Ok(completion.into()),
            ApiResponse::Err(error) => Err(CompletionError::ProviderError(error.message)),
        }
    }
}
