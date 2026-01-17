use dashmap::DashMap;
use std::sync::Arc;
use crate::types::NfProfile;
use rand::Rng;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub failure_count: usize,
    pub last_failure: Option<Instant>,
    pub circuit_open_until: Option<Instant>,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            is_healthy: true,
            failure_count: 0,
            last_failure: None,
            circuit_open_until: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StickySession {
    pub nf_instance_id: String,
    pub nf_type: String,
    pub created_at: Instant,
}

pub struct LoadBalancer {
    round_robin_index: Arc<DashMap<String, usize>>,
    connection_counts: Arc<DashMap<String, usize>>,
    health_status: Arc<DashMap<String, HealthStatus>>,
    sticky_sessions: Arc<DashMap<String, StickySession>>,
    failure_threshold: usize,
    circuit_timeout: Duration,
    session_ttl: Duration,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            round_robin_index: Arc::new(DashMap::new()),
            connection_counts: Arc::new(DashMap::new()),
            health_status: Arc::new(DashMap::new()),
            sticky_sessions: Arc::new(DashMap::new()),
            failure_threshold: 3,
            circuit_timeout: Duration::from_secs(30),
            session_ttl: Duration::from_secs(300),
        }
    }

    pub fn get_sticky_session(&self, session_id: &str, nf_type: &str) -> Option<String> {
        let now = Instant::now();

        if let Some(session) = self.sticky_sessions.get(session_id) {
            if session.nf_type == nf_type {
                let age = now.duration_since(session.created_at);
                if age < self.session_ttl {
                    if self.get_health_status(&session.nf_instance_id) {
                        return Some(session.nf_instance_id.clone());
                    } else {
                        tracing::debug!("Sticky session for {} expired due to unhealthy instance", session_id);
                        self.sticky_sessions.remove(session_id);
                    }
                } else {
                    tracing::debug!("Sticky session for {} expired (TTL exceeded)", session_id);
                    self.sticky_sessions.remove(session_id);
                }
            }
        }

        None
    }

    pub fn set_sticky_session(&self, session_id: &str, nf_instance_id: &str, nf_type: &str) {
        let session = StickySession {
            nf_instance_id: nf_instance_id.to_string(),
            nf_type: nf_type.to_string(),
            created_at: Instant::now(),
        };

        self.sticky_sessions.insert(session_id.to_string(), session);
        tracing::debug!("Created sticky session: {} -> {}", session_id, nf_instance_id);
    }

    pub fn cleanup_expired_sessions(&self) {
        let now = Instant::now();

        self.sticky_sessions.retain(|_, session| {
            now.duration_since(session.created_at) < self.session_ttl
        });
    }

    pub fn filter_healthy<'a>(&self, instances: &'a [NfProfile]) -> Vec<&'a NfProfile> {
        let now = Instant::now();

        instances
            .iter()
            .filter(|instance| {
                if let Some(health) = self.health_status.get(&instance.nf_instance_id) {
                    if let Some(circuit_open_until) = health.circuit_open_until {
                        if now < circuit_open_until {
                            return false;
                        }
                    }
                    health.is_healthy
                } else {
                    true
                }
            })
            .collect()
    }

    pub fn select_round_robin<'a>(&self, nf_type: &str, instances: &'a [NfProfile]) -> &'a NfProfile {
        if instances.is_empty() {
            panic!("Cannot select from empty instances list");
        }

        let healthy = self.filter_healthy(instances);
        let instances_to_use: &[&NfProfile] = if healthy.is_empty() {
            tracing::warn!("No healthy instances for {}, using all instances", nf_type);
            &instances.iter().collect::<Vec<_>>()
        } else {
            &healthy
        };

        if instances_to_use.len() == 1 {
            return instances_to_use[0];
        }

        let mut entry = self.round_robin_index.entry(nf_type.to_string()).or_insert(0);
        let current_index = *entry;
        let selected_index = current_index % instances_to_use.len();

        *entry = (current_index + 1) % instances_to_use.len();

        instances_to_use[selected_index]
    }

    pub fn select_least_connections<'a>(&self, instances: &'a [NfProfile]) -> &'a NfProfile {
        if instances.is_empty() {
            panic!("Cannot select from empty instances list");
        }

        let healthy = self.filter_healthy(instances);
        let instances_to_use: Vec<&NfProfile> = if healthy.is_empty() {
            tracing::warn!("No healthy instances, using all instances");
            instances.iter().collect()
        } else {
            healthy
        };

        if instances_to_use.len() == 1 {
            return instances_to_use[0];
        }

        let selected = instances_to_use
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

        let healthy = self.filter_healthy(instances);
        let instances_to_use: Vec<&NfProfile> = if healthy.is_empty() {
            tracing::warn!("No healthy instances, using all instances");
            instances.iter().collect()
        } else {
            healthy
        };

        if instances_to_use.len() == 1 {
            return instances_to_use[0];
        }

        let total_capacity: u32 = instances_to_use
            .iter()
            .map(|instance| instance.capacity.unwrap_or(100))
            .sum();

        if total_capacity == 0 {
            return instances_to_use[0];
        }

        let mut rng = rand::thread_rng();
        let mut random_value = rng.gen_range(0..total_capacity);

        for instance in &instances_to_use {
            let capacity = instance.capacity.unwrap_or(100);
            if random_value < capacity {
                return instance;
            }
            random_value -= capacity;
        }

        instances_to_use[instances_to_use.len() - 1]
    }

    pub fn select_with_sticky_session<'a>(
        &self,
        session_id: &str,
        nf_type: &str,
        instances: &'a [NfProfile],
    ) -> &'a NfProfile {
        if instances.is_empty() {
            panic!("Cannot select from empty instances list");
        }

        if let Some(sticky_instance_id) = self.get_sticky_session(session_id, nf_type) {
            if let Some(instance) = instances
                .iter()
                .find(|i| i.nf_instance_id == sticky_instance_id)
            {
                tracing::debug!(
                    "Using sticky session: {} -> {}",
                    session_id,
                    sticky_instance_id
                );
                return instance;
            } else {
                tracing::debug!("Sticky instance {} not in available instances", sticky_instance_id);
                self.sticky_sessions.remove(session_id);
            }
        }

        let selected = self.select_least_connections(instances);
        self.set_sticky_session(session_id, &selected.nf_instance_id, nf_type);

        selected
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

    pub fn mark_failure(&self, nf_instance_id: &str) {
        let now = Instant::now();

        let mut entry = self.health_status
            .entry(nf_instance_id.to_string())
            .or_insert_with(HealthStatus::default);

        entry.failure_count += 1;
        entry.last_failure = Some(now);

        if entry.failure_count >= self.failure_threshold {
            entry.is_healthy = false;
            entry.circuit_open_until = Some(now + self.circuit_timeout);
            tracing::warn!(
                "NF instance {} marked unhealthy after {} failures, circuit open for {:?}",
                nf_instance_id,
                entry.failure_count,
                self.circuit_timeout
            );
        }
    }

    pub fn mark_success(&self, nf_instance_id: &str) {
        if let Some(mut entry) = self.health_status.get_mut(nf_instance_id) {
            if !entry.is_healthy {
                tracing::info!("NF instance {} recovered", nf_instance_id);
            }
            entry.is_healthy = true;
            entry.failure_count = 0;
            entry.circuit_open_until = None;
        }
    }

    pub fn get_health_status(&self, nf_instance_id: &str) -> bool {
        let now = Instant::now();

        if let Some(health) = self.health_status.get(nf_instance_id) {
            if let Some(circuit_open_until) = health.circuit_open_until {
                if now >= circuit_open_until {
                    return true;
                }
            }
            health.is_healthy
        } else {
            true
        }
    }
}

impl Clone for LoadBalancer {
    fn clone(&self) -> Self {
        Self {
            round_robin_index: Arc::clone(&self.round_robin_index),
            connection_counts: Arc::clone(&self.connection_counts),
            health_status: Arc::clone(&self.health_status),
            sticky_sessions: Arc::clone(&self.sticky_sessions),
            failure_threshold: self.failure_threshold,
            circuit_timeout: self.circuit_timeout,
            session_ttl: self.session_ttl,
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
