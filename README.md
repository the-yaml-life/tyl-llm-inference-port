# TYL LLM Inference Port

🤖 **Template-based LLM inference port for TYL framework**

This module provides a simplified, template-based interface for LLM inference services following hexagonal architecture patterns.

## 🚀 Quick Start

### Basic Usage

```rust
use tyl_llm_inference_port::{InferenceService, InferenceRequest, ModelType};
use std::collections::HashMap;

// Create service (use real adapter in production)
let service = MockInferenceService::new();

// Create request with template and parameters
let mut params = HashMap::new();
params.insert("language".to_string(), "Rust".to_string());
params.insert("task".to_string(), "print hello world".to_string());

let request = InferenceRequest::new(
    "Write a {{language}} function that will {{task}}", 
    params, 
    ModelType::Coding
);

// Get JSON response with metadata
let response = service.infer(request).await?;
println!("Response: {}", serde_json::to_string_pretty(&response.content)?);
```

### Template System

Simple `{{parameter}}` placeholder replacement:

```rust
let mut params = HashMap::new();
params.insert("user".to_string(), "Alice".to_string());
params.insert("task".to_string(), "code review".to_string());
params.insert("urgency".to_string(), "high".to_string());

let request = InferenceRequest::new(
    "Hello {{user}}, please help with {{task}}. Priority: {{urgency}}", 
    params, 
    ModelType::General
);

// Renders to: "Hello Alice, please help with code review. Priority: high"
```

## 🏗️ Architecture

### Hexagonal Design

```rust
// Port (Interface)
#[async_trait]
trait InferenceService {
    async fn infer(&self, request: InferenceRequest) -> InferenceResult<InferenceResponse>;
    async fn health_check(&self) -> InferenceResult<HealthCheckResult>;
    fn supported_models(&self) -> Vec<String>;
    fn count_tokens(&self, text: &str) -> InferenceResult<usize>;
}

// Adapters (Implementations)
struct OpenAIAdapter { /* ... */ }
struct AnthropicAdapter { /* ... */ }
struct LocalModelAdapter { /* ... */ }
```

### Core Types

- **`InferenceRequest`** - Template with parameters for dynamic prompt generation
- **`InferenceResponse`** - JSON response with rich metadata
- **`ModelType`** - Optimization enum (Coding, Reasoning, General, Fast, Creative)
- **`ResponseMetadata`** - Token usage, model info, timing, and custom metadata

## 🎯 Model Types

Different model types optimize for different use cases:

| Type | OpenAI Model | Anthropic Model | Use Case |
|------|-------------|-----------------|----------|
| **Coding** | `gpt-4o` | `claude-3-5-sonnet-20241022` | Code generation, programming |
| **Reasoning** | `gpt-4o` | `claude-3-5-sonnet-20241022` | Complex analysis, logic |
| **General** | `gpt-4o-mini` | `claude-3-5-haiku-20241022` | Balanced performance |
| **Fast** | `gpt-3.5-turbo` | `claude-3-5-haiku-20241022` | Quick responses |
| **Creative** | `gpt-4o` | `claude-3-5-sonnet-20241022` | Creative writing, content |

## 📊 JSON Response Structures

The mock service generates different JSON structures based on model type:

### Coding Response
```json
{
  "code": "// Generated code...",
  "language": "rust", 
  "explanation": "This is a basic program..."
}
```

### Reasoning Response
```json
{
  "analysis": "After careful analysis...",
  "reasoning_steps": [
    "First, I analyzed...",
    "Then, I considered...",
    "Finally, I formulated..."
  ],
  "conclusion": "This is the final conclusion..."
}
```

### Creative Response
```json
{
  "story": "Once upon a time...",
  "genre": "fantasy",
  "mood": "whimsical"
}
```

## 🧪 Testing

```bash
# Run all tests
cargo test --all-features

# Run with mock feature
cargo test --features mock

# Run example
cargo run --example basic_usage --features mock

# Documentation tests
cargo test --doc
```

## 📦 Features

- **`mock`** - Enable mock implementations for testing (default: enabled)

## 🛠️ Development Commands

```bash
# Code quality
cargo clippy
cargo fmt
cargo audit

# Documentation  
cargo doc --no-deps --open

# Testing
cargo test --verbose
cargo test --all-features
```

## 📝 Examples

See [`examples/basic_usage.rs`](examples/basic_usage.rs) for comprehensive examples covering:

- Template-based prompts with parameter substitution
- JSON response parsing and handling  
- Different model types and optimization
- Health monitoring and service info
- Token counting and usage tracking
- Error handling patterns
- Custom responses for testing
- Concurrent request processing

## 🔧 Implementation Guide

### Creating Custom Adapters

```rust
use tyl_llm_inference_port::{InferenceService, InferenceRequest, InferenceResponse};

struct MyLLMAdapter {
    api_key: String,
    base_url: String,
}

#[async_trait]
impl InferenceService for MyLLMAdapter {
    async fn infer(&self, request: InferenceRequest) -> InferenceResult<InferenceResponse> {
        // 1. Render template with parameters
        let prompt = request.render_template();
        
        // 2. Call your LLM API
        let api_response = self.call_llm_api(&prompt, &request).await?;
        
        // 3. Parse response as JSON or fallback to string
        let response = InferenceResponse::from_text_with_json_fallback(
            api_response.content,
            api_response.model,
            TokenUsage::new(api_response.prompt_tokens, api_response.completion_tokens),
            api_response.processing_time_ms,
        );
        
        Ok(response)
    }
    
    // Implement other required methods...
}
```

## 📋 TODO List

### 🔄 **Current Development**
- [ ] **Add comprehensive logging to database for each Template + Parameters → JSON Response**
  - [ ] Design logging schema for inference requests/responses
  - [ ] Implement database adapter for logging (PostgreSQL/SQLite)
  - [ ] Log template, parameters, rendered prompt, response, metadata
  - [ ] Add configuration for logging levels and targets
  - [ ] Include performance metrics and error tracking
  - [ ] Add privacy controls for sensitive parameter filtering
  - [ ] Create analytics queries for common usage patterns

### 🚀 **Future Enhancements**
- [ ] **Advanced Template Engines**
  - [ ] Handlebars integration for complex templating
  - [ ] Tera template engine support
  - [ ] Template validation and syntax checking
  - [ ] Template library/registry system

- [ ] **Streaming Support**
  - [ ] Real-time streaming response interface
  - [ ] Server-sent events for web integration
  - [ ] Backpressure handling for large responses

- [ ] **Performance & Reliability**
  - [ ] Built-in response caching with TTL
  - [ ] Rate limiting with backoff strategies
  - [ ] Circuit breaker pattern for failing services
  - [ ] Request deduplication

- [ ] **Provider Integrations**
  - [ ] OpenAI adapter implementation
  - [ ] Anthropic Claude adapter implementation
  - [ ] Local model adapters (Ollama, llama.cpp)
  - [ ] Azure OpenAI integration
  - [ ] Google PaLM/Gemini adapter

- [ ] **Advanced Features**
  - [ ] Function calling support
  - [ ] Batch processing capabilities
  - [ ] Multi-modal input support (text + images)
  - [ ] Response format validation
  - [ ] Custom model fine-tuning integration

- [ ] **Monitoring & Observability**
  - [ ] Detailed metrics collection
  - [ ] Distributed tracing integration
  - [ ] Cost tracking per request
  - [ ] Usage analytics dashboard
  - [ ] Performance benchmarking tools

- [ ] **Security & Compliance**
  - [ ] API key rotation and management
  - [ ] Content filtering and moderation
  - [ ] PII detection and redaction
  - [ ] Audit logging for compliance
  - [ ] Rate limiting per user/tenant

### 🐛 **Known Limitations**
- [ ] Mock implementations use simple token estimation (4 chars ≈ 1 token)
- [ ] Template system is basic string replacement (no conditionals/loops)
- [ ] No streaming support yet
- [ ] Token counting is approximate for non-OpenAI models

## 🔗 Dependencies

### Runtime
- [`tyl-errors`](https://github.com/the-yaml-life/tyl-errors) - TYL unified error handling
- `async-trait` - Async trait support
- `serde` / `serde_json` - JSON serialization
- `chrono` - Date/time handling

### Development  
- `tokio` - Async runtime for testing
- `futures` - Future combinators for concurrent testing

## 🔗 Related TYL Modules

- [`tyl-errors`](https://github.com/the-yaml-life/tyl-errors) - Unified error handling
- [`tyl-config`](https://github.com/the-yaml-life/tyl-config) - Configuration management  
- [`tyl-logging`](https://github.com/the-yaml-life/tyl-logging) - Structured logging
- [`tyl-tracing`](https://github.com/the-yaml-life/tyl-tracing) - Distributed tracing
- [`tyl-embeddings-port`](https://github.com/the-yaml-life/tyl-embeddings-port) - Embedding generation

## 📄 License

AGPL-3.0 - See [LICENSE](LICENSE) for details.

---

**Built with the TYL Framework** - Hexagonal architecture, comprehensive testing, and production-ready design.