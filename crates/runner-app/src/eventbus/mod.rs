use runner_core::domain::RunEvent;
use std::sync::{Arc, Mutex};

pub trait EventSubscriber: Send + Sync {
    fn on_event(&self, event: &RunEvent);
}

#[derive(Clone, Default)]
pub struct EventBus {
    subscribers: Vec<Arc<dyn EventSubscriber>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe<S: EventSubscriber + 'static>(&mut self, subscriber: S) {
        self.subscribers.push(Arc::new(subscriber));
    }

    pub fn publish(&self, event: &RunEvent) {
        for sub in &self.subscribers {
            sub.on_event(event);
        }
    }
}

#[derive(Clone, Default)]
pub struct InMemoryEventCollector {
    events: Arc<Mutex<Vec<RunEvent>>>,
}

impl InMemoryEventCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> Vec<RunEvent> {
        self.events.lock().map(|v| v.clone()).unwrap_or_default()
    }
}

impl EventSubscriber for InMemoryEventCollector {
    fn on_event(&self, event: &RunEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eventbus_publish_and_collect() {
        let mut bus = EventBus::new();
        let collector = InMemoryEventCollector::new();
        let probe = collector.clone();
        bus.subscribe(collector);

        bus.publish(&RunEvent {
            kind: "run_started".to_string(),
            task_id: None,
        });
        bus.publish(&RunEvent {
            kind: "task_started".to_string(),
            task_id: Some("a".to_string()),
        });

        let events = probe.snapshot();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, "run_started");
        assert_eq!(events[1].task_id.as_deref(), Some("a"));
    }
}
