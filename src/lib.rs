#![feature(generators, generator_trait)]
// use std::cell::Cell;

mod container;
mod either;
mod keys;
mod scheduler;
mod simulation;

use std::{ops::Generator, time::Duration};

pub use either::Either;
pub use keys::Key;
pub use simulation::Simulation;

pub type GenBoxed<R> = Box<dyn Generator<R, Yield = Action, Return = ()> + Unpin>;

// Action Define que acción realiza la simulación
// Este enum es devuelto tras ejecutar un step de los generadores
#[derive(Debug, Clone)]
pub enum Action {
    Hold(Duration),
    Passivate,
    Activate(Either<Key, Vec<Key>>),
}

use either::Either::*;
impl Action {
    pub fn activate_one(key: Key) -> Self {
        Action::Activate(Left(key))
    }

    pub fn activate_many(keys: Vec<Key>) -> Self {
        Action::Activate(Right(keys))
    }
}

// thread_local! {
//     static ID_COUNTER: Cell<usize> = Cell::new(0);
// }

// // #[tracing::instrument]
// fn generate_next_id() -> usize {
//     // use tracing::trace;
//     // let id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
//     let id = ID_COUNTER.with(|cell| {
//         let id = cell.get();
//         cell.set(id + 1);
//         id
//     });
//     // trace!("Generating new Id = {}", id);
//     id
// }
