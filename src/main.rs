mod renderer;
mod vector;

use ggez::event;
use ggez::graphics::{self, Color};
use ggez::input::mouse;
use ggez::{Context, GameResult};
use renderer::{draw_circle, draw_rectangle};
use vector::*;

struct MainState {
    canvas: graphics::Canvas,
    mouse: [f32; 2],
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState {
            canvas: graphics::Canvas::with_window_size(ctx).unwrap(),
            mouse: [0.0; 2],
        };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::set_canvas(ctx, Some(&self.canvas));
        graphics::clear(ctx, graphics::Color::from((255, 255, 255, 255)));

        draw_rectangle(
            ctx,
            mouse::position(ctx).into(),
            vec2d!(40.0, 50.0),
            Color::RED,
        );
        draw_circle(ctx, mouse::position(ctx).into(), 40.0, Color::BLACK);
        graphics::set_canvas(ctx, None);

        graphics::draw(
            ctx,
            &self.canvas,
            graphics::DrawParam::new().color(Color::from((255, 255, 255, 255))),
        )?;

        graphics::present(ctx)?;

        Ok(())
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.mouse = [x, y];
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("hello_canvas", "ggez");
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
