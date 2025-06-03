# Agent Development Kit (ADK) - Rust

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/google-adk.svg)](https://crates.io/crates/google-adk)
[![Documentation](https://docs.rs/google-adk/badge.svg)](https://docs.rs/google-adk)

<html>
    <h2 align="center">
      <img src="https://raw.githubusercontent.com/google/adk-python/main/assets/agent-development-kit.png" width="256"/>
    </h2>
    <h3 align="center">
      An open-source, code-first Rust toolkit for building, evaluating, and deploying sophisticated AI agents with flexibility and control.
    </h3>
    <h3 align="center">
      Important Links:
      <a href="https://google.github.io/adk-docs/">Docs</a>, 
      <a href="https://github.com/google/adk-samples">Samples</a>,
      <a href="https://github.com/google/adk-python">Python ADK</a> &
      <a href="https://github.com/google/adk-web">ADK Web</a>.
    </h3>
</html>

Agent Development Kit (ADK) is a flexible and modular framework for developing and deploying AI agents. While optimized for Gemini and the Google ecosystem, ADK is model-agnostic, deployment-agnostic, and is built for compatibility with other frameworks. ADK was designed to make agent development feel more like software development, to make it easier for developers to create, deploy, and orchestrate agentic architectures that range from simple tasks to complex workflows.

This is the **Rust implementation** of the ADK, providing high-performance, memory-safe agent development with the power and safety of Rust.

---

## ✨ Key Features

- **Rich Tool Ecosystem**: Utilize pre-built tools, custom functions,
  OpenAPI specs, or integrate existing tools to give agents diverse
  capabilities, all for tight integration with the Google ecosystem.

- **Code-First Development**: Define agent logic, tools, and orchestration
  directly in Rust for ultimate flexibility, testability, and versioning.

- **Modular Multi-Agent Systems**: Design scalable applications by composing
  multiple specialized agents into flexible hierarchies.

- **Deploy Anywhere**: Easily containerize and deploy agents on Cloud Run or
  scale seamlessly with Vertex AI Agent Engine.

- **High Performance**: Built with Rust for maximum performance and memory safety.

- **Async-First**: Built on Tokio for high-concurrency agent execution.

## 🚀 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
google-adk = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## 🏁 Quick Start

### Define a single agent:

```rust
use google_adk::{agents::LlmAgent, tools::google_search, Result};

#[tokio::main]
async fn main() -> Result<()> {
    google_adk::init()?;
    
    let agent = LlmAgent::builder()
        .name("search_assistant")
        .model("gemini-2.0-flash") // Or your preferred Gemini model
        .instruction("You are a helpful assistant. Answer user questions using Google Search when needed.")
        .description("An assistant that can search the web.")
        .tool(google_search())
        .build()?;

    println!("Agent created: {}", agent.name());
    Ok(())
}
```

### Define a multi-agent system:

```rust
use google_adk::{agents::LlmAgent, Result};

#[tokio::main]
async fn main() -> Result<()> {
    google_adk::init()?;
    
    // Define individual agents
    let greeter = LlmAgent::builder()
        .name("greeter")
        .model("gemini-2.0-flash")
        .instruction("You greet users warmly.")
        .build()?;
    
    let task_executor = LlmAgent::builder()
        .name("task_executor")
        .model("gemini-2.0-flash")
        .instruction("You execute tasks efficiently.")
        .build()?;

    // Create parent agent and assign children via sub_agents
    let coordinator = LlmAgent::builder()
        .name("coordinator")
        .model("gemini-2.0-flash")
        .description("I coordinate greetings and tasks.")
        .sub_agent(greeter)
        .sub_agent(task_executor)
        .build()?;

    println!("Multi-agent system created with coordinator: {}", coordinator.name());
    Ok(())
}
```

## 🛠️ CLI Usage

The ADK comes with a powerful CLI for agent development:

```bash
# Create a new agent project
adk create my_agent --model gemini-2.0-flash

# Run an agent interactively
adk run path/to/my_agent

# Evaluate an agent
adk eval path/to/my_agent path/to/eval_set.json

# Start a web server with UI
adk web path/to/agents_dir --port 8000

# Start an API server
adk api_server path/to/agents_dir --port 8000

# Deploy to Cloud Run
adk deploy cloud_run path/to/my_agent --project my-project --region us-central1

# Deploy to Vertex AI
adk deploy vertex_ai path/to/my_agent --project my-project --region us-central1
```

## 🏗️ Architecture

The Rust ADK is built with the following core components:

- **Agents**: `BaseAgent`, `LlmAgent`, `SequentialAgent`, `ParallelAgent`, `LoopAgent`
- **Models**: `BaseLlm`, `GoogleLlm`, `AnthropicLlm` (with feature flags)
- **Tools**: `BaseTool`, `FunctionTool`, `BaseToolset`
- **Events**: Event-driven communication between agents
- **Sessions**: Session management for conversation state
- **Memory**: Persistent memory for agents
- **Artifacts**: File and cloud storage management
- **Runners**: Execution engine for agents
- **Web**: HTTP API and WebSocket support
- **CLI**: Command-line interface for development

## 🔧 Features

### Default Features
- `google-ai`: Google AI/Gemini model support

### Optional Features
- `anthropic`: Anthropic Claude model support

Enable features in your `Cargo.toml`:

```toml
[dependencies]
google-adk = { version = "0.1", features = ["anthropic"] }
```

## 📚 Documentation

Explore the full documentation for detailed guides on building, evaluating, and
deploying agents:

* **[Documentation](https://docs.rs/google-adk)**
* **[API Reference](https://docs.rs/google-adk)**
* **[Examples](examples/)**

## 🤝 Contributing

We welcome contributions from the community! Whether it's bug reports, feature requests, documentation improvements, or code contributions, please see our
- [General contribution guideline and flow](https://google.github.io/adk-docs/contributing-guide/#questions).
- Then if you want to contribute code, please read [CONTRIBUTING.md](./CONTRIBUTING.md) to get started.

## 📄 License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.

---

*Happy Agent Building with Rust!*
