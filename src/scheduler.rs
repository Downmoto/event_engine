use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use crate::scheduled_wrapper::ScheduledEvent;
use crate::Event;

pub struct Scheduler<'a, W> {
    pub current_tick: u64,
    pub queue: &'a mut PriorityQueue<ScheduledEvent<W>, Reverse<(u64, u64)>>,
    pub id_counter: &'a mut u64,
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