use std::collections::HashMap;
use std::sync::Arc;

use crate::worker::Worker;

#[derive(Default)]
pub struct WorkerRegistry {
    workers: HashMap<String, Arc<Worker>>,
}

impl WorkerRegistry {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }

    pub fn insert(&mut self, worker: Arc<Worker>) -> bool {
        let id = worker.id().to_string();
        if self.workers.contains_key(&id) {
            return false;
        }
        self.workers.insert(id, worker);
        true
    }

    pub fn get(&self, id: &str) -> Option<Arc<Worker>> {
        match self.workers.get(id) {
            Some(worker) => Some(Arc::clone(worker)),
            None => None,
        }
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.workers.remove(id).is_some()
    }

    pub fn snapshot(&self) -> Vec<Arc<Worker>> {
        let mut workers: Vec<Arc<Worker>> = self.workers.values().map(Arc::clone).collect();
        workers.sort_by(|left, right| left.id().cmp(right.id()));
        workers
    }
}

#[cfg(test)]
mod tests {
    use super::WorkerRegistry;
    use crate::{
        policy::{FirstHealthy, Policy},
        worker::{HealthStatus, Worker},
    };
    use std::sync::Arc;

    #[test]
    fn same_sanshot_reflects_worker_health_changes() {
        let mut registry = WorkerRegistry::default();
        assert!(registry.insert(Arc::new(Worker::new(
            "worker-a".to_string(),
            "127.0.0.1:3001".to_string()
        ))));
        assert!(registry.insert(Arc::new(Worker::new(
            "worker-b".to_string(),
            "127.0.0.1:3002".to_string()
        ))));

        let snapshot = registry.snapshot();
        let mut policy = FirstHealthy::new();

        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0].id(), "worker-a");
        assert_eq!(snapshot[1].id(), "worker-b");

        assert_eq!(policy.select(&snapshot), Some(0));

        // 从注册表获得同一个 Worker 的另一个 Arc。
        let worker_a = registry
            .get("worker-a")
            .expect("registered worker-a should exist");

        assert!(Arc::ptr_eq(&worker_a, &snapshot[0]));

        worker_a.set_status(crate::worker::HealthStatus::Unhealthy);
        assert_eq!(snapshot[0].status(), HealthStatus::Unhealthy);
        assert_eq!(policy.select(&snapshot), Some(1));
        worker_a.set_status(HealthStatus::Healthy);

        assert_eq!(snapshot[0].status(), HealthStatus::Healthy);
        assert_eq!(policy.select(&snapshot), Some(0));
    }

    #[test]
    fn empty_registry_has_empty_snapshot() {
        let registry = WorkerRegistry::new();

        assert!(registry.snapshot().is_empty());
    }

    #[test]
    fn snapshot_is_sorted_by_worker_id() {
        let mut registry = WorkerRegistry::new();

        assert!(registry.insert(Arc::new(Worker::new(
            "worker-b".to_string(),
            "127.0.0.1:3002".to_string()
        ),)));
        assert!(registry.insert(Arc::new(Worker::new(
            "worker-a".to_string(),
            "127.0.0.1:3001".to_string()
        ),)));

        let snapshot = registry.snapshot();

        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0].id(), "worker-a");
        assert_eq!(snapshot[0].address(), "127.0.0.1:3001");
        assert_eq!(snapshot[1].id(), "worker-b");
        assert_eq!(snapshot[1].address(), "127.0.0.1:3002");
    }

    #[test]
    fn removal_changes_new_snapshot_but_not_old_snapshot() {
        let mut registry = WorkerRegistry::new();
        assert!(registry.insert(Arc::new(Worker::new(
            "worker-a".to_string(),
            "127.0.0.1:3212".to_string()
        ))));
        assert!(registry.insert(Arc::new(Worker::new(
            "worker-b".to_string(),
            "127.0.0.1:3212".to_string()
        ))));

        let old_snapshot = registry.snapshot();
        let request_worker = registry.get("worker-a").expect("worker-a should exist");

        let guard = request_worker.begin_request();
        assert!(registry.remove("worker-a"));

        let new_snapshot = registry.snapshot();

        assert_eq!(old_snapshot.len(), 2);
        assert_eq!(old_snapshot[0].id(), "worker-a");

        assert_eq!(new_snapshot.len(), 1);
        assert_eq!(new_snapshot[0].id(), "worker-b");

        assert!(Arc::ptr_eq(&old_snapshot[1], &new_snapshot[0]));

        assert_eq!(old_snapshot[0].counter(), 1);
        drop(guard);
        assert_eq!(old_snapshot[0].counter(), 0);
    }

    #[test]
    fn existing_request_survives_worker_removal() {
        let mut registry = WorkerRegistry::new();

        assert!(registry.insert(Arc::new(Worker::new(
            "worker-a".to_string(),
            "127.0.0.1:3212".to_string()
        ),)));

        let request_worker = registry
            .get("worker-a")
            .expect("registered worker should exist");

        let guard = request_worker.begin_request();

        assert!(registry.remove("worker-a"));
        assert!(registry.get("worker-a").is_none());

        assert_eq!(request_worker.id(), "worker-a");
        assert_eq!(request_worker.counter(), 1);

        drop(guard);

        assert_eq!(request_worker.counter(), 0);
    }

    #[test]
    fn rejects_duplicate_id_without_replacing_worker() {
        let mut registry = WorkerRegistry::new();
        let original = Arc::new(Worker::new(
            "worker-a".to_string(),
            "127.0.0.1:3212".to_string(),
        ));

        assert!(registry.insert(Arc::clone(&original)));

        let duplicate = Arc::new(Worker::new(
            "worker-a".to_string(),
            "127.0.0.1:3212".to_string(),
        ));
        assert!(!registry.insert(duplicate));

        let found = registry
            .get("worker-a")
            .expect("original worker should remain");

        assert!(Arc::ptr_eq(&original, &found));
    }

    #[test]
    fn missing_worker_returns_none_and_cannot_be_removed() {
        let mut registry = WorkerRegistry::new();

        assert!(registry.get("missing").is_none());
        assert!(!registry.remove("missing"));
    }
}
