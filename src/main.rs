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

const GAME_MENU_INDEX: usize = 0;
const MAIN_MENU_INDEX: usize = 1;

/// A zero-sized type to mark living enemies as alive
/// This is so the compiler can find logic errors where dead enemies are left
/// in the enemies array, for example
#[derive(Debug, Clone, Copy)]
pub struct Alive;
/// A zero-sized type to mark enemies as dead
/// as opposed to Alive
#[derive(Debug, Clone, Copy)]
pub struct Dead;

/// Updating game elements such as enemies or bullets
/// Can result in either a living element or a dead element
/// This sum type defines those two possibilities so one function
/// can return either of the two.
#[derive(Debug)]
pub enum Updated<AliveType, DeadType> {
    Alive(AliveType),
    Dead(DeadType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    MainMenu,
    Play,
}

/// This stores the state of the game
/// and can be manipulated by menus
pub struct GameState<'a> {
    path: Web,
    enemies: RefCell<Vec<Enemy<'a, Alive>>>,
    bullets: RefCell<Vec<Bullet<'a, Alive>>>,
    towers: Vec<Box<dyn Tower<'a> + 'a>>,
    hover_position: Option<Vector>,
    mode: GameMode,
}

impl<'a> GameState<'a> {
    /// Initialises the game
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

        // one enemy at the beginning of the route
        // no bullets
        // one web
        // no towers
        Self {
            enemies: RefCell::new(vec![Enemy::new_random(path.route().clone())]),
            bullets: RefCell::new(Vec::new()),
            path,
            towers: Vec::new(),
            hover_position: None,
            mode: GameMode::MainMenu,
        }
    }
}

/// This stores all of the data related to the game, including the canvas and menu
pub struct MainState {
    canvas: graphics::Canvas,
    menus: Vec<Rc<RefCell<Menu<'static, GameState<'static>>>>>,
    state: GameState<'static>,
}

/// Gets the position of the mouse and converts it to a fixed scale, regardless of screen size or the position of the window
pub fn mouse_position(ctx: &mut Context) -> Vector {
    let mouse_position = mouse::position(ctx);
    let window_size = graphics::drawable_size(ctx);
    vec2d![
        mouse_position.x * SCREEN_WIDTH as f32 / window_size.0,
        mouse_position.y * SCREEN_HEIGHT as f32 / window_size.1
    ]
}

macro_rules! menu_new {
    ($location:expr, $scale:expr, $parent:expr, [$({$type:ident, $button_location:expr, $button_size:expr, $($arguments:expr,)* $(,)?})*]) => {
        {
            let menu = Rc::new(RefCell::new(Menu::new($location, $scale, $parent)));
            let buttons = vec![
                $($type::new($button_location, $button_size, Rc::clone(&menu), $($arguments,)*).into(),)*
            ];
            menu.borrow_mut().add_elements(buttons);
            menu
        }
    }
}

impl MainState {
    /// Initialises the game
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let game_menu = menu_new!(
            vec2d![0.0, 0.0],
            1.0,
            None,
            [
                {
                    Button, vec2d![0.0, 100.0], vec2d![100.0, 100.0],
                    |state: &mut GameState| {
                        state.towers.push(spawn_tower(vec2d![
                            SCREEN_WIDTH as f32,
                            SCREEN_HEIGHT as f32
                        ]));
                    },
                    "Spawn Tower",
                }
                {
                    DragButton, vec2d![0.0, 300.0], vec2d![75.0, 75.0],
                    |start, state| state.hover_position = Some(start),
                    |start, position, movement, state| state.hover_position = Some(position),
                    |start, position, state| {
                        state.hover_position = None;
                        state.towers.push(spawn_tower(position));
                    },
                    "Drag!",
                }
                {
                    Button, vec2d![0.0, 500.0], vec2d![100.0, 100.0],
                    |state: &mut GameState| {
                        state.mode = GameMode::MainMenu;
                    },
                    "Pause",
                }
            ]
        );
        let main_menu = menu_new!(
            vec2d![SCREEN_WIDTH as f32/2.0, SCREEN_HEIGHT as f32/2.0],
            1.0,
            None,
            [
                {
                    Button, vec2d![-50.0, -100.0], vec2d![100.0, 100.0],
                    |state: &mut GameState| {
                        state.mode = GameMode::Play;
                    },
                    "Play",
                }
            ]
        );
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
            menus: vec![game_menu, main_menu],
            state: GameState::new(),
        };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    /// Moves the game one step through time
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let size = graphics::drawable_size(_ctx);

        if self.state.mode == GameMode::Play {
            // update enemies
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
        }
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

        match self.state.mode {
            GameMode::MainMenu => {
                self.menus[MAIN_MENU_INDEX].borrow().draw(ctx);
            }
            GameMode::Play => {
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
                self.menus[GAME_MENU_INDEX].borrow().draw(ctx);
                if let Some(position) = self.state.hover_position {
                    draw_circle(ctx, position, 10.0, Color::WHITE);
                }
            }
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
            event::MouseButton::Left => match self.state.mode {
                GameMode::Play => self.menus[GAME_MENU_INDEX]
                    .borrow_mut()
                    .input_start(mouse_position(ctx), &mut self.state),
                GameMode::MainMenu => self.menus[MAIN_MENU_INDEX]
                    .borrow_mut()
                    .input_start(mouse_position(ctx), &mut self.state),
            },
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        match self.state.mode {
            GameMode::Play => self.menus[GAME_MENU_INDEX].borrow_mut().input_moved(
                mouse_position(ctx),
                vec2d![dx, dy],
                &mut self.state,
            ),
            GameMode::MainMenu => self.menus[MAIN_MENU_INDEX].borrow_mut().input_moved(
                mouse_position(ctx),
                vec2d![dx, dy],
                &mut self.state,
            ),
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
            event::MouseButton::Left => match self.state.mode {
                GameMode::Play => self.menus[GAME_MENU_INDEX]
                    .borrow_mut()
                    .input_released(mouse_position(ctx), &mut self.state),
                GameMode::MainMenu => self.menus[MAIN_MENU_INDEX]
                    .borrow_mut()
                    .input_released(mouse_position(ctx), &mut self.state),
            },
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
