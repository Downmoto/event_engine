use crate::event::Event;
use crate::scheduled_wrapper::ScheduledEvent;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;

pub struct Engine<W> {
    current_tick: u64,
    // The Priority is Reverse((Time, ID)).
    queue: PriorityQueue<ScheduledEvent<W>, Reverse<(u64, u64)>>,
    id_counter: u64,
}

impl<W> Engine<W> {
    pub fn new() -> Self {
        Self {
            current_tick: 0,
            queue: PriorityQueue::new(),
            id_counter: 0,
        }
    }

    pub fn schedule(&mut self, event: Box<dyn Event<W>>, delay: u64) {
        self.id_counter += 1;
        let execute_at = self.current_tick + delay;

        let wrapper = ScheduledEvent {
            id: self.id_counter,
            event,
        };

        // We push the Item and the Priority separately.
        // Priority = (Time, ID). We reverse it so smaller numbers pop first.
        self.queue
            .push(wrapper, Reverse((execute_at, self.id_counter)));
    }

    pub fn step(&mut self, world: &mut W) {
        self.current_tick += 1;

        loop {
            match self.queue.peek() {
                Some((_, Reverse((time, _)))) => {
                    if *time > self.current_tick {
                        return; // Next event is in the future
                    }
                }
                None => return, // Queue is empty
            }

            // pop() returns Option<(Item, Priority)>
            let (item, _) = self.queue.pop().unwrap();

            let new_events = item.event.execute(world, self.current_tick);

            // Schedule new events
            for (evt, delay) in new_events {
                self.schedule(evt, delay);
            }
        }
    }

    pub fn step_until(&mut self, target_tick: u64, world: &mut W) {
        while self.current_tick < target_tick {
            self.step(world);
        }
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
        fn execute(
            &self,
            world: &mut TestWorld,
            tick: u64,
        ) -> Vec<(Box<dyn Event<TestWorld>>, u64)> {
            world.gold += self.amount;
            world
                .logs
                .push(format!("Tick {}: Mined {}", tick, self.amount));

            // Recur every 5 ticks
            let next_miner = Box::new(Miner {
                amount: self.amount,
            });
            vec![(next_miner, 5)]
        }
    }

    struct Explosion {
        power: i32,
    }

    impl Event<TestWorld> for Explosion {
        fn execute(
            &self,
            world: &mut TestWorld,
            tick: u64,
        ) -> Vec<(Box<dyn Event<TestWorld>>, u64)> {
            world
                .logs
                .push(format!("Tick {}: BOOM {}", tick, self.power));
            vec![]
        }
    }

    #[test]
    fn test_simulation_workflow() {
        let mut world = TestWorld {
            gold: 0,
            logs: vec![],
        };
        let mut engine = Engine::new();

        // Miner starts at Tick 1, recurs every 5 ticks
        engine.schedule(Box::new(Miner { amount: 10 }), 1);

        // Explosion happens at Tick 12
        engine.schedule(Box::new(Explosion { power: 9000 }), 12);

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
}
