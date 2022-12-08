use crate::GenBoxed;
use crate::{/* component::Component , */ keys::Key, Action};
// use std::future::Future;
use std::ops::GeneratorState;
use std::pin::Pin;

// use genawaiter::{rc::Gen, GeneratorState};

// pub fn make_dyn<F, C>(future: F) -> Pin<Box<dyn Future<Output = C>>>
// where
//     F: Future<Output = C> + 'static,
// {
//     Box::pin(future)
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentState {
    Passivated,
    Active,
}

// pub type BoxedComponent<R> = Box<dyn Component<R>>;

pub struct Container<R> {
    // pub(crate) inner: Vec<Option<(BoxedComponent<R>, ComponentState)>>,
    pub(crate) inner: Vec<Option<(GenBoxed<R>, ComponentState)>>,
}

impl<R> Default for Container<R>
where
    R: 'static,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<R> Container<R>
where
    R: 'static,
{
    pub fn add_generator(&mut self, gen: GenBoxed<R>) -> Key {
        let key = Key::new(self.inner.len());
        // let gen: BoxedComponent<R> = Box::new(gen);
        self.inner.push(Some((gen, ComponentState::Active)));
        key
    }

    // pub fn add_generator_fn<G>(&mut self, producer: G) -> Key
    // where
    //     G: Generator<Option<StateKey<R>>, Yield = Action, Return = ()> + 'static,
    //     G: Unpin
    // {
    //     let key = Key::new(self.inner.len());

    //     // Without make_dyn type inference can't seems to get that it should return a dyn Future instead of F.
    //     // let gen: GenRcBoxed<R> = Gen::new(|co| make_dyn(producer(co.into())));

    //     let gen = Box::new(producer);
    //     self.inner.push(Some((gen, ComponentState::Active)));
    //     key
    // }

    // pub fn run_to_completion(&mut self, key: usize, resume_with: R) {
    //     let gen = self.inner.get_mut(&key).unwrap();
    //     while let genawaiter::GeneratorState::Yielded(_) = gen.resume_with(resume_with) {}
    // }

    #[allow(dead_code)]
    pub fn remove(&mut self, key: Key) -> Option<(GenBoxed<R>, ComponentState)> {
        if self.inner.get(key.id).is_some() {
            self.inner[key.id].take()
        } else {
            None
        }
        // Another way of doing the above added in rust 1.62
        // self.inner.get(key.id).is_some().then_some(self.inner[key.id].take()).flatten()
    }

    /// Returns the number of elements in the container.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the container contains no elements.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Advance the component defined by `key`
    ///
    /// # Panics
    ///
    /// Panics when the key used was for an already extracted generator
    /// or if the generator has already completed its execution.
    pub fn step_with(&mut self, key: Key, resume_with: R) -> GeneratorState<Action, ()> {
        // Esto asume que los eventos nunca son borrados.
        // TODO: Confirmar esta asumpción.

        let &mut (ref mut gen, _) = self
            .inner
            .get_mut(key.id)
            .map(Option::as_mut)
            .flatten()
            .expect("components shouldn't be removed from the container");

        // gen.step(resume_with)
        let gen = gen.as_mut();
        Pin::new(gen).resume(resume_with)
        // gen.resume_with(resume_with)
    }

    #[must_use]
    pub fn get_state(&self, key: Key) -> Option<&ComponentState> {
        // if let Some(values) = self.inner.get(key.id) {
        //     values.as_ref().map(|(_, ref state)| state)
        // } else {
        //     None
        // }

        self.inner
            .get(key.id)
            .map(Option::as_ref)
            .flatten()
            .map(|&(_, ref state)| state)
    }

    #[must_use]
    pub fn get_state_mut(&mut self, key: Key) -> Option<&mut ComponentState> {
        // if let Some(value) = self.inner.get_mut(key.id) {
        //     value.as_mut().map(|&mut (_, ref mut state)| state)
        // } else {
        //     None
        // }

        self.inner
            .get_mut(key.id)
            .map(Option::as_mut)
            .flatten()
            .map(|&mut (_, ref mut state)| state)
    }
}

impl Container<()> {
    #[allow(dead_code)]
    pub fn step(&mut self, key: Key) -> GeneratorState<Action, ()> {
        self.step_with(key, ())
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;

    fn producer(kind: &'static str) -> GenBoxed<()> {
        let gen = move |_| {
            println!("Iniciando {}", kind);
            // TODO: FIX THIS FUNCION. ESPECIFICAMENTE EL TIPO DE YIELD
            yield Action::Passivate;
            for i in 0..3 {
                println!(
                    "{} ha sido llamado {} {}",
                    kind,
                    i + 1,
                    if i == 0 { "vez" } else { "veces" }
                );
                yield Action::Passivate;
            }
            println!("{} Finaliza", kind);
        };
        Box::new(gen)
    }

    fn finite(name: &'static str, flag: bool, number_of_loops: u8) -> GenBoxed<()> {
        let gen = move |_| {
            if flag {
                println!(
                    "{} is starting... yielding in loop {} times",
                    name, number_of_loops
                );
            }
            for i in 0..number_of_loops {
                println!("Yield");
                let _ = yield Action::Hold(Duration::ZERO);
                // co.hold(Duration::ZERO).await
                println!("{} has yielded {} times", name, i + 1);
            }
            println!("{} completed", name);
        };
        Box::new(gen)
    }

    fn infinite(indentifier: usize) -> GenBoxed<()> {
        let gen = move |_| {
            println!("This function is starting and will never complete");
            let mut i = 1;
            loop {
                println!(
                    "Infinite Generator N°{} is Yielding | It has Yielded {} times",
                    indentifier, i
                );
                let _ = yield Action::Hold(Duration::ZERO);
                // co.hold(Duration::ZERO).await;
                i += 1;
            }
        };
        Box::new(gen)
    }

    #[test]
    fn generators_can_be_inserted() {
        let mut container = Container::default();

        // Assert that the container is empty
        assert!(container.is_empty());

        // First way of creating and inserting a generator to the container
        let gen = producer("A");
        let first_key = container.add_generator(gen);
        assert_eq!(0, first_key.id()); // Keys ids start at 2 because of implementation reasons.

        // Second way of creating and inserting a generator to the container
        let second_key = container.add_generator(producer("B"));
        assert_eq!(1, second_key.id());

        // A different function can be converted to a generator and inserted to the container
        let gen = finite("A", true, 42);
        let third_key = container.add_generator(gen);
        assert_eq!(2, third_key.id());

        // As long as the Co type parameter stay the same on all functions.
        // In this case is () from Co<()>.
        let fourth_key = container.add_generator(infinite(1));
        assert_eq!(3, fourth_key.id());

        // Assert that all generators were inserted correctly to the container.
        assert_eq!(4, container.len());
    }

    #[test]
    // With the following line we could test if the program fails as expected by doing an incorrect operation.
    // #[should_panic(expected = "`async fn` resumed after completion")]
    fn generators_can_be_resumed() {
        let mut container = Container::default();

        // Using the finite function because if infinite was used in its place this test would never end.
        let finite_key = container.add_generator(finite("A", true, 1));

        // This could be written as:
        //
        // while let GeneratorState::Yielded(_) = container.step(finite_key, None) {}
        //
        // But this makes clearer that the loop will continue until GeneratorState::Complete is recieved.
        loop {
            if let GeneratorState::Complete(_) = container.step_with(finite_key, ()) {
                break;
            }
        }

        // Uncommenting the following line will cause the test to fail.
        // container.step(finite_key, None);
        //
        // This is because when a generator completes, to say, the original function end its excecution
        // The generator cannot be resumed again and it's an error to do so.
    }
}
