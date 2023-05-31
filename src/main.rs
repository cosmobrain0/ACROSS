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

pub const SCREEN_WIDTH: usize = 1920;
pub const SCREEN_HEIGHT: usize = 1080;

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

pub fn mouse_position(ctx: &mut Context) -> Vector {
    let mouse_position = mouse::position(ctx);
    let window_size = graphics::drawable_size(ctx);
    vec2d!(
        mouse_position.x * SCREEN_WIDTH as f32 / window_size.0,
        mouse_position.y * SCREEN_HEIGHT as f32 / window_size.1
    )
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let menu = Rc::new(RefCell::new(Menu::new(vec2d!(0.0, 0.0), 1.0, None)));
        menu.borrow_mut().set_position(vec2d!(90.0, 350.0));
        menu.borrow_mut().set_scale(2.0);
        let buttons = vec![
            Button::new(
                vec2d!(0.0, 0.0),
                vec2d!(75.0, 20.0),
                menu.clone(),
                |_| println!("Hi"),
                "Hi",
            )
            .into(),
            Button::new(
                vec2d!(75.0, 75.0),
                vec2d!(100.0, 60.0),
                menu.clone(),
                |_| println!("Hello"),
                "Hello",
            )
            .into(),
        ];
        menu.borrow_mut().add_elements(buttons);
        let position = menu.borrow().elements[0].position();
        println!("ElementOne(x={x}, y={y})", x = position.x, y = position.y);

        let s = MainState {
            canvas: graphics::Canvas::new(
                ctx,
                SCREEN_WIDTH as u16,
                SCREEN_HEIGHT as u16,
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
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::set_canvas(ctx, Some(&self.canvas));
        graphics::set_screen_coordinates(
            ctx,
            Rect::new(0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
        )
        .unwrap();
        graphics::clear(ctx, graphics::Color::from((255, 255, 255, 255)));

        self.menu.borrow().draw(ctx);

        graphics::set_canvas(ctx, None);
        graphics::draw(
            ctx,
            &self.canvas,
            graphics::DrawParam::new().color(Color::from((255, 255, 255, 255))),
        )?;

        graphics::present(ctx)?;

        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        match button {
            event::MouseButton::Left => self
                .menu
                .borrow()
                .input_at(mouse_position(ctx), &mut self.state),
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
