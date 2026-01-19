use crate::event::Event;
use crate::scheduled_wrapper::ScheduledEvent;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::HashSet;

pub struct Scheduler<'a, W> {
    current_tick: u64,
    queue: &'a mut PriorityQueue<ScheduledEvent<W>, Reverse<(u64, u64)>>,
    id_counter: &'a mut u64,
}

impl<'a, W> Scheduler<'a, W> {
    pub fn schedule(&mut self, event: Box<dyn Event<W>>, delay: u64) -> u64 {
        *self.id_counter += 1;
        let id = *self.id_counter;

        let item = ScheduledEvent { id, event };
        let priority = Reverse((self.current_tick + delay, id));

        self.queue.push(item, priority);
        id
    }
}

pub struct Engine<W> {
    current_tick: u64,
    // The Priority is Reverse((Time, ID)).
    queue: PriorityQueue<ScheduledEvent<W>, Reverse<(u64, u64)>>,
    id_counter: u64,

    cancelled_events: HashSet<u64>,
}

impl<W> Engine<W> {
    pub fn new() -> Self {
        Self {
            current_tick: 0,
            queue: PriorityQueue::new(),
            id_counter: 0,
            cancelled_events: HashSet::new(),
        }
    }

    pub fn schedule(&mut self, event: Box<dyn Event<W>>, delay: u64) -> u64 {
        let mut scheduler = Scheduler {
                current_tick: self.current_tick,
                queue: &mut self.queue,
                id_counter: &mut self.id_counter,
            };

        let id: u64 = scheduler.schedule(event, delay);
        id
    }

    pub fn cancel(&mut self, event_id: u64) {
        self.cancelled_events.insert(event_id);
    }

    pub fn step(&mut self, world: &mut W) {
        self.current_tick += 1;

        loop {
            match self.queue.peek() {
                Some((_, Reverse((time, _)))) => {
                    if *time > self.current_tick {
                        return; // next event is in the future
                    }
                }
                None => return, // queue is empty
            }

            let (item, _) = self.queue.pop().unwrap();

            if self.cancelled_events.contains(&item.id) {
                self.cancelled_events.remove(&item.id);
                continue; 
            }

            let mut scheduler = Scheduler {
                current_tick: self.current_tick,
                queue: &mut self.queue,
                id_counter: &mut self.id_counter,
            };

            item.event.execute(world, self.current_tick, &mut scheduler);
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
            scheduler: &mut Scheduler<TestWorld>,
        ) {
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
        fn execute(
            &self,
            world: &mut TestWorld,
            tick: u64,
            _scheduler: &mut Scheduler<TestWorld>,
        ) {
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
