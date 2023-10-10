use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, MeshBuilder, Rect, Text, TextFragment},
    Context,
};

use crate::{vec2d, vector::Vector};

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

pub fn draw_sector(
    ctx: &mut Context,
    position: Vector,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    triangle_count: usize,
    colour: Color,
) {
    // TODO: I have to make my own arc???
    let step_size = (end_angle - start_angle) / triangle_count as f32;
    let triangles: Vec<_> = (0..triangle_count)
        .map(|i| i as f32 * (end_angle - start_angle))
        .map(|theta| theta + start_angle)
        .map(|theta| {
            [
                vec2d![0.0, 0.0],
                Vector::from_polar(theta, radius),
                Vector::from_polar(theta + step_size, radius),
            ]
        })
        .flatten()
        .collect();
    let mesh = MeshBuilder::new()
        .triangles(triangles.as_slice(), Color::WHITE)
        .unwrap()
        .build(ctx)
        .unwrap();
    let position: [f32; 2] = position.into();
    graphics::draw(ctx, &mesh, (position, colour)).unwrap();
}
/// Draw text, given its top-left corner's position, the font size and the bounds.
/// The default size is 32px
/// The default bounds are infinity (no bounds).
pub fn draw_text(
    ctx: &mut Context,
    text: &str,
    position: Vector,
    size: Option<f32>,
    bounds: Option<(Vector, graphics::Align)>,
    colour: Color,
) {
    let size = size.unwrap_or(32.0);
    let mut text = Text::new(TextFragment::new(text).scale(size));
    if let Some((b, align)) = bounds {
        let bounds: [f32; 2] = b.into();
        text.set_bounds(bounds, align);
    }
    let position: [f32; 2] = position.into();
    graphics::draw(ctx, &text, DrawParam::from((position, colour))).unwrap();
}

pub fn draw_line(ctx: &mut Context, a: Vector, b: Vector, thickness: f32, colour: Color) {
    let mesh = MeshBuilder::new()
        .line(&[[a.x, a.y], [b.x, b.y]], thickness, colour)
        .unwrap()
        .build(ctx)
        .unwrap();
    graphics::draw(ctx, &mesh, DrawParam::from(([0.0, 0.0], Color::WHITE))).unwrap();
}

#[allow(dead_code)]
pub fn draw_joined_lines(ctx: &mut Context, points: Vec<Vector>, thickness: f32, colour: Color) {
    let mesh = MeshBuilder::new()
        .line(
            points
                .iter()
                .map(|p| [p.x, p.y])
                .collect::<Vec<_>>()
                .as_slice(),
            thickness,
            colour,
        )
        .unwrap()
        .build(ctx)
        .unwrap();
    graphics::draw(ctx, &mesh, DrawParam::from(([0.0, 0.0], Color::WHITE))).unwrap();
}
