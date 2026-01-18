pub trait Event<W> {
    fn execute(&self, world: &mut W, current_tick: u64) -> Vec<(Box<dyn Event<W>>, u64)>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWorld {
        counter: u32,
    }

    // event that increments the counter
    struct IncrementEvent {
        amount: u32,
    }

    impl Event<TestWorld> for IncrementEvent {
        fn execute(&self, world: &mut TestWorld, _current_tick: u64) -> Vec<(Box<dyn Event<TestWorld>>, u64)> {
            world.counter += self.amount;
            vec![]
        }
    }

    // event that returns new scheduled events
    struct SchedulingEvent {
        schedule_count: usize,
    }

    impl Event<TestWorld> for SchedulingEvent {
        fn execute(&self, _world: &mut TestWorld, current_tick: u64) -> Vec<(Box<dyn Event<TestWorld>>, u64)> {
            (0..self.schedule_count)
                .map(|i| {
                    let event = Box::new(IncrementEvent { amount: 1 }) as Box<dyn Event<TestWorld>>;
                    (event, current_tick + i as u64 + 1)
                })
                .collect()
        }
    }

    #[test]
    fn test_event_executes_and_modifies_world() {
        let mut world = TestWorld { counter: 0 };
        let event = IncrementEvent { amount: 5 };

        let scheduled_events = event.execute(&mut world, 0);

        assert_eq!(world.counter, 5);
        assert_eq!(scheduled_events.len(), 0);
    }

    #[test]
    fn test_event_returns_no_scheduled_events() {
        let mut world = TestWorld { counter: 0 };
        let event = IncrementEvent { amount: 1 };

        let scheduled_events = event.execute(&mut world, 10);

        assert!(scheduled_events.is_empty());
    }

    #[test]
    fn test_event_returns_scheduled_events() {
        let mut world = TestWorld { counter: 0 };
        let event = SchedulingEvent { schedule_count: 3 };

        let scheduled_events = event.execute(&mut world, 10);

        assert_eq!(scheduled_events.len(), 3);
        assert_eq!(scheduled_events[0].1, 11);
        assert_eq!(scheduled_events[1].1, 12);
        assert_eq!(scheduled_events[2].1, 13);
    }

    #[test]
    fn test_event_current_tick_parameter() {
        let mut world = TestWorld { counter: 0 };
        let event = SchedulingEvent { schedule_count: 1 };

        let scheduled_events_tick_5 = event.execute(&mut world, 5);
        let scheduled_events_tick_100 = event.execute(&mut world, 100);

        assert_eq!(scheduled_events_tick_5[0].1, 6);
        assert_eq!(scheduled_events_tick_100[0].1, 101);
    }

    #[test]
    fn test_event_with_zero_schedules() {
        let mut world = TestWorld { counter: 0 };
        let event = SchedulingEvent { schedule_count: 0 };

        let scheduled_events = event.execute(&mut world, 10);

        assert_eq!(scheduled_events.len(), 0);
    }
}
