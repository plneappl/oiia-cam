use crate::animation::Animation;
use crate::state::State;
use crate::util::Vec2d;

pub struct Bounce {
    pub x_dir: i32,
    pub y_dir: i32,
}

impl Animation for Bounce {
    fn next_state<'a>(&mut self, state: State<'a>) -> State<'a> {
        let new_pos = state.cat_pos.plus(&Vec2d {
            x: self.x_dir,
            y: self.y_dir,
        });
        let touches_left = new_pos.x <= 0;
        let touches_right =
            usize::try_from(new_pos.x).unwrap_or(0) + state.cat_img.size.w > state.size.w;
        let touches_top = new_pos.y <= 0;
        let touches_bottom =
            usize::try_from(new_pos.y).unwrap_or(0) + state.cat_img.size.h > state.size.h;

        if touches_left || touches_right {
            self.x_dir = -1 * self.x_dir
        }
        if touches_top || touches_bottom {
            self.y_dir = -1 * self.y_dir
        }
        if touches_left || touches_right || touches_top || touches_bottom {
            return self.next_state(state);
        }

        State {
            cat_pos: new_pos,
            ..state
        }
    }
}
