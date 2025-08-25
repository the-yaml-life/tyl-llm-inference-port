//! Integration tests for TYL LLM Inference Port
//!
//! These tests verify that the module integrates correctly with the TYL framework
//! and follows the established patterns for the simplified template-based interface.

use std::collections::HashMap;
use tyl_llm_inference_port::{
    inference_errors, HealthStatus, InferenceRequest, InferenceService, ModelType, TylError,
    TylResult,
};

#[tokio::test]
async fn test_tyl_error_integration() {
    // Test that our error helpers create proper TylError instances
    let error = inference_errors::generation_failed("test failure");
    assert!(error.to_string().contains("Inference generation failed"));

    let error = inference_errors::token_limit_exceeded(1000, 2000);
    assert!(error.to_string().contains("Token limit 1000 exceeded"));

    let error = inference_errors::invalid_api_key("OpenAI");
    assert!(error.to_string().contains("Invalid API key for OpenAI"));

    let error = inference_errors::template_processing_failed("invalid placeholder");
    assert!(error.to_string().contains("Template processing failed"));
}

#[cfg(feature = "mock")]
#[tokio::test]
async fn test_inference_service_integration() {
    use tyl_llm_inference_port::MockInferenceService;

    let service = MockInferenceService::new().with_latency(10);

    // Test basic template-based inference
    let mut params = HashMap::new();
    params.insert("language".to_string(), "Rust".to_string());

    let request =
        InferenceRequest::new("Write a {{language}} function", params, ModelType::General);
    let response = service.infer(request).await;
    assert!(response.is_ok());

    let response = response.unwrap();
    assert!(!response.content.is_null());
    assert!(response.metadata.token_usage.total_tokens > 0);
    assert!(response.metadata.processing_time_ms >= 10);

    // Test health check
    let health = service.health_check().await.unwrap();
    assert_eq!(health.status, HealthStatus::Healthy);

    // Test token counting
    let token_count = service.count_tokens("test text").unwrap();
    assert!(token_count > 0);

    // Test supported models
    let models = service.supported_models();
    assert!(!models.is_empty());
}

#[test]
fn test_model_type_optimization() {
    // Test that model types provide correct optimization settings
    assert_eq!(ModelType::Coding.optimal_openai_model(), "gpt-4o");
    assert_eq!(ModelType::Fast.optimal_openai_model(), "gpt-3.5-turbo");
    assert_eq!(
        ModelType::Reasoning.optimal_anthropic_model(),
        "claude-3-5-sonnet-20241022"
    );
    assert_eq!(
        ModelType::General.optimal_anthropic_model(),
        "claude-3-5-haiku-20241022"
    );

    // Test max tokens are reasonable
    assert!(ModelType::Coding.typical_max_tokens() >= 1024);
    assert!(ModelType::Reasoning.typical_max_tokens() >= 4096);
    assert!(ModelType::Fast.typical_max_tokens() <= 2048);
}

#[test]
fn test_template_request_builder_patterns() {
    // Test inference request builder
    let mut params = HashMap::new();
    params.insert("name".to_string(), "Alice".to_string());
    params.insert("task".to_string(), "code review".to_string());

    let request = InferenceRequest::new(
        "Hello {{name}}, please help with {{task}}",
        params,
        ModelType::Coding,
    )
    .with_model("custom-model")
    .with_max_tokens(500)
    .with_temperature(0.8)
    .with_metadata("context", "test");

    assert_eq!(
        request.template,
        "Hello {{name}}, please help with {{task}}"
    );
    assert_eq!(request.model_type, ModelType::Coding);
    assert_eq!(request.model_override, Some("custom-model".to_string()));
    assert_eq!(request.max_tokens, Some(500));
    assert_eq!(request.temperature, Some(0.8));
    assert_eq!(request.metadata.get("context"), Some(&"test".to_string()));

    // Test template rendering
    let rendered = request.render_template();
    assert_eq!(rendered, "Hello Alice, please help with code review");
}

#[test]
fn test_template_rendering() {
    // Test simple template rendering
    let mut params = HashMap::new();
    params.insert("greeting".to_string(), "Hello".to_string());
    params.insert("name".to_string(), "World".to_string());

    let request = InferenceRequest::new("{{greeting}} {{name}}!", params, ModelType::General);
    assert_eq!(request.render_template(), "Hello World!");

    // Test template with no parameters
    let empty_params = HashMap::new();
    let request = InferenceRequest::new("No parameters here", empty_params, ModelType::General);
    assert_eq!(request.render_template(), "No parameters here");

    // Test template with unused parameters
    let mut extra_params = HashMap::new();
    extra_params.insert("used".to_string(), "yes".to_string());
    extra_params.insert("unused".to_string(), "no".to_string());

    let request =
        InferenceRequest::new("Only {{used}} parameter", extra_params, ModelType::General);
    assert_eq!(request.render_template(), "Only yes parameter");
}

#[test]
fn test_serialization_compatibility() {
    // Test that all public types can be serialized/deserialized
    let mut params = HashMap::new();
    params.insert("test".to_string(), "value".to_string());

    let request = InferenceRequest::new("Template {{test}}", params, ModelType::General);
    let json = serde_json::to_string(&request).unwrap();
    let deserialized: InferenceRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(request.template, deserialized.template);

    let token_usage = tyl_llm_inference_port::TokenUsage::new(10, 20);
    let json = serde_json::to_string(&token_usage).unwrap();
    let deserialized: tyl_llm_inference_port::TokenUsage = serde_json::from_str(&json).unwrap();
    assert_eq!(token_usage.total_tokens, deserialized.total_tokens);
}

#[cfg(feature = "mock")]
#[tokio::test]
async fn test_service_error_handling() {
    use tyl_llm_inference_port::MockInferenceService;

    // Test unhealthy service
    let unhealthy_service = MockInferenceService::new().with_health_failure();
    let health_result = unhealthy_service.health_check().await.unwrap();
    assert!(!health_result.status.is_healthy());
}

#[cfg(feature = "mock")]
#[tokio::test]
async fn test_trait_object_usage() {
    use tyl_llm_inference_port::MockInferenceService;

    // Test using services as trait objects
    let inference_service: Box<dyn InferenceService> =
        Box::new(MockInferenceService::new().with_latency(1));

    let params = HashMap::new();
    let request = InferenceRequest::new("Test template", params, ModelType::Fast);
    let response = inference_service.infer(request).await;
    assert!(response.is_ok());
}

#[test]
fn test_temperature_clamping() {
    // Test that temperature values are properly clamped
    let params = HashMap::new();

    let request =
        InferenceRequest::new("Test", params.clone(), ModelType::General).with_temperature(2.0); // Should be clamped to 1.0
    assert_eq!(request.temperature, Some(1.0));

    let request =
        InferenceRequest::new("Test", params.clone(), ModelType::General).with_temperature(-0.5); // Should be clamped to 0.0
    assert_eq!(request.temperature, Some(0.0));

    let request = InferenceRequest::new("Test", params, ModelType::General).with_temperature(0.7); // Should remain as-is
    assert_eq!(request.temperature, Some(0.7));
}

#[test]
fn test_result_type_compatibility() {
    // Test that our result types are compatible with TYL framework
    fn test_function() -> TylResult<String> {
        Ok("success".to_string())
    }

    fn test_inference_result() -> tyl_llm_inference_port::InferenceResult<String> {
        Ok("success".to_string())
    }

    // Both should work interchangeably
    let result1 = test_function();
    let result2 = test_inference_result();

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert_eq!(result1.unwrap(), result2.unwrap());
}

#[cfg(feature = "mock")]
#[tokio::test]
async fn test_concurrent_requests() {
    use tokio::time::Instant;
    use tyl_llm_inference_port::MockInferenceService;

    let service = std::sync::Arc::new(MockInferenceService::new().with_latency(50));

    let start = Instant::now();

    // Send multiple concurrent requests
    let tasks: Vec<_> = (0..5)
        .map(|i| {
            let service = service.clone();
            tokio::spawn(async move {
                let mut params = HashMap::new();
                params.insert("id".to_string(), i.to_string());

                let request = InferenceRequest::new("Request {{id}}", params, ModelType::Fast);
                service.infer(request).await
            })
        })
        .collect();

    // Wait for all tasks to complete
    let results: Vec<_> = futures::future::join_all(tasks).await;

    let elapsed = start.elapsed();

    // All requests should succeed
    for result in results {
        assert!(result.unwrap().is_ok());
    }

    // Should complete in roughly the latency time (not 5x the latency)
    // allowing some margin for test environment variability
    assert!(elapsed.as_millis() < 200); // Should be around 50ms, not 250ms
}

#[cfg(feature = "mock")]
#[tokio::test]
async fn test_json_response_handling() {
    use tyl_llm_inference_port::MockInferenceService;

    let service = MockInferenceService::new();

    // Test different model types produce different JSON structures
    let params = HashMap::new();

    // Coding model should produce structured code response
    let request = InferenceRequest::new("Generate code", params.clone(), ModelType::Coding);
    let response = service.infer(request).await.unwrap();

    if let serde_json::Value::Object(obj) = &response.content {
        assert!(obj.get("code").is_some());
        assert!(obj.get("language").is_some());
    } else {
        panic!("Expected JSON object for coding response");
    }

    // Reasoning model should produce structured reasoning response
    let request = InferenceRequest::new("Analyze problem", params.clone(), ModelType::Reasoning);
    let response = service.infer(request).await.unwrap();

    if let serde_json::Value::Object(obj) = &response.content {
        assert!(obj.get("analysis").is_some());
        assert!(obj.get("reasoning_steps").is_some());
    } else {
        panic!("Expected JSON object for reasoning response");
    }
}

#[cfg(feature = "mock")]
#[tokio::test]
async fn test_custom_json_parsing() {
    use tyl_llm_inference_port::MockInferenceService;

    // Test with valid JSON
    let valid_json = r#"{"message": "Hello", "status": "success", "data": {"count": 42}}"#;
    let service = MockInferenceService::new().with_custom_response(valid_json);

    let params = HashMap::new();
    let request = InferenceRequest::new("Test", params, ModelType::General);
    let response = service.infer(request).await.unwrap();

    if let serde_json::Value::Object(obj) = &response.content {
        assert_eq!(
            obj.get("message"),
            Some(&serde_json::Value::String("Hello".to_string()))
        );
        assert_eq!(
            obj.get("status"),
            Some(&serde_json::Value::String("success".to_string()))
        );
        if let Some(serde_json::Value::Object(data)) = obj.get("data") {
            assert_eq!(
                data.get("count"),
                Some(&serde_json::Value::Number(serde_json::Number::from(42)))
            );
        }
    } else {
        panic!("Expected JSON object");
    }

    // Test with invalid JSON (should fallback to string)
    let invalid_json = "Not valid JSON at all";
    let service = MockInferenceService::new().with_custom_response(invalid_json);

    let params = HashMap::new();
    let request = InferenceRequest::new("Test", params, ModelType::General);
    let response = service.infer(request).await.unwrap();

    if let serde_json::Value::String(s) = &response.content {
        assert_eq!(s, "Not valid JSON at all");
    } else {
        panic!("Expected string fallback for invalid JSON");
    }
}

#[cfg(feature = "mock")]
#[tokio::test]
async fn test_complex_template_parameters() {
    use tyl_llm_inference_port::MockInferenceService;

    let service = MockInferenceService::new();

    let mut params = HashMap::new();
    params.insert("user_name".to_string(), "Bob".to_string());
    params.insert("user_age".to_string(), "25".to_string());
    params.insert("user_job".to_string(), "software engineer".to_string());
    params.insert("project_name".to_string(), "AI assistant".to_string());
    params.insert("technology".to_string(), "Rust".to_string());

    let complex_template = "Hi {{user_name}}! As a {{user_age}}-year-old {{user_job}}, I need help with {{project_name}} built in {{technology}}.";

    let request = InferenceRequest::new(complex_template, params, ModelType::General);

    let expected_rendering =
        "Hi Bob! As a 25-year-old software engineer, I need help with AI assistant built in Rust.";
    assert_eq!(request.render_template(), expected_rendering);

    let response = service.infer(request).await.unwrap();
    assert!(!response.content.is_null());

    // The rendered template should be reflected in the response somehow
    let response_str = response.content.to_string();
    assert!(response_str.contains("Bob"));
    assert!(response_str.contains("software engineer"));
}

#[test]
fn test_response_metadata_structure() {
    use tyl_llm_inference_port::{ResponseMetadata, TokenUsage};

    let token_usage = TokenUsage::new(100, 200);
    let metadata = ResponseMetadata::new("test-model".to_string(), token_usage, 1000)
        .with_metadata("custom_field", "custom_value");

    assert_eq!(metadata.model, "test-model");
    assert_eq!(metadata.token_usage.prompt_tokens, 100);
    assert_eq!(metadata.token_usage.completion_tokens, 200);
    assert_eq!(metadata.token_usage.total_tokens, 300);
    assert_eq!(metadata.processing_time_ms, 1000);
    assert_eq!(
        metadata.metadata.get("custom_field"),
        Some(&"custom_value".to_string())
    );
}
