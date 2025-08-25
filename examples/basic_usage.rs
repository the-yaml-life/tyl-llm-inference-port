//! Basic usage example for TYL LLM Inference Port
//!
//! This example demonstrates how to use the simplified template-based inference
//! with the mock service for testing and understanding the API structure.

use std::collections::HashMap;
use tyl_llm_inference_port::{InferenceRequest, InferenceService, ModelType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 TYL LLM Inference Port - Template-Based Usage Example");
    println!("========================================================\n");

    // This example uses mock services since we don't have real API keys
    // In a real application, you would use actual adapter implementations
    #[cfg(feature = "mock")]
    {
        example_with_mock().await?;
    }

    #[cfg(not(feature = "mock"))]
    {
        println!("⚠️  Mock feature not enabled. Enable with: cargo run --example basic_usage --features mock");
        println!("\nThis example would typically show how to:");
        println!("1. Create an InferenceService adapter (OpenAI, Anthropic, etc.)");
        println!("2. Use templates with parameters for dynamic prompts");
        println!("3. Receive JSON responses with metadata");
        println!("4. Handle different model types for optimization");
        println!("5. Check service health and monitor performance");
    }

    Ok(())
}

#[cfg(feature = "mock")]
async fn example_with_mock() -> Result<(), Box<dyn std::error::Error>> {
    use tyl_llm_inference_port::MockInferenceService;

    // Example 1: Basic template-based inference
    println!("📝 Example 1: Basic Template-Based Inference");
    println!("--------------------------------------------");

    let inference_service = MockInferenceService::new().with_latency(50);

    let mut params = HashMap::new();
    params.insert("language".to_string(), "Rust".to_string());
    params.insert("task".to_string(), "print hello world".to_string());

    let request = InferenceRequest::new(
        "Write a {{language}} function that will {{task}}",
        params,
        ModelType::Coding,
    )
    .with_max_tokens(200)
    .with_temperature(0.3);

    println!("Template: {}", request.template);
    println!("Parameters: {:?}", request.parameters);
    println!("Rendered: {}", request.render_template());
    println!("Model Type: {:?}", request.model_type);

    let response = inference_service.infer(request).await?;

    println!("\nResponse JSON:");
    println!("{}", serde_json::to_string_pretty(&response.content)?);
    println!("\nMetadata:");
    println!("  Model: {}", response.metadata.model);
    println!(
        "  Tokens: {} prompt + {} completion = {} total",
        response.metadata.token_usage.prompt_tokens,
        response.metadata.token_usage.completion_tokens,
        response.metadata.token_usage.total_tokens
    );
    println!("  Processing time: {}ms\n", response.metadata.processing_time_ms);

    // Example 2: Creative writing with parameters
    println!("🎨 Example 2: Creative Writing Template");
    println!("--------------------------------------");

    let mut story_params = HashMap::new();
    story_params.insert("character".to_string(), "brave knight".to_string());
    story_params.insert("setting".to_string(), "enchanted forest".to_string());
    story_params.insert("quest".to_string(), "find the lost crystal".to_string());

    let story_request = InferenceRequest::new(
        "Write a short story about a {{character}} who must {{quest}} in an {{setting}}",
        story_params,
        ModelType::Creative,
    ).with_temperature(0.9);

    println!("Story Template: {}", story_request.template);
    println!("Story Parameters: {:?}", story_request.parameters);

    let story_response = inference_service.infer(story_request).await?;

    println!("\nStory Response:");
    println!("{}", serde_json::to_string_pretty(&story_response.content)?);

    // Example 3: Reasoning and analysis
    println!("\n🧠 Example 3: Reasoning and Analysis");
    println!("-----------------------------------");

    let mut analysis_params = HashMap::new();
    analysis_params.insert("problem".to_string(), "climate change".to_string());
    analysis_params.insert("context".to_string(), "developing countries".to_string());

    let reasoning_request = InferenceRequest::new(
        "Analyze the problem of {{problem}} specifically in the context of {{context}}. Provide a structured analysis with reasoning steps.",
        analysis_params,
        ModelType::Reasoning,
    );

    let reasoning_response = inference_service.infer(reasoning_request).await?;

    println!("Analysis Response:");
    println!("{}", serde_json::to_string_pretty(&reasoning_response.content)?);

    // Example 4: Fast responses for simple queries
    println!("\n⚡ Example 4: Fast Response Template");
    println!("----------------------------------");

    let mut quick_params = HashMap::new();
    quick_params.insert("topic".to_string(), "machine learning".to_string());

    let quick_request = InferenceRequest::new(
        "Give me a quick summary about {{topic}}",
        quick_params,
        ModelType::Fast,
    );

    let quick_response = inference_service.infer(quick_request).await?;

    println!("Quick Response:");
    println!("{}", serde_json::to_string_pretty(&quick_response.content)?);
    println!("Processing time: {}ms", quick_response.metadata.processing_time_ms);

    // Example 5: Complex template with multiple parameters
    println!("\n🔧 Example 5: Complex Multi-Parameter Template");
    println!("----------------------------------------------");

    let mut complex_params = HashMap::new();
    complex_params.insert("user_name".to_string(), "Alice".to_string());
    complex_params.insert("user_role".to_string(), "senior developer".to_string());
    complex_params.insert("project".to_string(), "e-commerce platform".to_string());
    complex_params.insert("technology".to_string(), "microservices architecture".to_string());
    complex_params.insert("timeline".to_string(), "3 months".to_string());

    let complex_request = InferenceRequest::new(
        "Hello {{user_name}}! As a {{user_role}} working on a {{project}}, please provide recommendations for implementing {{technology}} within a {{timeline}} timeline. Include pros, cons, and implementation steps.",
        complex_params,
        ModelType::General,
    ).with_metadata("request_id".to_string(), "example_5".to_string());

    println!("Complex Template: {}", complex_request.template);
    println!("Rendered: {}", complex_request.render_template());

    let complex_response = inference_service.infer(complex_request).await?;

    println!("\nComplex Response:");
    println!("{}", serde_json::to_string_pretty(&complex_response.content)?);

    // Example 6: Custom JSON response
    println!("\n📋 Example 6: Custom JSON Response");
    println!("---------------------------------");

    let custom_json = r#"{"status": "success", "data": {"message": "Custom response", "score": 95}, "metadata": {"version": "1.0"}}"#;
    let custom_service = MockInferenceService::new().with_custom_response(custom_json);

    let params = HashMap::new();
    let custom_request = InferenceRequest::new("Any template", params, ModelType::General);

    let custom_response = custom_service.infer(custom_request).await?;

    println!("Custom JSON Response:");
    println!("{}", serde_json::to_string_pretty(&custom_response.content)?);

    // Example 7: Health checks and service info
    println!("\n🏥 Example 7: Service Health and Info");
    println!("-----------------------------------");

    let health_result = inference_service.health_check().await?;
    println!("Service Health: {:?}", health_result.status);
    println!("Health Check Time: {}", health_result.timestamp);
    if let Some(metadata) = health_result.metadata.get("latency_ms") {
        println!("Configured latency: {}ms", metadata);
    }

    println!("\nSupported models:");
    for model in inference_service.supported_models() {
        println!("  - {}", model);
    }

    // Example 8: Token counting
    println!("\n🔢 Example 8: Token Counting");
    println!("---------------------------");

    let sample_template = "Generate a {{type}} about {{topic}} with {{details}}";
    let token_count = inference_service.count_tokens(sample_template)?;
    println!("Template: \"{}\"", sample_template);
    println!("Estimated tokens: {}", token_count);

    // Example 9: Model type optimization
    println!("\n🎯 Example 9: Model Type Optimization");
    println!("------------------------------------");

    let model_types = [
        ModelType::Coding,
        ModelType::Fast,
        ModelType::Creative,
        ModelType::Reasoning,
        ModelType::General,
    ];

    for model_type in model_types {
        println!("{:?}:", model_type);
        println!("  OpenAI: {}", model_type.optimal_openai_model());
        println!("  Anthropic: {}", model_type.optimal_anthropic_model());
        println!("  Max tokens: {}", model_type.typical_max_tokens());
    }

    println!("\n✅ All examples completed successfully!");
    println!("\n📖 Key Features Demonstrated:");
    println!("• Template-based prompts with {{parameter}} substitution");
    println!("• JSON responses with automatic parsing/fallback");
    println!("• Different model types for optimization");
    println!("• Rich metadata including tokens and timing");
    println!("• Health monitoring and service info");
    println!("• Custom responses for testing");

    println!("\n🚀 Next steps:");
    println!("1. Implement real adapters for OpenAI, Anthropic, or local models");
    println!("2. Add more sophisticated template engines (Handlebars, Tera, etc.)");
    println!("3. Implement response caching and rate limiting");
    println!("4. Add streaming support for real-time responses");
    println!("5. Create validation for template parameters");

    Ok(())
}