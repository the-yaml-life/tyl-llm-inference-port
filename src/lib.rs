//! # TYL LLM Inference
//!
//! Template-based LLM inference port for TYL framework following hexagonal architecture.
//!
//! This module provides a simplified abstraction for LLM inference: you provide a template with
//! parameters and get back a JSON response with metadata. It replaces complex completion/chat
//! patterns with a single, unified interface.
//!
//! ## Features
//!
//! - **Template-based inference** - Simple template + parameters → JSON response
//! - **Model optimization** - Different models for different use cases
//! - **Token management** - Token counting and usage tracking
//! - **TYL framework integration** - Uses TYL error handling, config, logging, and tracing
//! - **Mock implementation** - Testing support with in-memory mock service
//!
//! ## Quick Start
//!
//! ```rust
//! use tyl_llm_inference_port::{InferenceService, InferenceRequest, ModelType};
//! use std::collections::HashMap;
//!
//! # async fn example() -> tyl_llm_inference_port::TylResult<()> {
//! // Implementation would be provided by an adapter
//! // let service = SomeInferenceAdapter::new();
//! // let mut params = HashMap::new();
//! // params.insert("name".to_string(), "Juan".to_string());
//! // let request = InferenceRequest::new("Hello {{name}}!", params, ModelType::General);
//! // let response = service.infer(request).await?;
//! // println!("Response: {}", response.content);
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! This module follows hexagonal architecture:
//!
//! - **Port (Interface)**: `InferenceService` - defines the contract
//! - **Adapters**: HTTP-based services (OpenAI, Anthropic), local model adapters
//! - **Domain Logic**: Template processing and response handling
//!
//! ## Model Types
//!
//! Different model types use optimized models for better performance:
//!
//! - **Coding** - Code generation and programming tasks
//! - **Reasoning** - Complex reasoning and analysis
//! - **General** - General text generation and conversation
//! - **Fast** - Quick responses for simple tasks
//! - **Creative** - Creative writing and content generation
//!
//! ## Examples
//!
//! See the `examples/` directory for complete usage examples.

// Re-export TYL framework functionality
pub use tyl_errors::{TylError, TylResult};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple health status for inference services
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy { reason: String },
}

impl HealthStatus {
    pub fn healthy() -> Self {
        Self::Healthy
    }

    pub fn unhealthy(reason: impl Into<String>) -> Self {
        Self::Unhealthy {
            reason: reason.into(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }
}

/// Health check result for inference services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl HealthCheckResult {
    pub fn new(status: HealthStatus) -> Self {
        Self {
            status,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Type alias for inference operations using TYL unified error handling
pub type InferenceResult<T> = TylResult<T>;

/// Inference-specific error helpers that extend TylError
pub mod inference_errors {
    use super::*;

    /// Create an inference generation error
    pub fn generation_failed(message: impl Into<String>) -> TylError {
        TylError::internal(format!("Inference generation failed: {}", message.into()))
    }

    /// Create an invalid model type error
    pub fn invalid_model_type(model_type: impl Into<String>) -> TylError {
        TylError::validation(
            "model_type",
            format!("Invalid model type: {}", model_type.into()),
        )
    }

    /// Create a token limit exceeded error
    pub fn token_limit_exceeded(limit: usize, requested: usize) -> TylError {
        TylError::validation(
            "token_limit",
            format!("Token limit {limit} exceeded, requested {requested}"),
        )
    }

    /// Create an API rate limit error
    pub fn rate_limit_exceeded(provider: impl Into<String>) -> TylError {
        TylError::network(format!("{} rate limit exceeded", provider.into()))
    }

    /// Create an invalid API key error
    pub fn invalid_api_key(provider: impl Into<String>) -> TylError {
        TylError::configuration(format!("Invalid API key for {}", provider.into()))
    }

    /// Create a context window exceeded error
    pub fn context_window_exceeded(max_tokens: usize, actual_tokens: usize) -> TylError {
        TylError::validation(
            "context_window",
            format!("Context window {max_tokens} exceeded with {actual_tokens} tokens"),
        )
    }

    /// Create an unsupported model error
    pub fn unsupported_model(model: impl Into<String>) -> TylError {
        TylError::validation("model", format!("Unsupported model: {}", model.into()))
    }

    /// Create a template processing error
    pub fn template_processing_failed(message: impl Into<String>) -> TylError {
        TylError::validation("template", format!("Template processing failed: {}", message.into()))
    }
}

/// Model types for inference optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ModelType {
    /// Code generation and programming tasks
    Coding,
    /// Complex reasoning and analysis
    Reasoning,
    /// General text generation and conversation
    #[default]
    General,
    /// Quick responses for simple tasks
    Fast,
    /// Creative writing and content generation
    Creative,
}

impl ModelType {
    /// Get optimal model for this type with OpenAI provider
    pub fn optimal_openai_model(&self) -> &'static str {
        match self {
            ModelType::Coding => "gpt-4o",       // Code-optimized
            ModelType::Reasoning => "gpt-4o",    // Best reasoning
            ModelType::General => "gpt-4o-mini", // Balanced
            ModelType::Fast => "gpt-3.5-turbo",  // Speed optimized
            ModelType::Creative => "gpt-4o",     // Creative tasks
        }
    }

    /// Get optimal model for this type with Anthropic provider
    pub fn optimal_anthropic_model(&self) -> &'static str {
        match self {
            ModelType::Coding => "claude-3-5-sonnet-20241022", // Code-optimized
            ModelType::Reasoning => "claude-3-5-sonnet-20241022", // Best reasoning
            ModelType::General => "claude-3-5-haiku-20241022", // Balanced
            ModelType::Fast => "claude-3-5-haiku-20241022",    // Speed optimized
            ModelType::Creative => "claude-3-5-sonnet-20241022", // Creative tasks
        }
    }

    /// Get typical max tokens for this model type
    pub fn typical_max_tokens(&self) -> usize {
        match self {
            ModelType::Coding => 4096,    // Longer code completions
            ModelType::Reasoning => 8192, // Complex reasoning
            ModelType::General => 2048,   // Standard responses
            ModelType::Fast => 1024,      // Quick responses
            ModelType::Creative => 4096,  // Creative content
        }
    }
}

/// Template-based inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Template with placeholders like "Hello {{name}}!"
    pub template: String,
    /// Parameters to replace in template
    pub parameters: HashMap<String, String>,
    /// Model type for optimization
    pub model_type: ModelType,
    /// Optional model override
    pub model_override: Option<String>,
    /// Maximum tokens to generate
    pub max_tokens: Option<usize>,
    /// Temperature for randomness (0.0 to 1.0)
    pub temperature: Option<f32>,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

impl InferenceRequest {
    pub fn new(
        template: impl Into<String>, 
        parameters: HashMap<String, String>, 
        model_type: ModelType
    ) -> Self {
        Self {
            template: template.into(),
            parameters,
            model_type,
            model_override: None,
            max_tokens: Some(model_type.typical_max_tokens()),
            temperature: Some(0.7),
            metadata: HashMap::new(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model_override = Some(model.into());
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.clamp(0.0, 1.0));
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Process template with parameters to create the final prompt
    pub fn render_template(&self) -> String {
        let mut rendered = self.template.clone();
        for (key, value) in &self.parameters {
            let placeholder = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }
        rendered
    }
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

/// Response metadata containing processing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Model used for generation
    pub model: String,
    /// Token usage information
    pub token_usage: TokenUsage,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Response timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ResponseMetadata {
    pub fn new(
        model: String,
        token_usage: TokenUsage,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            model,
            token_usage,
            processing_time_ms,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Inference response containing JSON content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    /// Generated JSON content
    pub content: serde_json::Value,
    /// Response metadata
    pub metadata: ResponseMetadata,
}

impl InferenceResponse {
    pub fn new(
        content: serde_json::Value,
        metadata: ResponseMetadata,
    ) -> Self {
        Self {
            content,
            metadata,
        }
    }

    /// Create response with string content (will be converted to JSON string value)
    pub fn from_string(
        content: String,
        model: String,
        token_usage: TokenUsage,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            content: serde_json::Value::String(content),
            metadata: ResponseMetadata::new(model, token_usage, processing_time_ms),
        }
    }

    /// Try to parse content as JSON, fallback to string if parsing fails
    pub fn from_text_with_json_fallback(
        content: String,
        model: String,
        token_usage: TokenUsage,
        processing_time_ms: u64,
    ) -> Self {
        let json_content = match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(json) => json,
            Err(_) => serde_json::Value::String(content),
        };
        
        Self {
            content: json_content,
            metadata: ResponseMetadata::new(model, token_usage, processing_time_ms),
        }
    }
}

/// Template-based inference service trait
#[async_trait]
pub trait InferenceService: Send + Sync {
    /// Generate inference response from template and parameters
    async fn infer(&self, request: InferenceRequest) -> InferenceResult<InferenceResponse>;

    /// Check if service is healthy
    async fn health_check(&self) -> InferenceResult<HealthCheckResult>;

    /// Get supported models
    fn supported_models(&self) -> Vec<String>;

    /// Count tokens in text (approximate)
    fn count_tokens(&self, text: &str) -> InferenceResult<usize>;
}

// Mock adapter for testing and demonstration
#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "mock")]
pub use mock::MockInferenceService;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_request_creation() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), "Juan".to_string());
        
        let request = InferenceRequest::new("Hello {{name}}!", params, ModelType::General)
            .with_max_tokens(100)
            .with_temperature(0.5);

        assert_eq!(request.template, "Hello {{name}}!");
        assert_eq!(request.model_type, ModelType::General);
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.5));
        assert_eq!(request.parameters.get("name"), Some(&"Juan".to_string()));
    }

    #[test]
    fn test_template_rendering() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), "Juan".to_string());
        params.insert("age".to_string(), "30".to_string());
        
        let request = InferenceRequest::new("Hello {{name}}, you are {{age}} years old!", params, ModelType::General);
        let rendered = request.render_template();
        
        assert_eq!(rendered, "Hello Juan, you are 30 years old!");
    }

    #[test]
    fn test_model_type_optimal_models() {
        assert_eq!(ModelType::Coding.optimal_openai_model(), "gpt-4o");
        assert_eq!(ModelType::Fast.optimal_openai_model(), "gpt-3.5-turbo");
        assert_eq!(
            ModelType::Reasoning.optimal_anthropic_model(),
            "claude-3-5-sonnet-20241022"
        );
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(50, 100);
        assert_eq!(usage.prompt_tokens, 50);
        assert_eq!(usage.completion_tokens, 100);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_inference_errors() {
        let error = inference_errors::generation_failed("test error");
        assert!(error.to_string().contains("Inference generation failed"));

        let error = inference_errors::token_limit_exceeded(1000, 2000);
        assert!(error
            .to_string()
            .contains("Token limit 1000 exceeded, requested 2000"));

        let error = inference_errors::template_processing_failed("invalid placeholder");
        assert!(error.to_string().contains("Template processing failed"));
    }

    #[test]
    fn test_health_status() {
        let healthy = HealthStatus::healthy();
        assert!(healthy.is_healthy());

        let unhealthy = HealthStatus::unhealthy("Service down");
        assert!(!unhealthy.is_healthy());
    }

    #[test]
    fn test_inference_response_from_string() {
        let token_usage = TokenUsage::new(10, 20);
        let response = InferenceResponse::from_string(
            "Generated text".to_string(),
            "gpt-4o".to_string(),
            token_usage,
            500,
        );

        assert_eq!(response.content, serde_json::Value::String("Generated text".to_string()));
        assert_eq!(response.metadata.model, "gpt-4o");
        assert_eq!(response.metadata.token_usage.total_tokens, 30);
        assert_eq!(response.metadata.processing_time_ms, 500);
    }

    #[test]
    fn test_inference_response_json_fallback() {
        let token_usage = TokenUsage::new(5, 15);
        
        // Valid JSON
        let json_response = InferenceResponse::from_text_with_json_fallback(
            "{\"message\": \"Hello\"}".to_string(),
            "gpt-4o".to_string(),
            token_usage.clone(),
            250,
        );
        
        match &json_response.content {
            serde_json::Value::Object(obj) => {
                assert_eq!(obj.get("message"), Some(&serde_json::Value::String("Hello".to_string())));
            },
            _ => panic!("Expected JSON object"),
        }
        
        // Invalid JSON (fallback to string)
        let text_response = InferenceResponse::from_text_with_json_fallback(
            "Not valid JSON".to_string(),
            "gpt-4o".to_string(),
            token_usage,
            250,
        );
        
        assert_eq!(text_response.content, serde_json::Value::String("Not valid JSON".to_string()));
    }

    #[test]
    fn test_response_metadata() {
        let token_usage = TokenUsage::new(25, 50);
        let metadata = ResponseMetadata::new(
            "claude-3-5-sonnet".to_string(),
            token_usage,
            750,
        );
        
        assert_eq!(metadata.model, "claude-3-5-sonnet");
        assert_eq!(metadata.token_usage.total_tokens, 75);
        assert_eq!(metadata.processing_time_ms, 750);
        assert!(metadata.metadata.is_empty());
    }
}