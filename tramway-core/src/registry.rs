use std::collections::HashMap;
use std::sync::Arc;
use crate::Intelligence;

pub type BoxedIntelligence = Arc<dyn Intelligence + Send + Sync>;

#[derive(Default)]
pub struct IntelligenceRegistry {
    adapters: HashMap<String, BoxedIntelligence>,
}

impl IntelligenceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        mut self,
        name: impl Into<String>,
        adapter: impl Intelligence + Send + Sync + 'static,
    ) -> Self {
        self.adapters.insert(name.into(), Arc::new(adapter));
        self
    }

    pub fn get(&self, name: &str) -> Option<&BoxedIntelligence> {
        self.adapters.get(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.adapters.keys()
    }
}
