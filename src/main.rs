mod path;
// mod pathfind; // This is for prototype 2
mod bullet;
mod enemy;
mod renderer;
mod tower;
mod ui;
mod vector;

use std::cell::RefCell;
use std::rc::Rc;

use bullet::bullet::Bullet;
use enemy::enemy::Enemy;
use ggez::event;
use ggez::graphics::{self, get_window_color_format, Color, Rect};
use ggez::input::mouse;
use ggez::{Context, GameResult};

use path::Web;
use renderer::draw_circle;
use tower::tower::{spawn_tower, TestTower, Tower};
use ui::{Button, DragButton, Menu};
use vector::*;

pub const SCREEN_WIDTH: usize = 1920;
pub const SCREEN_HEIGHT: usize = 1080;

#[derive(Debug, Clone, Copy)]
pub struct Alive;
#[derive(Debug, Clone, Copy)]
pub struct Dead;

#[derive(Debug)]
pub enum Updated<AliveType, DeadType> {
    Alive(AliveType),
    Dead(DeadType),
}

pub struct GameState<'a> {
    path: Web,
    enemies: RefCell<Vec<Enemy<'a, Alive>>>,
    bullets: RefCell<Vec<Bullet<'a, Alive>>>,
    towers: Vec<Box<dyn Tower<'a> + 'a>>,
    hover_position: Option<Vector>,
}

impl<'a> GameState<'a> {
    pub fn new() -> Self {
        let path = Web::new(
            vec![
                vec2d![210.0, 10.0],
                vec2d![700.0, 100.0],
                vec2d![350.0, 200.0],
                vec2d![1000.0, 1000.0],
            ],
            vec![(0, 1), (1, 3), (0, 2), (1, 2), (2, 3)],
            vec![0, 1, 3],
        )
        .expect("Failed to build a path");

        Self {
            enemies: RefCell::new(vec![Enemy::new_random(path.route().clone())]),
            bullets: RefCell::new(Vec::new()),
            path,
            towers: Vec::new(),
            hover_position: None,
        }
    }
}

pub struct MainState {
    canvas: graphics::Canvas,
    menu: Rc<RefCell<Menu<'static, GameState<'static>>>>,
    state: GameState<'static>,
}

pub fn mouse_position(ctx: &mut Context) -> Vector {
    let mouse_position = mouse::position(ctx);
    let window_size = graphics::drawable_size(ctx);
    vec2d![
        mouse_position.x * SCREEN_WIDTH as f32 / window_size.0,
        mouse_position.y * SCREEN_HEIGHT as f32 / window_size.1
    ]
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let menu: Rc<RefCell<Menu<GameState>>> =
            Rc::new(RefCell::new(Menu::new(vec2d!(0.0, 0.0), 1.0, None)));
        let buttons = vec![
            Button::new(
                vec2d![0.0, 100.0],
                vec2d![100.0, 100.0],
                Rc::clone(&menu),
                |state: &mut GameState| {
                    state.towers.push(spawn_tower(vec2d![
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32
                    ]));
                },
                "Spawn Tower",
            )
            .into(),
            DragButton::new(
                vec2d![0.0, 300.0],
                vec2d![75.0, 75.0],
                Rc::clone(&menu),
                |start, state| state.hover_position = Some(start),
                |start, position, movement, state| state.hover_position = Some(position),
                |start, position, state| {
                    state.hover_position = None;
                    state.towers.push(spawn_tower(position));
                },
                "Drag!",
            )
            .into(),
        ];
        menu.borrow_mut().add_elements(buttons);
        graphics::set_drawable_size(ctx, 1920.0 / 2.0, 1080.0 / 2.0).unwrap();

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
        let size = graphics::drawable_size(_ctx);
        let enemies = Enemy::update_all(self.state.enemies.replace(Vec::new()));
        self.state.enemies.replace(enemies);
        let (bullets, mut enemies) = Bullet::update_all(
            self.state.bullets.replace(Vec::new()),
            self.state.enemies.replace(Vec::new()),
            vec2d![size.0, size.1],
        );
        for tower in self.state.towers.iter_mut() {
            enemies = tower.update(enemies, vec2d! {SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32});
        }
        self.state.bullets.replace(bullets);
        self.state.enemies.replace(enemies);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::set_canvas(ctx, Some(&self.canvas));
        graphics::set_screen_coordinates(
            ctx,
            Rect::new(0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
        )
        .unwrap();
        graphics::clear(ctx, graphics::Color::from((0, 0, 0, 255)));

        self.state.path.draw(ctx);
        for enemy in self.state.enemies.borrow().iter() {
            enemy.draw(ctx);
        }
        for bullet in self.state.bullets.borrow().iter() {
            bullet.draw(ctx);
        }
        for tower in &self.state.towers {
            tower.draw(ctx);
        }
        self.menu.borrow().draw(ctx);
        if let Some(position) = self.state.hover_position {
            draw_circle(ctx, position, 10.0, Color::WHITE);
        }

        graphics::set_canvas(ctx, None);
        graphics::draw(
            ctx,
            &self.canvas,
            graphics::DrawParam::new().color(Color::from((255, 255, 255, 255))),
        )?;

        graphics::present(ctx)?;

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        match button {
            event::MouseButton::Left => {
                self.menu
                    .borrow_mut()
                    .input_start(mouse_position(ctx), &mut self.state);
            }
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        if let Ok(mut menu) = self.menu.try_borrow_mut() {
            menu.input_moved(mouse_position(ctx), vec2d![dx, dy], &mut self.state);
        } else {
            println!("Can't borrow menu!");
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        match button {
            event::MouseButton::Left => {
                self.menu
                    .borrow_mut()
                    .input_released(mouse_position(ctx), &mut self.state);
            }
            _ => (),
        }
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("ACROSS", "Cosmo Brain");
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
