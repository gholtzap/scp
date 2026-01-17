use dashmap::DashMap;
use std::sync::Arc;
use crate::types::NfProfile;

pub struct LoadBalancer {
    round_robin_index: Arc<DashMap<String, usize>>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            round_robin_index: Arc::new(DashMap::new()),
        }
    }

    pub fn select_round_robin<'a>(&self, nf_type: &str, instances: &'a [NfProfile]) -> &'a NfProfile {
        if instances.is_empty() {
            panic!("Cannot select from empty instances list");
        }

        if instances.len() == 1 {
            return &instances[0];
        }

        let mut entry = self.round_robin_index.entry(nf_type.to_string()).or_insert(0);
        let current_index = *entry;
        let selected_index = current_index % instances.len();

        *entry = (current_index + 1) % instances.len();

        &instances[selected_index]
    }
}

impl Clone for LoadBalancer {
    fn clone(&self) -> Self {
        Self {
            round_robin_index: Arc::clone(&self.round_robin_index),
        }
    }
}
