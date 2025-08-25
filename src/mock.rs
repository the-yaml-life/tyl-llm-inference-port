//! Mock implementations for testing and demonstration

use crate::*;
use async_trait::async_trait;
use std::time::Instant;

/// Mock inference service for testing
#[derive(Debug, Clone)]
pub struct MockInferenceService {
    /// Simulated latency in milliseconds
    pub simulated_latency_ms: u64,
    /// Whether health checks should fail
    pub health_check_fails: bool,
    /// Custom response for testing (JSON string or plain text)
    pub custom_response: Option<String>,
}

impl MockInferenceService {
    pub fn new() -> Self {
        Self {
            simulated_latency_ms: 100,
            health_check_fails: false,
            custom_response: None,
        }
    }

    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.simulated_latency_ms = latency_ms;
        self
    }

    pub fn with_health_failure(mut self) -> Self {
        self.health_check_fails = true;
        self
    }

    pub fn with_custom_response(mut self, response: impl Into<String>) -> Self {
        self.custom_response = Some(response.into());
        self
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        // Simple approximation: ~4 characters per token
        (text.len() + 3) / 4
    }

    fn generate_mock_response(&self, request: &InferenceRequest) -> String {
        if let Some(ref custom) = self.custom_response {
            return custom.clone();
        }

        let rendered_template = request.render_template();

        match request.model_type {
            ModelType::Coding => {
                // Generate JSON response for code generation
                format!(
                    r#"{{
    "code": "// Generated code for: {}\nfn main() {{\n    println!(\"Hello from mock!\");\n}}",
    "language": "rust",
    "explanation": "This is a basic Rust program generated from the template."
}}"#,
                    rendered_template.replace('"', r#"\""#)
                )
            }
            ModelType::Fast => {
                format!(
                    r#"{{"response": "Quick response: {}"}}"#,
                    rendered_template.replace('"', r#"\""#)
                )
            }
            ModelType::Creative => {
                format!(
                    r#"{{
    "story": "Once upon a time, when prompted with '{}', there was a magical response...",
    "genre": "fantasy",
    "mood": "whimsical"
}}"#,
                    rendered_template.replace('"', r#"\""#)
                )
            }
            ModelType::Reasoning => {
                format!(
                    r#"{{
    "analysis": "After careful analysis of '{}'...",
    "reasoning_steps": [
        "First, I analyzed the template and parameters",
        "Then, I considered the context and implications", 
        "Finally, I formulated this structured response"
    ],
    "conclusion": "This is a mock reasoning response with detailed analysis."
}}"#,
                    rendered_template.replace('"', r#"\""#)
                )
            }
            ModelType::General => {
                format!(
                    r#"{{"message": "Mock completion for: {}"}}"#,
                    rendered_template.replace('"', r#"\""#)
                )
            }
        }
    }
}

impl Default for MockInferenceService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InferenceService for MockInferenceService {
    async fn infer(&self, request: InferenceRequest) -> InferenceResult<InferenceResponse> {
        let start = Instant::now();

        // Simulate processing time
        if self.simulated_latency_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.simulated_latency_ms)).await;
        }

        let generated_content = self.generate_mock_response(&request);
        let rendered_template = request.render_template();
        let prompt_tokens = self.estimate_tokens(&rendered_template);
        let completion_tokens = self.estimate_tokens(&generated_content);

        let model = request
            .model_override
            .unwrap_or_else(|| request.model_type.optimal_openai_model().to_string());

        // Try to parse as JSON, fallback to string if it fails
        let response = InferenceResponse::from_text_with_json_fallback(
            generated_content,
            model,
            TokenUsage::new(prompt_tokens as u32, completion_tokens as u32),
            start.elapsed().as_millis() as u64,
        );

        Ok(response)
    }

    async fn health_check(&self) -> InferenceResult<HealthCheckResult> {
        if self.health_check_fails {
            Ok(HealthCheckResult::new(HealthStatus::unhealthy(
                "Mock service intentionally failing",
            )))
        } else {
            Ok(HealthCheckResult::new(HealthStatus::healthy())
                .with_metadata("service", serde_json::Value::String("mock".to_string()))
                .with_metadata(
                    "latency_ms",
                    serde_json::Value::Number(serde_json::Number::from(self.simulated_latency_ms)),
                ))
        }
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "mock-general".to_string(),
            "mock-coding".to_string(),
            "mock-fast".to_string(),
            "mock-creative".to_string(),
            "mock-reasoning".to_string(),
        ]
    }

    fn count_tokens(&self, text: &str) -> InferenceResult<usize> {
        Ok(self.estimate_tokens(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_mock_inference_service_basic() {
        let service = MockInferenceService::new().with_latency(10);

        let mut params = HashMap::new();
        params.insert("name".to_string(), "Juan".to_string());

        let request = InferenceRequest::new("Hello {{name}}!", params, ModelType::General);

        let response = service.infer(request).await.unwrap();

        // Should be JSON response
        match &response.content {
            serde_json::Value::Object(obj) => {
                assert!(obj.get("message").is_some());
            }
            _ => panic!("Expected JSON object response"),
        }

        assert!(response.metadata.processing_time_ms >= 10);
        assert_eq!(response.metadata.model, "gpt-4o-mini");
    }

    #[tokio::test]
    async fn test_mock_inference_service_coding() {
        let service = MockInferenceService::new().with_latency(5);

        let mut params = HashMap::new();
        params.insert("task".to_string(), "print hello world".to_string());

        let request = InferenceRequest::new("Generate code to {{task}}", params, ModelType::Coding);

        let response = service.infer(request).await.unwrap();

        match &response.content {
            serde_json::Value::Object(obj) => {
                assert!(obj.get("code").is_some());
                assert!(obj.get("language").is_some());
                assert!(obj.get("explanation").is_some());
            }
            _ => panic!("Expected JSON object response with code structure"),
        }
    }

    #[tokio::test]
    async fn test_mock_inference_service_reasoning() {
        let service = MockInferenceService::new();

        let mut params = HashMap::new();
        params.insert("problem".to_string(), "solve this puzzle".to_string());

        let request = InferenceRequest::new("Please {{problem}}", params, ModelType::Reasoning);

        let response = service.infer(request).await.unwrap();

        match &response.content {
            serde_json::Value::Object(obj) => {
                assert!(obj.get("analysis").is_some());
                assert!(obj.get("reasoning_steps").is_some());
                assert!(obj.get("conclusion").is_some());
            }
            _ => panic!("Expected JSON object response with reasoning structure"),
        }
    }

    #[tokio::test]
    async fn test_mock_service_custom_response() {
        let custom_json = r#"{"custom": "test response", "number": 42}"#;
        let service = MockInferenceService::new().with_custom_response(custom_json);

        let params = HashMap::new();
        let request = InferenceRequest::new("Any template", params, ModelType::General);

        let response = service.infer(request).await.unwrap();

        match &response.content {
            serde_json::Value::Object(obj) => {
                assert_eq!(
                    obj.get("custom"),
                    Some(&serde_json::Value::String("test response".to_string()))
                );
                assert_eq!(
                    obj.get("number"),
                    Some(&serde_json::Value::Number(serde_json::Number::from(42)))
                );
            }
            _ => panic!("Expected JSON object response"),
        }
    }

    #[tokio::test]
    async fn test_mock_service_template_rendering() {
        let service = MockInferenceService::new();

        let mut params = HashMap::new();
        params.insert("user".to_string(), "Alice".to_string());
        params.insert("action".to_string(), "coding".to_string());

        let request =
            InferenceRequest::new("User {{user}} is {{action}}", params, ModelType::General);

        let response = service.infer(request).await.unwrap();

        // The rendered template should be included in the response
        let response_str = response.content.to_string();
        assert!(response_str.contains("User Alice is coding"));
    }

    #[tokio::test]
    async fn test_mock_service_health_checks() {
        let healthy_service = MockInferenceService::new();
        let unhealthy_service = MockInferenceService::new().with_health_failure();

        let healthy_result = healthy_service.health_check().await.unwrap();
        assert!(healthy_result.status.is_healthy());

        let unhealthy_result = unhealthy_service.health_check().await.unwrap();
        assert!(!unhealthy_result.status.is_healthy());
    }

    #[test]
    fn test_mock_service_token_counting() {
        let service = MockInferenceService::new();
        let tokens = service.count_tokens("Hello world").unwrap();
        assert!(tokens > 0);
        assert!(tokens <= 3); // Approximately 2-3 tokens for "Hello world"
    }

    #[test]
    fn test_mock_service_supported_models() {
        let service = MockInferenceService::new();
        let models = service.supported_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"mock-general".to_string()));
        assert!(models.contains(&"mock-coding".to_string()));
    }

    #[tokio::test]
    async fn test_mock_service_model_override() {
        let service = MockInferenceService::new();

        let params = HashMap::new();
        let request =
            InferenceRequest::new("Test", params, ModelType::General).with_model("custom-model");

        let response = service.infer(request).await.unwrap();
        assert_eq!(response.metadata.model, "custom-model");
    }

    #[tokio::test]
    async fn test_mock_service_fallback_to_string() {
        // Test with invalid JSON that should fallback to string
        let invalid_json = "This is not valid JSON { incomplete";
        let service = MockInferenceService::new().with_custom_response(invalid_json);

        let params = HashMap::new();
        let request = InferenceRequest::new("Test", params, ModelType::General);

        let response = service.infer(request).await.unwrap();

        match &response.content {
            serde_json::Value::String(s) => {
                assert_eq!(s, invalid_json);
            }
            _ => panic!("Expected string fallback for invalid JSON"),
        }
    }
}
