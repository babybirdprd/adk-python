//! Google AI/Gemini LLM implementation

use crate::{
    error::Result,
    models::{BaseLlm, LlmRequest, LlmResponse, FinishReason, Usage},
    types::{Content, ContentPart, FunctionCall},
};
use async_trait::async_trait;
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{pin::Pin, time::Duration};
use tracing::{debug, error, info, warn};

/// Google AI/Gemini LLM implementation
#[derive(Debug, Clone)]
pub struct GoogleLlm {
    model: String,
    api_key: Option<String>,
    project_id: Option<String>,
    region: Option<String>,
    client: Client,
    base_url: String,
}

/// Google AI API request format
#[derive(Debug, Serialize)]
struct GoogleAiRequest {
    contents: Vec<GoogleAiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GoogleAiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GoogleAiGenerationConfig>,
}

#[derive(Debug, Serialize)]
struct GoogleAiContent {
    role: String,
    parts: Vec<GoogleAiPart>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum GoogleAiPart {
    Text { text: String },
    FunctionCall { function_call: GoogleAiFunctionCall },
    FunctionResponse { function_response: GoogleAiFunctionResponse },
}

#[derive(Debug, Serialize)]
struct GoogleAiFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct GoogleAiFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct GoogleAiTool {
    function_declarations: Vec<GoogleAiFunctionDeclaration>,
}

#[derive(Debug, Serialize)]
struct GoogleAiFunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct GoogleAiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop_sequences: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_schema: Option<serde_json::Value>,
}

/// Google AI API response format
#[derive(Debug, Deserialize)]
struct GoogleAiResponse {
    candidates: Vec<GoogleAiCandidate>,
    #[serde(default)]
    usage_metadata: Option<GoogleAiUsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct GoogleAiCandidate {
    content: GoogleAiResponseContent,
    finish_reason: Option<String>,
    safety_ratings: Option<Vec<GoogleAiSafetyRating>>,
}

#[derive(Debug, Deserialize)]
struct GoogleAiResponseContent {
    parts: Vec<GoogleAiResponsePart>,
    role: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GoogleAiResponsePart {
    Text { text: String },
    FunctionCall { function_call: GoogleAiResponseFunctionCall },
}

#[derive(Debug, Deserialize)]
struct GoogleAiResponseFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GoogleAiSafetyRating {
    category: String,
    probability: String,
}

#[derive(Debug, Deserialize)]
struct GoogleAiUsageMetadata {
    prompt_token_count: Option<u32>,
    candidates_token_count: Option<u32>,
    total_token_count: Option<u32>,
}

impl GoogleLlm {
    /// Create a new Google LLM instance
    pub fn new(model: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            model: model.into(),
            api_key: None,
            project_id: None,
            region: None,
            client,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        }
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set project ID (for Vertex AI)
    pub fn with_project_id(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    /// Set region (for Vertex AI)
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Use Vertex AI endpoint
    pub fn use_vertex_ai(mut self) -> Self {
        if let (Some(project), Some(region)) = (&self.project_id, &self.region) {
            self.base_url = format!(
                "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models",
                region, project, region
            );
        }
        self
    }

    /// Convert ADK request to Google AI format
    fn convert_request(&self, request: &LlmRequest) -> GoogleAiRequest {
        let contents = request.contents.iter().map(|content| {
            let parts = content.parts.iter().map(|part| {
                match part {
                    ContentPart::Text { text } => GoogleAiPart::Text { text: text.clone() },
                    _ => GoogleAiPart::Text { text: "[Unsupported content type]".to_string() },
                }
            }).collect();

            GoogleAiContent {
                role: content.role.clone(),
                parts,
            }
        }).collect();

        let tools = if !request.config.tools.is_empty() {
            Some(request.config.tools.iter().map(|tool| {
                GoogleAiTool {
                    function_declarations: tool.function_declarations.iter().map(|decl| {
                        GoogleAiFunctionDeclaration {
                            name: decl.name.clone(),
                            description: decl.description.clone(),
                            parameters: decl.parameters.clone(),
                        }
                    }).collect(),
                }
            }).collect())
        } else {
            None
        };

        let generation_config = Some(GoogleAiGenerationConfig {
            temperature: request.config.temperature,
            top_p: request.config.top_p,
            top_k: request.config.top_k,
            max_output_tokens: request.config.max_output_tokens,
            stop_sequences: request.config.stop_sequences.clone(),
            response_mime_type: request.config.response_mime_type.clone(),
            response_schema: request.config.response_schema.clone(),
        });

        GoogleAiRequest {
            contents,
            tools,
            generation_config,
        }
    }

    /// Convert Google AI response to ADK format
    fn convert_response(&self, response: GoogleAiResponse) -> Result<LlmResponse> {
        if response.candidates.is_empty() {
            return Ok(LlmResponse::new());
        }

        let candidate = &response.candidates[0];
        let mut llm_response = LlmResponse::new();

        // Convert content
        if !candidate.content.parts.is_empty() {
            let mut text_parts = Vec::new();
            let mut function_calls = Vec::new();

            for part in &candidate.content.parts {
                match part {
                    GoogleAiResponsePart::Text { text } => {
                        text_parts.push(text.clone());
                    }
                    GoogleAiResponsePart::FunctionCall { function_call } => {
                        function_calls.push(FunctionCall {
                            name: function_call.name.clone(),
                            args: function_call.args.clone(),
                        });
                    }
                }
            }

            if !text_parts.is_empty() {
                llm_response.content = Some(Content::model_text(text_parts.join("")));
            }

            llm_response.function_calls = function_calls;
        }

        // Convert finish reason
        if let Some(finish_reason) = &candidate.finish_reason {
            llm_response.finish_reason = Some(match finish_reason.as_str() {
                "STOP" => FinishReason::Stop,
                "MAX_TOKENS" => FinishReason::MaxTokens,
                "SAFETY" => FinishReason::Safety,
                "RECITATION" => FinishReason::Recitation,
                _ => FinishReason::Other,
            });
        }

        // Convert usage
        if let Some(usage_metadata) = response.usage_metadata {
            llm_response.usage = Some(Usage {
                prompt_tokens: usage_metadata.prompt_token_count,
                completion_tokens: usage_metadata.candidates_token_count,
                total_tokens: usage_metadata.total_token_count,
            });
        }

        Ok(llm_response)
    }

    /// Get the API endpoint URL
    fn get_endpoint_url(&self) -> String {
        if self.project_id.is_some() && self.region.is_some() {
            // Vertex AI endpoint
            format!("{}/{}:generateContent", self.base_url, self.model)
        } else {
            // Google AI endpoint
            format!("{}/models/{}:generateContent", self.base_url, self.model)
        }
    }

    /// Get authentication header
    fn get_auth_header(&self) -> Result<String> {
        if let Some(api_key) = &self.api_key {
            Ok(format!("Bearer {}", api_key))
        } else if let Ok(token) = std::env::var("GOOGLE_API_KEY") {
            Ok(format!("Bearer {}", token))
        } else {
            Err(crate::adk_error!(
                AuthError,
                "No API key provided. Set GOOGLE_API_KEY environment variable or use with_api_key()"
            ))
        }
    }
}

#[async_trait]
impl BaseLlm for GoogleLlm {
    fn model_name(&self) -> &str {
        &self.model
    }

    fn supported_models() -> Vec<String> {
        vec![
            r"gemini-.*".to_string(),
            r"gemini-pro.*".to_string(),
            r"gemini-flash.*".to_string(),
            r"gemini-2\.0-.*".to_string(),
        ]
    }

    async fn generate_content(&self, request: LlmRequest) -> Result<LlmResponse> {
        debug!("Generating content with Google AI for model: {}", self.model);

        let google_request = self.convert_request(&request);
        let url = self.get_endpoint_url();
        let auth_header = self.get_auth_header()?;

        let response = self.client
            .post(&url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&google_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Google AI API error: {} - {}", status, error_text);
            return Err(crate::adk_error!(
                ModelError,
                "Google AI API error: {} - {}",
                status,
                error_text
            ));
        }

        let google_response: GoogleAiResponse = response.json().await?;
        let llm_response = self.convert_response(google_response)?;

        info!("Successfully generated content with Google AI");
        Ok(llm_response)
    }

    async fn generate_content_stream(
        &self,
        request: LlmRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<LlmResponse>> + Send>>> {
        // For now, just return the non-streaming response as a single item stream
        // TODO: Implement actual streaming support
        warn!("Streaming not yet implemented for Google AI, falling back to non-streaming");
        let response = self.generate_content(request).await?;
        Ok(Box::pin(futures::stream::once(async move { Ok(response) })))
    }

    fn supports_streaming(&self) -> bool {
        true // Google AI supports streaming, but not implemented yet
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    fn supports_multimodal(&self) -> bool {
        self.model.contains("pro") || self.model.contains("flash") || self.model.contains("2.0")
    }

    fn supports_live(&self) -> bool {
        self.model.contains("2.0")
    }
}
