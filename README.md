# event_engine

a simple event scheduling engine for rust

## overview

event_engine is a tick-based event scheduling system that allows you to schedule and execute events over time. it uses a priority queue to efficiently manage events and execute them in order.

## features

- tick-based event execution
- priority queue for efficient event scheduling
- configurable execution rate per tick
- event rescheduling capabilities

## usage

```rust
use event_engine::{Engine, Event, Scheduler};

// define your world state
struct World {
    // your game/simulation state
}

// implement the Event trait
struct MyEvent;

impl Event<World> for MyEvent {
    fn execute(&self, world: &mut World, current_tick: u64, scheduler: &mut Scheduler<World>) {
        // your event logic here
        // optionally schedule more events with scheduler.schedule()
    }
}

// create and run the engine
let mut engine = Engine::<World>::build()
    .max_executions_per_tick(10)
    .initial_event_pool(vec![
        (Box::new(MyEvent), 5), // schedule event for tick 5
    ]);

let mut world = World { /* ... */ };

// or schedule events later
engine.schedule(Box::new(MyEvent), 10); // execute after 10 ticks

// step through time
engine.step(&mut world); // advance one tick
engine.step_until(100, &mut world); // advance to tick 100
```

## installation

add to your `Cargo.toml`:

```toml
[dependencies]
event_engine = { git = "https://github.com/Downmoto/event_engine" }
```

## license

MIT

## Author

Arad Fadaei
