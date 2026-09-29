use std::sync::MutexGuard;

use crate::{object::Object, state::State, util::BoundingBox};

pub trait Animation: Send {
    fn bounding_box(&self) -> BoundingBox;
    fn advance_animation(&mut self, state: &State);
    fn objects(&self, state: &State) -> Vec<Object>;
}
