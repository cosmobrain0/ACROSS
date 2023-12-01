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

/// Draws an approximation of a sector
/// given the number of triangles to use
/// 100 is normally a good number of triangles
pub fn draw_sector(
    ctx: &mut Context,
    position: Vector,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    triangle_count: usize,
    colour: Color,
) {
    let step_size = (end_angle - start_angle) / triangle_count as f32;
    let triangles: Vec<_> = (0..triangle_count)
        .map(|i| i as f32 / triangle_count as f32 * (end_angle - start_angle))
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

/// Draws a line between two points, given a thickness
pub fn draw_line(ctx: &mut Context, a: Vector, b: Vector, thickness: f32, colour: Color) {
    let mesh = MeshBuilder::new()
        .line(&[[a.x, a.y], [b.x, b.y], [a.x, a.y]], thickness, colour)
        .unwrap()
        .build(ctx)
        .unwrap();
    graphics::draw(ctx, &mesh, DrawParam::from(([0.0, 0.0], Color::WHITE))).unwrap();
}

/// Draws a progress bar
pub fn draw_progress_bar(
    ctx: &mut Context,
    top_left: Vector,
    size: Vector,
    progress: f32,
    back_colour: Color,
    colour: Color,
    padding: f32,
) {
    draw_rectangle(ctx, top_left, size, back_colour);
    let total_inner_width = size.x - padding * 2.0;
    let inner_height = size.y - padding * 2.0;
    let inner_top_left = top_left + vec2d![padding, padding];
    let inner_width = total_inner_width * progress.clamp(0.0, 1.0);
    draw_rectangle(
        ctx,
        inner_top_left,
        vec2d![inner_width, inner_height],
        colour,
    );
}
