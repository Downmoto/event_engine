use crate::engine::Scheduler;

pub trait Event<W> {
    fn execute(&self, world: &mut W, current_tick: u64, scheduler: &mut Scheduler<W>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Engine;

    struct TestWorld {
        counter: u32,
    }

    // event that increments the counter
    struct IncrementEvent {
        amount: u32,
    }

    impl Event<TestWorld> for IncrementEvent {
        fn execute(&self, world: &mut TestWorld, _current_tick: u64, _scheduler: &mut Scheduler<TestWorld>) {
            world.counter += self.amount;
        }
    }

    // event that returns new scheduled events
    struct SchedulingEvent {
        schedule_count: usize,
    }

    impl Event<TestWorld> for SchedulingEvent {
        fn execute(&self, _world: &mut TestWorld, _current_tick: u64, scheduler: &mut Scheduler<TestWorld>) {
            for i in 0..self.schedule_count {
                let event = Box::new(IncrementEvent { amount: 1 });
                scheduler.schedule(event, i as u64 + 1);
            }
        }
    }

    #[test]
    fn test_event_executes_and_modifies_world() {
        let mut world = TestWorld { counter: 0 };
        let mut engine = Engine::new();

        engine.schedule(Box::new(IncrementEvent { amount: 5 }), 1);
        engine.step(&mut world);

        assert_eq!(world.counter, 5);
    }

    #[test]
    fn test_event_schedules_child_events() {
        let mut world = TestWorld { counter: 0 };
        let mut engine = Engine::new();

        engine.schedule(Box::new(SchedulingEvent { schedule_count: 3 }), 1);
        
        for _ in 0..5 {
            engine.step(&mut world);
        }

        assert_eq!(world.counter, 3);
    }

    #[test]
    fn test_scheduling_uses_correct_timing() {
        let mut world = TestWorld { counter: 0 };
        let mut engine = Engine::new();

        // schedule at tick 5 - it schedules a child event with delay of 1
        engine.schedule(Box::new(SchedulingEvent { schedule_count: 1 }), 5);
        
        for _ in 0..5 {
            engine.step(&mut world);
        }
        assert_eq!(world.counter, 0); // child not executed yet

        // step to tick 6 - child executes
        engine.step(&mut world);
        assert_eq!(world.counter, 1);
    }

    #[test]
    fn test_event_with_zero_schedules() {
        let mut world = TestWorld { counter: 0 };
        let mut engine = Engine::new();

        engine.schedule(Box::new(SchedulingEvent { schedule_count: 0 }), 1);
        
        // step multiple times
        for _ in 0..5 {
            engine.step(&mut world);
        }

        assert_eq!(world.counter, 0);
    }
}
