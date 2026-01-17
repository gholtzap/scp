use dashmap::DashMap;
use std::sync::Arc;
use crate::types::NfProfile;
use rand::Rng;

pub struct LoadBalancer {
    round_robin_index: Arc<DashMap<String, usize>>,
    connection_counts: Arc<DashMap<String, usize>>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            round_robin_index: Arc::new(DashMap::new()),
            connection_counts: Arc::new(DashMap::new()),
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

    pub fn select_least_connections<'a>(&self, instances: &'a [NfProfile]) -> &'a NfProfile {
        if instances.is_empty() {
            panic!("Cannot select from empty instances list");
        }

        if instances.len() == 1 {
            return &instances[0];
        }

        let selected = instances
            .iter()
            .min_by_key(|instance| {
                self.connection_counts
                    .get(&instance.nf_instance_id)
                    .map(|count| *count)
                    .unwrap_or(0)
            })
            .expect("instances is not empty");

        selected
    }

    pub fn select_weighted<'a>(&self, instances: &'a [NfProfile]) -> &'a NfProfile {
        if instances.is_empty() {
            panic!("Cannot select from empty instances list");
        }

        if instances.len() == 1 {
            return &instances[0];
        }

        let total_capacity: u32 = instances
            .iter()
            .map(|instance| instance.capacity.unwrap_or(100))
            .sum();

        if total_capacity == 0 {
            return &instances[0];
        }

        let mut rng = rand::thread_rng();
        let mut random_value = rng.gen_range(0..total_capacity);

        for instance in instances {
            let capacity = instance.capacity.unwrap_or(100);
            if random_value < capacity {
                return instance;
            }
            random_value -= capacity;
        }

        &instances[instances.len() - 1]
    }

    pub fn increment_connections(&self, nf_instance_id: &str) {
        self.connection_counts
            .entry(nf_instance_id.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    pub fn decrement_connections(&self, nf_instance_id: &str) {
        if let Some(mut count) = self.connection_counts.get_mut(nf_instance_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    pub fn acquire_connection(&self, nf_instance_id: String) -> ConnectionGuard {
        self.increment_connections(&nf_instance_id);
        ConnectionGuard {
            nf_instance_id,
            load_balancer: self.clone(),
        }
    }
}

impl Clone for LoadBalancer {
    fn clone(&self) -> Self {
        Self {
            round_robin_index: Arc::clone(&self.round_robin_index),
            connection_counts: Arc::clone(&self.connection_counts),
        }
    }
}

pub struct ConnectionGuard {
    nf_instance_id: String,
    load_balancer: LoadBalancer,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.load_balancer.decrement_connections(&self.nf_instance_id);
    }
}
