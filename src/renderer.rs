use ggez::{
    graphics::{self, Color, DrawMode, MeshBuilder, Rect},
    Context,
};

use crate::vector::Vector;

pub fn draw_rectangle(ctx: &mut Context, position: Vector, size: Vector, colour: Color) {
    let mesh = MeshBuilder::new()
        .rectangle(
            DrawMode::fill(),
            Rect::new(0.0, 0.0, size.x, size.y),
            Color::WHITE,
        )
        .unwrap()
        .build(ctx)
        .unwrap();
    let position: [f32; 2] = position.into();
    graphics::draw(ctx, &mesh, (position, colour)).unwrap();
}

pub fn draw_circle(ctx: &mut Context, position: Vector, radius: f32, colour: Color) {
    let mesh = MeshBuilder::new()
        .circle(DrawMode::fill(), [0.0, 0.0], radius, 0.2, Color::WHITE)
        .unwrap()
        .build(ctx)
        .unwrap();
    let position: [f32; 2] = position.into();
    graphics::draw(ctx, &mesh, (position, colour)).unwrap();
}
