mod renderer;
mod ui;
mod vector;

use ggez::event;
use ggez::graphics::{self, Color};
use ggez::input::mouse;
use ggez::{Context, GameResult};
use renderer::{draw_circle, draw_rectangle, draw_text};
use vector::*;

struct MainState {
    canvas: graphics::Canvas,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState {
            canvas: graphics::Canvas::with_window_size(ctx).unwrap(),
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

        draw_text(
            ctx,
            "Hi there!",
            vec2d!(0.0, 0.0),
            None,
            None,
            Color::MAGENTA,
        );
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
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("hello_canvas", "ggez");
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
