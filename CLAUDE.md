# CLAUDE.md - tyl-llm-inference-port

## 📋 **Module Context**

**tyl-llm-inference-port** is the simplified template-based LLM inference module for the TYL framework. It provides a single, unified interface for text generation services following hexagonal architecture patterns.

## 🏗️ **Architecture**

### **Port (Interface)**
```rust
#[async_trait]
trait InferenceService {
    async fn infer(&self, request: InferenceRequest) -> InferenceResult<InferenceResponse>;
    async fn health_check(&self) -> InferenceResult<HealthCheckResult>;
    fn supported_models(&self) -> Vec<String>;
    fn count_tokens(&self, text: &str) -> InferenceResult<usize>;
}
```

### **Adapters (Implementations)**
- `MockInferenceService` - Mock implementation for testing and demonstration
- Future adapters: OpenAI adapter, Anthropic adapter, local model adapters

### **Core Types**
- `InferenceRequest` - Template with parameters for dynamic prompt generation
- `InferenceResponse` - JSON response with metadata
- `ModelType` - Optimization enum (Coding, Reasoning, General, Fast, Creative)
- `ResponseMetadata` - Token usage, model info, timing, and custom metadata
- `HealthStatus` / `HealthCheckResult` - Service health monitoring

## 🧪 **Testing**

```bash
cargo test -p tyl-llm-inference-port
cargo test --doc -p tyl-llm-inference-port
cargo run --example basic_usage -p tyl-llm-inference-port --features mock
cargo test --all-features -p tyl-llm-inference-port
```

## 📂 **File Structure**

```
tyl-llm-inference-port/
├── src/
│   ├── lib.rs                 # Core implementation with traits and types
│   └── mock.rs                # Mock implementations for testing
├── examples/
│   └── basic_usage.rs         # Comprehensive usage examples
├── tests/
│   └── integration_tests.rs   # Integration tests with TYL framework
├── README.md                  # Main documentation
├── CLAUDE.md                  # This file
└── Cargo.toml                 # Package metadata with features
```

## 🔧 **How to Use**

### **Basic Template-Based Inference**
```rust
use tyl_llm_inference_port::{InferenceService, InferenceRequest, ModelType};
use std::collections::HashMap;

// Using mock for example (replace with real adapter)
#[cfg(feature = "mock")]
use tyl_llm_inference_port::MockInferenceService;

let service = MockInferenceService::new();

let mut params = HashMap::new();
params.insert("language".to_string(), "Rust".to_string());
params.insert("task".to_string(), "print hello world".to_string());

let request = InferenceRequest::new(
    "Write a {{language}} function that will {{task}}", 
    params, 
    ModelType::Coding
)
.with_max_tokens(200)
.with_temperature(0.3);

let response = service.infer(request).await?;
println!("Response JSON: {}", serde_json::to_string_pretty(&response.content)?);
```

### **JSON Response Handling**
```rust
// The response content is always a serde_json::Value
match &response.content {
    serde_json::Value::Object(obj) => {
        // Structured JSON response
        if let Some(code) = obj.get("code") {
            println!("Generated code: {}", code);
        }
    }
    serde_json::Value::String(text) => {
        // Plain text response (fallback)
        println!("Text response: {}", text);
    }
    _ => {
        // Other JSON types (arrays, numbers, etc.)
        println!("Other JSON: {}", response.content);
    }
}
```

### **Custom Implementation**
```rust
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

## 🛠️ **Useful Commands**

```bash
# Development
cargo clippy -p tyl-llm-inference-port
cargo fmt -p tyl-llm-inference-port  
cargo doc --no-deps -p tyl-llm-inference-port --open
cargo test -p tyl-llm-inference-port --verbose

# With features
cargo test --all-features -p tyl-llm-inference-port
cargo run --example basic_usage --features mock
cargo build --all-features
```

## 📦 **Dependencies**

### **Runtime**
- `tyl-errors` - TYL unified error handling
- `async-trait` - Async trait support
- `serde` / `serde_json` - JSON serialization support
- `chrono` - Date/time handling
- `uuid` - Unique identifier generation (future use)
- `tokio` (optional) - Async runtime for mock implementations

### **Development**
- `tokio` - Full async runtime for testing
- `futures` - Future combinators for concurrent testing

## 🎯 **Design Principles**

1. **Hexagonal Architecture** - Clean separation between domain logic and adapters
2. **Template-Based Interface** - Simple, flexible prompt construction
3. **JSON-First Responses** - Structured data with automatic fallback to text
4. **Async-First Design** - All operations are async for scalability
5. **Model Type Optimization** - Different models optimized for different use cases
6. **TYL Framework Integration** - Uses TylError, consistent patterns
7. **Builder Pattern** - Fluent APIs for request construction
8. **Comprehensive Testing** - Unit, integration, and doc tests
9. **Health Monitoring** - Built-in health checking for services
10. **Token Management** - First-class token counting and usage tracking

## 🎯 **Model Type Optimization**

- **Coding**: `gpt-4o` / `claude-3-5-sonnet-20241022` - Optimized for code generation
- **Reasoning**: `gpt-4o` / `claude-3-5-sonnet-20241022` - Best for complex analysis
- **General**: `gpt-4o-mini` / `claude-3-5-haiku-20241022` - Balanced performance
- **Fast**: `gpt-3.5-turbo` / `claude-3-5-haiku-20241022` - Speed optimized
- **Creative**: `gpt-4o` / `claude-3-5-sonnet-20241022` - Creative content generation

## 🎨 **Template System**

The template system uses simple `{{parameter}}` placeholders:

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
let rendered = request.render_template();
```

### **Template Best Practices**
- Use descriptive parameter names: `{{user_name}}` not `{{u}}`
- Include context in templates: "As a {{role}}, please {{task}}"
- Consider model types when designing templates
- Test template rendering before sending requests

## 📊 **Mock Response Structures**

The mock service generates different JSON structures based on model type:

### **Coding** (`ModelType::Coding`)
```json
{
  "code": "// Generated code...",
  "language": "rust",
  "explanation": "This is a basic program..."
}
```

### **Reasoning** (`ModelType::Reasoning`)
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

### **Creative** (`ModelType::Creative`)
```json
{
  "story": "Once upon a time...",
  "genre": "fantasy",
  "mood": "whimsical"
}
```

### **General/Fast**
```json
{
  "message": "Response content...",
  "response": "Quick response..."
}
```

## ⚠️ **Known Limitations**

- Mock implementations use simple token estimation (4 chars ≈ 1 token)
- Template system is basic string replacement (no advanced templating features)
- No streaming support yet (planned for future versions)
- No built-in caching (to be implemented in adapters)
- Token counting is approximate for non-OpenAI models

## 📝 **Notes for Contributors**

- Follow TDD approach with comprehensive test coverage
- Maintain hexagonal architecture separation
- Document all public APIs with examples
- Add integration tests for new features
- Keep dependencies minimal and optional where possible
- Use builder patterns for complex request types
- Implement proper error handling with TylError
- Include health checks in all adapter implementations
- Test template rendering thoroughly
- Validate JSON parsing and fallback behavior

## 🚀 **Future Enhancements**

- Advanced templating engines (Handlebars, Tera, etc.)
- Streaming response support
- Built-in response caching with TTL
- Rate limiting and retry logic with backoff
- More sophisticated token counting per provider
- Template parameter validation
- Function calling support
- Batch processing capabilities
- Response format validation
- Template library/registry system

## 🔗 **Related TYL Modules**

- [`tyl-errors`](https://github.com/the-yaml-life/tyl-errors) - Unified error handling
- [`tyl-config`](https://github.com/the-yaml-life/tyl-config) - Configuration management
- [`tyl-logging`](https://github.com/the-yaml-life/tyl-logging) - Structured logging
- [`tyl-tracing`](https://github.com/the-yaml-life/tyl-tracing) - Distributed tracing
- [`tyl-embeddings-port`](https://github.com/the-yaml-life/tyl-embeddings-port) - Embedding generation

## 📚 **Examples and Tutorials**

See `examples/basic_usage.rs` for comprehensive examples covering:
- Template-based prompts with parameter substitution
- JSON response parsing and handling
- Different model types and optimization
- Health monitoring and service info
- Token counting and usage tracking
- Error handling patterns
- Custom responses for testing
- Concurrent request processing

## 🎯 **Migration from Complex Interface**

If migrating from the previous complex interface (CompletionRequest, ChatRequest, etc.):

### **Before (Complex)**
```rust
// Multiple request types
let completion_request = CompletionRequest::new("Write code", ModelType::Coding);
let chat_request = ChatRequest::new(messages, ModelType::General);

// Multiple service traits
let inference_service: Box<dyn InferenceService> = ...;
let chat_service: Box<dyn ChatService> = ...;

let completion = inference_service.complete(completion_request).await?;
let chat_response = chat_service.chat(chat_request).await?;
```

### **After (Simplified)**
```rust
// Single request type with templates
let mut params = HashMap::new();
params.insert("task", "Write code");

let request = InferenceRequest::new("{{task}}", params, ModelType::Coding);

// Single service trait
let service: Box<dyn InferenceService> = ...;
let response = service.infer(request).await?;

// JSON response handling
match &response.content {
    serde_json::Value::Object(obj) => { /* handle JSON */ }
    serde_json::Value::String(text) => { /* handle text */ }
    _ => { /* handle other types */ }
}
```

The new interface is simpler, more flexible, and easier to extend with new use cases.