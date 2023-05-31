mod renderer;
mod ui;
mod vector;

use std::cell::RefCell;
use std::rc::Rc;

use ggez::event;
use ggez::graphics::{self, get_window_color_format, Color, Rect};
use ggez::input::mouse;
use ggez::{Context, GameResult};

use renderer::draw_rectangle;
use ui::{Button, Menu, UIElement};
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
        let (window_width, window_height) = graphics::drawable_size(ctx);
        println!("Window(x={}, y={})", window_width, window_height);
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
        self.menu.borrow_mut().set_position(vec2d!(0.0, 0.0));
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

        let mouse_position = mouse_position(ctx);
        let mouse_position = vec2d!(mouse_position.x, mouse_position.y);

        self.menu.borrow().draw(ctx);
        if let UIElement::Button(button) = &self.menu.borrow().elements[1] {
            if button.is_hovered(mouse_position) {
                draw_rectangle(ctx, vec2d!(0.0, 0.0), vec2d!(20.0, 20.0), Color::RED);
            }
        }

        graphics::set_canvas(ctx, None);
        let (canvas_width, canvas_height) =
            (self.canvas.width() as f32, self.canvas.height() as f32);
        let (window_width, window_height) = graphics::drawable_size(ctx);
        graphics::draw(
            ctx,
            &self.canvas,
            graphics::DrawParam::new().color(Color::from((255, 255, 255, 255))), // .scale([window_width / canvas_width, 1.0]), // .scale([
                                                                                 //     window_width / SCREEN_WIDTH as f32,
                                                                                 //     window_height / SCREEN_HEIGHT as f32,
                                                                                 // ]),
        )?;

        graphics::present(ctx)?;

        Ok(())
    }

    fn key_up_event(
        &mut self,
        ctx: &mut Context,
        keycode: event::KeyCode,
        _keymods: event::KeyMods,
    ) {
        match keycode {
            event::KeyCode::Space => {
                let position = mouse_position(ctx);
                println!("Mouse(x = {}, y = {})", position.x, position.y);
            }
            event::KeyCode::Q => {
                let (canvas_width, canvas_height) =
                    (self.canvas.width() as f32, self.canvas.height() as f32);
                let (window_width, window_height) = graphics::drawable_size(ctx);
                println!("Window(x={window_width}, y={window_height})");
                println!("Canvas(x={canvas_width}, y={canvas_height})");
            }
            _ => (),
        };
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
