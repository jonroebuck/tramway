use std::sync::Arc;
use crate::registry::AdapterRegistry;

/// Shared application state, cloned into every Axum handler.
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<AdapterRegistry>,
}

impl AppState {
    pub fn new(registry: AdapterRegistry) -> Self {
        AppState {
            registry: Arc::new(registry),
        }
    }
}
