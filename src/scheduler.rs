use crate::keys::Key;

use std::cell::Cell;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct EventEntry {
    time: Reverse<Duration>,
    component: Key,
}

impl EventEntry {
    pub(crate) fn new(time: Duration, component: Key) -> Self {
        Self {
            time: Reverse(time),
            component,
        }
    }
    pub(crate) fn key(&self) -> Key {
        self.component
    }
}

impl PartialEq for EventEntry {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for EventEntry {}

impl PartialOrd for EventEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Ord for EventEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

type Clock = Rc<Cell<Duration>>;

pub struct ClockRef {
    clock: Clock,
}

impl From<Clock> for ClockRef {
    fn from(clock: Clock) -> Self {
        Self { clock }
    }
}

impl ClockRef {
    /// Return the current simulation time.
    #[must_use]
    pub fn time(&self) -> Duration {
        self.clock.get()
    }
}

#[derive(Debug)]
pub struct Scheduler {
    events: BinaryHeap<EventEntry>,
    clock: Clock,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            events: BinaryHeap::default(),
            clock: Rc::new(Cell::new(Duration::ZERO)),
        }
    }
}

impl Scheduler {
    /// Schedules `event` to be executed for `component` at `self.time() + time`.
    ///
    /// `component` is a [`Key`](crate::key::Key) corresponding to the [Component](crate::component::Component) to be scheduled.
    /// `resume_with` is a [`StateKey`](crate::key::StateKey) used access the list of permited components to be Activated by the `component`
    pub fn schedule(&mut self, time: Duration, component: Key) {
        let time = self.time() + time;
        let event = EventEntry::new(time, component);
        self.events.push(event);
    }

    /// Schedules `event` to be executed for `component` at `self.time()`.
    ///
    /// `component` is a [`Key`](crate::key::Key) corresponding to the [Component](crate::component::Component) to be scheduled.
    /// `resume_with` is a [`StateKey`](crate::key::StateKey) used access the list of permited components to be Activated by the `component`
    pub fn schedule_now(&mut self, component: Key) {
        self.schedule(Duration::ZERO, component);
    }

    /// Returns the current simulation time.
    #[must_use]
    pub fn time(&self) -> Duration {
        self.clock.get()
    }

    /// Returns a structure with immutable access to the simulation time.
    #[must_use]
    pub fn clock(&self) -> ClockRef {
        ClockRef {
            clock: Rc::clone(&self.clock),
        }
    }

    /// Removes and returns the next scheduled event or `None` if none are left.
    pub fn pop(&mut self) -> Option<EventEntry> {
        self.events.pop().map(|event| {
            self.clock.replace(event.time.0);
            event
        })
    }

    // Utility function used to give each EventEntry an unique id
    // to break of ties based on the orden of insertion
    // the earliest to be inserted is the first to get out
    // if both EventEntry has the same time.
    // fn get_new_id(&mut self) -> Reverse<u128> {
    //     self.next_id += 1;
    //     Reverse(self.next_id)
    // }

    // Private function to insert `EventEntry` for testing.
    // Not used in public API
    #[allow(dead_code)]
    fn insert(&mut self, event: EventEntry) {
        // let next = self.get_new_id();
        self.events.push(event);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clock_ref_update() {
        let time = Duration::from_secs(1);
        let clock = Clock::new(Cell::new(time));
        let clock_ref = ClockRef::from(clock.clone());
        assert_eq!(clock_ref.time(), time);
        let time = time + Duration::from_secs(5);
        clock.set(time);
        assert_eq!(clock_ref.time(), time);
    }

    // #[test]
    // fn test_event_entry_debug() {
    //     let entry = EventEntry {
    //         time: Reverse(Duration::from_secs(1)),
    //         component: Key::new_unchecked(2),
    //     };
    //     assert_eq!(
    //         &format!("{:?}", entry),
    //         "EventEntry { time: Reverse(1s), component: Key { id: 2 } }"
    //     );
    // }

    #[test]
    fn event_entry_cmp() {
        let make_entry = || -> EventEntry {
            EventEntry {
                time: Reverse(Duration::from_secs(1)),
                component: Key::new(2),
            }
        };
        assert_eq!(
            EventEntry {
                time: Reverse(Duration::from_secs(1)),
                ..make_entry()
            },
            EventEntry {
                time: Reverse(Duration::from_secs(1)),
                ..make_entry()
            }
        );
        assert_eq!(
            EventEntry {
                time: Reverse(Duration::from_secs(0)),
                ..make_entry()
            }
            .cmp(&EventEntry {
                time: Reverse(Duration::from_secs(1)),
                ..make_entry()
            }),
            Ordering::Greater
        );
        assert_eq!(
            EventEntry {
                time: Reverse(Duration::from_secs(2)),
                ..make_entry()
            }
            .cmp(&EventEntry {
                time: Reverse(Duration::from_secs(1)),
                ..make_entry()
            }),
            Ordering::Less
        );
    }

    #[test]
    fn scheduler_and_event_entry() {
        let mut scheduler = Scheduler::default();
        let mut key_id = 1;
        let mut make_event_entry = |x: u64, time: Duration| -> EventEntry {
            key_id += 1;
            EventEntry {
                time: Reverse(Duration::from_secs(x) + time),
                component: Key::new(key_id),
            }
        };
        let event_1 = make_event_entry(1, scheduler.time()); // Output order:
        let event_2 = make_event_entry(8, scheduler.time()); // event_1 -> event_3 -> event_2;
        let event_3 = make_event_entry(4, scheduler.time()); // Simulation Time after executing these 3 events: 8 sec.

        let (c_event_1, c_event_2, c_event_3) = (event_1.clone(), event_2.clone(), event_3.clone());
        scheduler.insert(event_1);
        scheduler.insert(event_2);
        scheduler.insert(event_3);

        assert_eq!(Duration::ZERO, scheduler.time()); // Assert that inserting events will not advance the simulation time.

        let r_event = scheduler.pop(); // Extract the event closer to the actual simulation time.
        assert_eq!(Some(c_event_1), r_event); // Assert that the extracted event is event_1.
        assert_eq!(Duration::from_secs(1), scheduler.time()); // The simulation time advance to when the event was scheduled.
                                                              //
        let r_event = scheduler.pop(); // Do the same for the other events.
        assert_eq!(Some(c_event_3), r_event);
        assert_eq!(Duration::from_secs(4), scheduler.time());

        let r_event = scheduler.pop();
        assert_eq!(Duration::from_secs(8), scheduler.time());
        assert_eq!(Some(c_event_2), r_event);

        let r_event = scheduler.pop();
        assert_eq!(None, r_event); // All events were extracted no more events remains in the Scheduler.
        assert_eq!(Duration::from_secs(8), scheduler.time()); // Actual Simulation Time: 8 sec.

        let event_4 = make_event_entry(10, scheduler.time()); // Schedule in Simulation Time + 10 sec.
        let event_5 = make_event_entry(2, scheduler.time()); // Schedule in Simulation Time + 2 seg.
        let (c_event_4, c_event_5) = (event_4.clone(), event_5.clone());

        scheduler.insert(event_4); // Output order: event_5 -> event_4
        scheduler.insert(event_5); // Simulation Time after extracting these 2 events: 18 sec.
                                   //
        let r_event = scheduler.pop(); // Extract the inserted events
        assert_eq!(Some(c_event_5), r_event); // The closer one is extracted first no mather if it was inserted later.
        assert_eq!(Duration::from_secs(10), scheduler.time()); // The simulation time is replaced by Simulation Time + Event Time
                                                               // i.e Simulation Time = 8 secs + 2 secs;
        let r_event = scheduler.pop();
        assert_eq!(Some(c_event_4), r_event);
        assert_eq!(Duration::from_secs(18), scheduler.time());
    }
}
