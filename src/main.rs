mod renderer;
mod ui;
mod vector;

use std::cell::RefCell;

use ggez::event;
use ggez::graphics::{self, Color, Rect};
use ggez::input::mouse;
use ggez::{Context, GameResult};
use renderer::{draw_circle, draw_rectangle, draw_text};
use ui::{Button, Menu};
use vector::*;

struct MainState {
    canvas: graphics::Canvas,
    menu: RefCell<Menu<'static>>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let menu = RefCell::new(Menu::new(vec2d!(0.0, 0.0), 1.0, None));
        let buttons = vec![
            Button::new(vec2d!(0.0, 0.0), vec2d!(75.0, 20.0), menu.clone()).into(),
            Button::new(vec2d!(75.0, 75.0), vec2d!(100.0, 60.0), menu.clone()).into(),
        ];
        menu.borrow_mut().add_elements(buttons);
        let s = MainState {
            canvas: graphics::Canvas::with_window_size(ctx).unwrap(),
            menu,
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
        graphics::set_screen_coordinates(ctx, Rect::new(0.0, 0.0, 1920.0, 1080.0)).unwrap();

        self.menu.borrow().draw(ctx);

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
