use crate::event::Event;
use std::hash::{Hash, Hasher};
pub struct ScheduledEvent<W> {
    pub id: u64,
    pub event: Box<dyn Event<W>>,
}

impl<W> Hash for ScheduledEvent<W> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<W> PartialEq for ScheduledEvent<W> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<W> Eq for ScheduledEvent<W> {}

#[cfg(test)]
mod tests {
    use super::*;

    // mock event for testing
    struct MockEvent;
    
    impl Event<()> for MockEvent {
        fn execute(&self, _world: &mut (), _current_tick: u64, _scheduler: &mut crate::engine::Scheduler<()>) {
        }
    }

    #[test]
    fn test_scheduled_event_equality_same_id() {
        let event1 = ScheduledEvent {
            id: 42,
            event: Box::new(MockEvent),
        };
        let event2 = ScheduledEvent {
            id: 42,
            event: Box::new(MockEvent),
        };

        assert!(event1 == event2);
    }

    #[test]
    fn test_scheduled_event_inequality_different_id() {
        let event1 = ScheduledEvent {
            id: 42,
            event: Box::new(MockEvent),
        };
        let event2 = ScheduledEvent {
            id: 100,
            event: Box::new(MockEvent),
        };

        assert!(event1 != event2);
    }

    #[test]
    fn test_scheduled_event_eq_reflexive() {
        let event = ScheduledEvent {
            id: 42,
            event: Box::new(MockEvent),
        };

        assert!(event == event);
    }
}
