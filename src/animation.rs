use crate::state::State;

pub trait Animation {
    fn next_state<'a>(&mut self, state: State<'a>) -> State<'a>;
}
