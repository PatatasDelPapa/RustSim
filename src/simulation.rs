use std::ops::GeneratorState;
use std::time::Duration;

use crate::container::{Container, ComponentState};
use crate::scheduler::Scheduler;
use crate::{Action, GenBoxed, Key};

pub struct Simulation<R> {
    scheduler: Scheduler,
    components: Container<R>,
}

pub enum ShouldContinue {
    Advance(GeneratorState<Action, ()>, Key),
    Break,
}

impl<R> Default for Simulation<R>
where
    R: 'static,
{
    fn default() -> Self {
        Self {
            scheduler: Scheduler::default(),
            components: Container::default(),
        }
    }
}

impl<R> Simulation<R>
where
    R: 'static,
{
    /// Add an already constructed Generator into the simulation.
    pub fn add_generator(&mut self, gen: GenBoxed<R>) -> Key {
        let key = self.components.add_generator(gen);
        key
    }

    /// Schedules `event` to be executed for `component_key` at `self.time() + time`.
    /// component_key is a key corresponding to the component to be scheduled.
    /// resume_with is a key to access the list of permited components capable of being Activated by this component.
    pub fn schedule(&mut self, time: Duration, component_key: Key) {
        self.scheduler.schedule(time, component_key)
    }

    /// Schedules `component_key` to be executed for at `self.time()`.
    ///
    /// the `component_key` argument is a [`Key`](crate::key::Key) corresponding to the [Component](crate::component::Component) to be scheduled.
    /// `resume_with` is a [`StateKey`](crate::key::StateKey) used access the list of permited components to be Activated by the `component`
    pub fn schedule_now(&mut self, component_key: Key) {
        self.scheduler.schedule_now(component_key)
    }

    /// Advance the simulation 1 event.
    pub fn step_with(&mut self, resume_with: R) -> ShouldContinue {
        if let Some(event_entry) = self.scheduler.pop() {
            let key = event_entry.key();

            /* TODO:
            En lugar de reiniciar la ejecución del componente con StateKey<Vec<(Key, ComponentState)>>
            Revistar el estado actual (Passivated / Activated) de cada key
            Y enviar ese resultado como una lista al generador junto con la llave
            para que pueda hacerle Action::Activate(key) si es necesario
            */

            // if key.is_limit() {
            //     info!("End Simulation Event recieved. Ending Simulation...");
            //     ShouldContinue::Break
            // } else {
            //     trace!("An event from component ID = {}, at simulated time = {:?} was recieved, continuing simulation...", key.id, self.time());
            //     let state = self.components.step(key, resume_with);
            //     ShouldContinue::Advance(state, key)
            // }

            let state = self.components.step_with(key, resume_with);
            ShouldContinue::Advance(state, key)
        } else {
            ShouldContinue::Break
        }
    }

    
    /// Returns the current simulation time.
    #[must_use]
    pub fn time(&self) -> Duration {
        self.scheduler.time()
    }

    #[must_use]
    pub fn clock(&self) -> crate::scheduler::ClockRef {
        self.scheduler.clock()
    }

    /// Retrieve the current state of the component `key`
    ///
    /// This method is used to construct the list of access that a component can access to
    /// by inserting a vec of keys returned by this function and putting the resulting key
    /// into the function [add_access](add_access)
    #[must_use]
    pub fn get_component_state(&self, key: Key) -> Option<(Key, ComponentState)> {
        self.components.get_state(key).map(|&state| (key, state))
    }

    fn run_one_step(&mut self, state: GeneratorState<Action, ()>, key: Key) {
        match state {
            GeneratorState::Yielded(yielded_value) => match yielded_value {
                Action::Hold(duration) => {
                    let component_state = self.components.get_state_mut(key)
                        .expect(&format!("An attempt was made to get the state of a component that does not exist.  Key.id = {}", key.id));
                    
                    if let ComponentState::Passivated = *component_state {
                        panic!("A Passivated component received a hold command. ID = {}", key.id);
                    }

                    self.schedule(duration, key);
                }
                Action::Passivate => {
                    let component_state = self
                            .components
                            .get_state_mut(key)
                            .expect("Se intento conseguir un state de un componente que no existe");
                    match *component_state {
                        ComponentState::Passivated => {
                            panic!("A Passivated component received a passivate command. ID = {}", key.id);
                        },
                        ComponentState::Active => {
                            *component_state = ComponentState::Passivated;
                        },
                    }
                },
                Action::Activate(component) => {
                    use crate::either::Either::*;
                    match component {
                        Left(component) => {
                            let component_state = self.components.get_state_mut(component).expect(&format!("An attempt was made to get the state of a component that does not exist.  Key.id = {}", key.id));
                            match *component_state {
                                ComponentState::Passivated => {
                                    *component_state = ComponentState::Active;
                                },
                                ComponentState::Active => {
                                    panic!(
                                        "An attempt was made to activate an already active component. ID = {}",
                                        component.id
                                    )
                                },
                            }
                            self.schedule_now(key);
                            self.schedule_now(component);
                        },
                        Right(vec_of_components) => {
                            self.schedule_now(key);
                            for component in vec_of_components {
                                let component_state = self.components.get_state_mut(component).expect(&format!("An attempt was made to get the state of a component that does not exist.  Key.id = {}", key.id));
                                match *component_state {
                                    ComponentState::Passivated => {
                                        *component_state = ComponentState::Active;
                                    },
                                    ComponentState::Active => {
                                        panic!(
                                            "An attempt was made to activate an already active component. ID = {}",
                                            component.id
                                        );
                                    },
                                }
                                self.schedule_now(component);
                            }
                        },
                    }
                },
            },
            GeneratorState::Complete(_) => {
                // TODO: Remove the generator from the Vec not shrinking the vec.
            }
        }
    }
}

impl Simulation<()> {
    pub fn step(&mut self) -> ShouldContinue {
        self.step_with(())
    }

    pub fn run(&mut self) {
        while let ShouldContinue::Advance(state, key) = self.step() {
            self.run_one_step(state, key);
        }
    }

    pub fn run_with_limit(&mut self, limit: Duration) {
        while let ShouldContinue::Advance(state, key) = self.step() {
            if self.time() >= limit {
                break;
            }
            self.run_one_step(state, key);
        }
    }
}
