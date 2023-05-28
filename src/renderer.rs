use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, MeshBuilder, Rect, Text, TextFragment},
    Context,
};

use crate::vector::Vector;

/// Draw a rectangle, given its top-left corner and its width and height.
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

/// Draw a circle, given its centre position and its radius.
pub fn draw_circle(ctx: &mut Context, position: Vector, radius: f32, colour: Color) {
    let mesh = MeshBuilder::new()
        .circle(DrawMode::fill(), [0.0, 0.0], radius, 0.2, Color::WHITE)
        .unwrap()
        .build(ctx)
        .unwrap();
    let position: [f32; 2] = position.into();
    graphics::draw(ctx, &mesh, (position, colour)).unwrap();
}

/// Draw text, given its top-left corner's position, the font size and the bounds.
/// The default size is 16px
/// The default bounds are infinity (no bounds).
pub fn draw_text(
    ctx: &mut Context,
    text: &str,
    position: Vector,
    size: Option<f32>,
    bounds: Option<Vector>,
    colour: Color,
) {
    let size = match size {
        Some(x) => x,
        None => 16.0,
    };
    let mut text = Text::new(TextFragment::new(text).scale(size));
    match bounds {
        Some(x) => {
            let bounds: [f32; 2] = x.into();
            text.set_bounds(bounds, graphics::Align::Left);
        }
        None => (),
    }
    let position: [f32; 2] = position.into();
    graphics::draw(ctx, &text, DrawParam::from((position, colour))).unwrap();
}
