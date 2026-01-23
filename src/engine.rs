use crate::Event;
use crate::Scheduler;
use crate::scheduled_wrapper::ScheduledEvent;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;

pub struct Engine<W> {
    current_tick: u64,
    total_events_executed: u64,
    id_counter: u64,

    queue: PriorityQueue<ScheduledEvent<W>, Reverse<(u64, u64)>>,
    max_executions_per_tick: u64,
}

impl<W> Engine<W> {
    pub fn initial_event_pool(mut self, initial_pool: Vec<(Box<dyn Event<W>>, u64)>) -> Self {
        for (event, delay) in initial_pool {
            self.schedule(event, delay);
        }
        self
    }

    pub fn max_executions_per_tick(mut self, execution_rate: u64) -> Self {
        self.max_executions_per_tick = execution_rate;

        self
    }

    pub fn build() -> Self {
        Self {
            current_tick: 0,
            max_executions_per_tick: 5,
            queue: PriorityQueue::new(),
            id_counter: 0,
            total_events_executed: 0,
        }
    }

    pub fn schedule(&mut self, event: Box<dyn Event<W>>, delay: u64) {
        let mut scheduler = Scheduler {
            current_tick: self.current_tick,
            queue: &mut self.queue,
            id_counter: &mut self.id_counter,
        };

        scheduler.schedule(event, delay);
    }

    pub fn step(&mut self, world: &mut W) {
        self.current_tick += 1;

        let mut executions: u64 = 0;

        loop {
            if executions >= self.max_executions_per_tick {
                return;
            }

            let (item, Reverse((time, _))) = match self.queue.pop() {
                Some(entry) => entry,
                None => return, // queue is empty
            };

            if time > self.current_tick {
                self.queue.push(item, Reverse((time, time)));
                return;
            }

            let mut scheduler = Scheduler {
                current_tick: self.current_tick,
                queue: &mut self.queue,
                id_counter: &mut self.id_counter,
            };

            item.event.execute(world, self.current_tick, &mut scheduler);
            executions += 1;
            self.total_events_executed += 1;
        }
    }

    pub fn step_until(&mut self, target_tick: u64, world: &mut W) {
        while self.current_tick < target_tick {
            self.step(world);
        }
    }

    pub fn get_queue_size(&self) -> usize {
        self.queue.len()
    }

    pub fn get_total_events_executed(&self) -> u64 {
        self.total_events_executed
    }

    pub fn get_current_tick(&self) -> u64 {
        self.current_tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWorld {
        gold: i32,
        logs: Vec<String>,
    }

    struct Miner {
        amount: i32,
    }

    impl Event<TestWorld> for Miner {
        fn execute(&self, world: &mut TestWorld, tick: u64, scheduler: &mut Scheduler<TestWorld>) {
            world.gold += self.amount;
            world
                .logs
                .push(format!("Tick {}: Mined {}", tick, self.amount));

            // recur every 5 ticks
            let next_miner = Box::new(Miner {
                amount: self.amount,
            });
            scheduler.schedule(next_miner, 5);
        }
    }

    struct Explosion {
        power: i32,
    }

    impl Event<TestWorld> for Explosion {
        fn execute(&self, world: &mut TestWorld, tick: u64, _scheduler: &mut Scheduler<TestWorld>) {
            world
                .logs
                .push(format!("Tick {}: BOOM {}", tick, self.power));
        }
    }

    #[test]
    fn test_simulation_workflow() {
        let mut world = TestWorld {
            gold: 0,
            logs: vec![],
        };

        let mut engine = Engine::build()
            .max_executions_per_tick(100)
            .initial_event_pool(vec![
                (
                    Box::new(Miner { amount: 10 }) as Box<dyn Event<TestWorld>>,
                    1,
                ),
                (
                    Box::new(Explosion { power: 9000 }) as Box<dyn Event<TestWorld>>,
                    12,
                ),
            ]);

        // Expected Miner Activations: Tick 1, 6, 11, 16. (Total 4 times)
        // Expected Explosion: Tick 12.
        engine.step_until(20, &mut world);

        assert_eq!(world.gold, 40, "Gold should be 40 after 4 mining cycles");

        assert_eq!(world.logs[0], "Tick 1: Mined 10");
        assert_eq!(world.logs[1], "Tick 6: Mined 10");
        assert_eq!(world.logs[2], "Tick 11: Mined 10");
        assert_eq!(world.logs[3], "Tick 12: BOOM 9000");
        assert_eq!(world.logs[4], "Tick 16: Mined 10");

        assert_eq!(engine.current_tick, 20);
    }

    #[test]
    fn test_100k_ticks_with_probabilistic_spawning() {
        use rand::Rng;

        struct CounterWorld {
            event_count: u64,
        }

        struct SpawningEvent;

        impl Event<CounterWorld> for SpawningEvent {
            fn execute(
                &self,
                world: &mut CounterWorld,
                _tick: u64,
                scheduler: &mut Scheduler<CounterWorld>,
            ) {
                world.event_count += 1;

                // 50% chance to spawn 3 more events
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.5) {
                    for _ in 0..3 {
                        scheduler.schedule(Box::new(SpawningEvent), 5);
                    }
                }
            }
        }

        let mut world = CounterWorld { event_count: 0 };
        let mut engine = Engine::build()
            .max_executions_per_tick(1000)
            .initial_event_pool(vec![(Box::new(SpawningEvent), 1)]);

        // run for 100k ticks with progress monitoring
        for tick in 1..=100_000 {
            engine.step(&mut world);

            if tick % 10000 == 0 {
                println!("tick {}: event_count = {}", tick, world.event_count);
            }
        }

        assert_eq!(engine.current_tick, 100_000);
        assert!(
            world.event_count > 0,
            "at least one event should have executed"
        );
    }
}
