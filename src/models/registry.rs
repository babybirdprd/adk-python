//! LLM registry for model management

use crate::{
    error::Result,
    models::{BaseLlm, GoogleLlm},
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Registry for LLM models
pub struct LlmRegistry {
    models: Arc<RwLock<HashMap<String, Box<dyn Fn(&str) -> Result<Box<dyn BaseLlm>> + Send + Sync>>>>,
}

impl LlmRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        let registry = Self {
            models: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Register default models
        tokio::spawn({
            let registry = registry.clone();
            async move {
                registry.register_default_models().await;
            }
        });
        
        registry
    }

    /// Register default models
    async fn register_default_models(&self) {
        info!("Registering default LLM models");
        
        // Register Google/Gemini models
        self.register_google_models().await;
        
        #[cfg(feature = "anthropic")]
        self.register_anthropic_models().await;
        
        debug!("Default models registered successfully");
    }

    /// Register Google models
    async fn register_google_models(&self) {
        let mut models = self.models.write().await;
        
        // Register Gemini models with various patterns
        let gemini_patterns = vec![
            "gemini",
            "gemini-pro",
            "gemini-flash",
            "gemini-1.0",
            "gemini-1.5",
            "gemini-2.0",
        ];
        
        for pattern in gemini_patterns {
            models.insert(
                pattern.to_string(),
                Box::new(|model_name: &str| {
                    let mut llm = GoogleLlm::new(model_name);
                    
                    // Auto-configure from environment
                    if let Ok(api_key) = std::env::var("GOOGLE_API_KEY") {
                        llm = llm.with_api_key(api_key);
                    }
                    
                    if let Ok(project_id) = std::env::var("GOOGLE_CLOUD_PROJECT") {
                        llm = llm.with_project_id(project_id);
                    }
                    
                    if let Ok(region) = std::env::var("GOOGLE_CLOUD_REGION") {
                        llm = llm.with_region(region);
                    }
                    
                    // Use Vertex AI if project and region are set
                    if std::env::var("GOOGLE_CLOUD_PROJECT").is_ok() && 
                       std::env::var("GOOGLE_CLOUD_REGION").is_ok() {
                        llm = llm.use_vertex_ai();
                    }
                    
                    Ok(Box::new(llm) as Box<dyn BaseLlm>)
                }),
            );
        }
        
        debug!("Google/Gemini models registered");
    }

    /// Register Anthropic models
    #[cfg(feature = "anthropic")]
    async fn register_anthropic_models(&self) {
        let mut models = self.models.write().await;
        
        models.insert(
            "claude".to_string(),
            Box::new(|model_name: &str| {
                // TODO: Implement AnthropicLlm
                Err(crate::adk_error!(
                    ModelError,
                    "Anthropic models not yet implemented: {}",
                    model_name
                ))
            }),
        );
        
        debug!("Anthropic models registered");
    }

    /// Register a model factory
    pub async fn register<F>(&self, pattern: String, factory: F)
    where
        F: Fn(&str) -> Result<Box<dyn BaseLlm>> + Send + Sync + 'static,
    {
        let mut models = self.models.write().await;
        debug!("Registered custom model pattern: {}", pattern);
        models.insert(pattern, Box::new(factory));
    }

    /// Create a model instance
    pub async fn create_model(&self, model_name: &str) -> Result<Box<dyn BaseLlm>> {
        let models = self.models.read().await;
        
        debug!("Creating model instance for: {}", model_name);
        
        // Try exact match first
        if let Some(factory) = models.get(model_name) {
            return factory(model_name);
        }
        
        // Try pattern matching
        for (pattern, factory) in models.iter() {
            if model_name.starts_with(pattern) || model_name.contains(pattern) {
                debug!("Matched pattern '{}' for model '{}'", pattern, model_name);
                return factory(model_name);
            }
        }
        
        Err(crate::adk_error!(
            ModelError,
            "No registered model found for: {}. Available patterns: {:?}",
            model_name,
            models.keys().collect::<Vec<_>>()
        ))
    }

    /// List available model patterns
    pub async fn list_patterns(&self) -> Vec<String> {
        let models = self.models.read().await;
        models.keys().cloned().collect()
    }

    /// Check if a model is supported
    pub async fn is_supported(&self, model_name: &str) -> bool {
        let models = self.models.read().await;
        
        // Check exact match
        if models.contains_key(model_name) {
            return true;
        }
        
        // Check pattern match
        for pattern in models.keys() {
            if model_name.starts_with(pattern) || model_name.contains(pattern) {
                return true;
            }
        }
        
        false
    }

    /// Get model information
    pub async fn get_model_info(&self, model_name: &str) -> Result<ModelInfo> {
        if !self.is_supported(model_name).await {
            return Err(crate::adk_error!(
                ModelError,
                "Model not supported: {}",
                model_name
            ));
        }

        // Create a temporary instance to get capabilities
        let model = self.create_model(model_name).await?;
        
        Ok(ModelInfo {
            name: model_name.to_string(),
            supports_streaming: model.supports_streaming(),
            supports_function_calling: model.supports_function_calling(),
            supports_multimodal: model.supports_multimodal(),
            supports_live: model.supports_live(),
        })
    }
}

impl Clone for LlmRegistry {
    fn clone(&self) -> Self {
        Self {
            models: self.models.clone(),
        }
    }
}

impl Default for LlmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a model's capabilities
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub supports_multimodal: bool,
    pub supports_live: bool,
}

/// Global registry instance
static GLOBAL_REGISTRY: once_cell::sync::Lazy<LlmRegistry> = 
    once_cell::sync::Lazy::new(|| LlmRegistry::new());

/// Get the global registry
pub fn global_registry() -> &'static LlmRegistry {
    &GLOBAL_REGISTRY
}

/// Create a model using the global registry
pub async fn create_model(model_name: &str) -> Result<Box<dyn BaseLlm>> {
    global_registry().create_model(model_name).await
}

/// Check if a model is supported
pub async fn is_model_supported(model_name: &str) -> bool {
    global_registry().is_supported(model_name).await
}

/// Get model information
pub async fn get_model_info(model_name: &str) -> Result<ModelInfo> {
    global_registry().get_model_info(model_name).await
}

/// List all available model patterns
pub async fn list_available_models() -> Vec<String> {
    global_registry().list_patterns().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_registry() {
        let registry = LlmRegistry::new();
        
        // Wait for async registration to complete
        sleep(Duration::from_millis(100)).await;
        
        let patterns = registry.list_patterns().await;
        assert!(!patterns.is_empty());
        
        // Test Gemini model creation
        let model = registry.create_model("gemini-2.0-flash").await;
        assert!(model.is_ok());
        
        let model = model.unwrap();
        assert_eq!(model.model_name(), "gemini-2.0-flash");
    }

    #[tokio::test]
    async fn test_global_registry() {
        // Wait for async registration to complete
        sleep(Duration::from_millis(100)).await;
        
        assert!(is_model_supported("gemini-pro").await);
        
        let model = create_model("gemini-pro").await;
        assert!(model.is_ok());
        
        let info = get_model_info("gemini-2.0-flash").await;
        assert!(info.is_ok());
        
        let info = info.unwrap();
        assert!(info.supports_function_calling);
    }
}
