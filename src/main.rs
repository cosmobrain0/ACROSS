mod renderer;
mod ui;
mod vector;

use std::cell::RefCell;
use std::rc::Rc;

use ggez::event;
use ggez::graphics::{self, get_window_color_format, Color, Rect};
use ggez::input::mouse;
use ggez::{Context, GameResult};

use ui::{Button, Menu};
use vector::*;

pub struct GameState {}

impl GameState {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct MainState {
    canvas: graphics::Canvas,
    menu: Rc<RefCell<Menu<'static, GameState>>>,
    state: GameState,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let menu = Rc::new(RefCell::new(Menu::new(vec2d!(0.0, 0.0), 1.0, None)));
        let buttons = vec![
            Button::new(vec2d!(0.0, 0.0), vec2d!(75.0, 20.0), menu.clone(), |_| {
                println!("Hi")
            })
            .into(),
            Button::new(
                vec2d!(75.0, 75.0),
                vec2d!(100.0, 60.0),
                menu.clone(),
                |_| println!("Hello"),
            )
            .into(),
        ];
        menu.borrow_mut().add_elements(buttons);
        let s = MainState {
            canvas: graphics::Canvas::new(
                ctx,
                1920,
                1080,
                ggez::conf::NumSamples::One,
                get_window_color_format(ctx),
            )
            .unwrap(),
            menu,
            state: GameState::new(),
        };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.menu.borrow_mut().set_position(vec2d!(20.0, 50.0));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::set_canvas(ctx, Some(&self.canvas));
        graphics::clear(ctx, graphics::Color::from((255, 255, 255, 255)));
        graphics::set_screen_coordinates(ctx, Rect::new(0.0, 0.0, 1920.0, 1080.0)).unwrap();

        self.menu.borrow().draw(ctx);

        graphics::set_canvas(ctx, None);
        let (window_width, window_height) = graphics::size(ctx);
        graphics::draw(
            ctx,
            &self.canvas,
            graphics::DrawParam::new()
                .color(Color::from((255, 255, 255, 255)))
                .scale([1920.0 / window_width, 1080.0 / window_height]),
        )?;

        graphics::present(ctx)?;

        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let mouse_position = vec2d!(x, y);
        match button {
            event::MouseButton::Left => {
                self.menu.borrow().input_at(mouse_position, &mut self.state)
            }
            _ => (),
        }
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("hello_canvas", "ggez");
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
