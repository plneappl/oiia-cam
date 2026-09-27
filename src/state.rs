use std::sync::Arc;

use crate::{
    object::Object,
    resources::{Image, Resources},
    texture::{Texture, Textures},
    util::{Frame, Size, Vec2d},
};

pub struct State<'a> {
    pub size: Size,
    pub microphone_input_detected: bool,
    pub cat_pos: Vec2d,
    pub cat_img: &'a Image,
}

impl<'a> State<'a> {
    pub fn new(resources: &'a Resources, size: Size) -> State<'a> {
        State {
            size: size,
            microphone_input_detected: false,
            cat_pos: Vec2d { x: 400, y: 400 },
            cat_img: &resources.standing,
        }
    }

    pub fn render_gpu<'b>(&self, textures: &'b Textures<'b>) -> Vec<Object<'b>> {
        let tex: &Texture<'b> = textures
            .all_textures()
            .iter()
            .find(|it| it.image == self.cat_img)
            .expect("img never uploaded");
        vec![Object {
            position: self.cat_pos,
            texture: tex,
        }]
    }
}
