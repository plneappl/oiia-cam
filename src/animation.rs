use std::sync::MutexGuard;

use crate::state::State;

pub trait Animation<'a> {
    fn next_state(&mut self, state: &State<'a>) -> State<'a>;
}
