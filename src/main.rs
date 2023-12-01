// Naming Conventions
// snake_case: modules, functions, variables, macros
// SCREAMING_SNAKE_CASE: constants
// PascalCase: structs, types, traits

mod bullet;
mod collisions;
mod enemy;
mod files;
mod path;
mod pathfind;
mod renderer;
mod rounds;
mod tower;
mod ui;
mod vector;

use collisions::{line_circle_collision, line_sector_collision};
use files::{load_from_file, save_to_file, SaveData};
use rounds::Round;
use std::cell::RefCell;
use std::f32::consts::PI;
use std::path::PathBuf;
use std::rc::Rc;

use bullet::Bullet;
use enemy::Enemy;
use ggez::event;
use ggez::graphics::{self, get_window_color_format, Color, Rect};
use ggez::input::mouse;
use ggez::{Context, GameResult};

use path::{Route, Web};
use renderer::{draw_circle, draw_line, draw_sector, draw_text};
use tower::{Range, SectorTower, TestTower, Tower};
use ui::{Button, DragButton, Menu};
use vector::*;

/// The width of the canvas which will be drawn to, in pixels
pub const SCREEN_WIDTH: usize = 1920;
/// The height of the canvas which will be drawn to, in pixels
pub const SCREEN_HEIGHT: usize = 1080;
/// The number of lives that the player will start the game with
pub const STARTING_LIVES: usize = 5;
/// The ammount (radians) by which the direction of a tower will change
/// when the user rotates the tower they are placing
const TOWER_PLACEMENT_DIRECTION_CHANGE_SPEED: f32 = 5.0 * PI / 180.0;

/// Used as a marker for the compiler
/// to show that something is still "usable"
/// e.g. an enemy is alive, or a bullet
/// is still moving
#[derive(Debug, Clone, Copy)]
pub struct Alive;
/// Used as a marker for the compiler
/// to show that something is unusable
/// and should be destroyed. e.g. an
/// enemy has been killed, or a bullet
/// has hit its target
#[derive(Debug, Clone, Copy)]
pub struct Dead;

/// Used to wrap the result updating something
/// which can be `Dead` or `Alive`
/// This allows for signatures like
/// `fn update(self) -> Updated<Self<Alive>, Self<Dead>>`
/// so that when something is updated, the old value
/// can no longer be used, and the state of the new
/// value (alive or dead) must be considered
#[derive(Debug)]
pub enum Updated<AliveType, DeadType> {
    Alive(AliveType),
    Dead(DeadType),
}

/// Used to represent which "screen" the user
/// is currently interacting with
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// The main menu
    MainMenu,
    /// The main game
    Play,
}

/// Used to store information about the current state
/// of the game
pub struct GameState<'a> {
    path: Web,
    towers: Vec<Box<dyn Tower<'a> + 'a>>,
    hover_position: Option<Vector>,
    mode: GameMode,
    round: Round<'a>,
    lives: usize,
    score: usize,
    money: usize,
    /// this is **not** good but it works
    tower_placement_direction: f32,
    tower_placement_range: Option<Box<dyn Range>>,
}

impl<'a> GameState<'a> {
    /// Inititialises the game
    /// at round 0, with no towers placed
    /// and a score of 0, and the correct
    /// number of lives, and 30 money
    pub fn new() -> Self {
        let path = Web::new(
            vec![
                vec2d![210.0, 10.0],
                vec2d![700.0, 100.0],
                vec2d![350.0, 200.0],
                vec2d![1000.0, 1000.0],
                vec2d![370.0, 800.0],
            ],
            vec![
                // (0, 1),
                (1, 3),
                (0, 2),
                (1, 2),
                // (2, 3),
                (0, 4),
                (1, 4),
                (4, 2),
                (4, 3),
            ]
            .into_iter()
            .map(|(a, b)| [(a, b), (b, a)])
            .flatten()
            .collect(),
            0,
            1,
            |a, b| (a - b).length(),
        )
        .expect("Failed to build a path");

        Self {
            path,
            towers: Vec::new(),
            hover_position: None,
            mode: GameMode::MainMenu,
            round: Round::new(1),
            lives: STARTING_LIVES,
            score: 0,
            money: 30,
            tower_placement_direction: 0.0,
            tower_placement_range: None,
        }
    }
}

impl<'a> Default for GameState<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Stores the data for the whole application
/// including the `GameState`
pub struct MainState {
    /// The canvas to draw to
    canvas: graphics::Canvas,
    /// the in-game menu for `GameMode::Play`
    menu: Rc<RefCell<Menu<'static, GameState<'static>>>>,
    /// the main menu for `GameMode::Menu`
    main_menu: Rc<RefCell<Menu<'static, GameState<'static>>>>,
    /// the state of the game
    state: GameState<'static>,
    /// data for all played games
    score_list: Vec<SaveData>,
}

/// Returns the coordinates of the mouse in screen-space
/// so (0, 0) is the top-left of the screen and
/// (SCREEN_WIDTH, SCREEN_HEIGHT) is the bottom-right
pub fn mouse_position(ctx: &mut Context) -> Vector {
    let mouse_position = mouse::position(ctx);
    let window_size = graphics::drawable_size(ctx);
    vec2d![
        mouse_position.x * SCREEN_WIDTH as f32 / window_size.0,
        mouse_position.y * SCREEN_HEIGHT as f32 / window_size.1
    ]
}

/// Constructs a `DragButton` which can be used
/// by the player to place a tower.
macro_rules! tower_button {
    ($position:expr, $tower:ident $name:literal $menu:ident) => {
        DragButton::new(
            $position,
            vec2d![100.0, 75.0],
            Rc::clone(&$menu),
            |start, state: &mut GameState| {
                state.hover_position = Some(start);
                state.tower_placement_range =
                    Some($tower::new_range(start, state.tower_placement_direction));
                // TODO: use the new tower trait thing
            },
            |_start, position, _movement, state| {
                state.hover_position = Some(position);
                state
                    .tower_placement_range
                    .as_mut()
                    .map(|x| x.set_position(position));
            },
            |_start, position, state| {
                state.hover_position = None;
                if (130.0..=SCREEN_WIDTH as f32 - 10.0).contains(&position.x)
                    && (10.0..SCREEN_HEIGHT as f32 - 10.0).contains(&position.y)
                {
                    let price = $tower::price();
                    if price <= state.money {
                        state.money -= price;
                        state
                            .towers
                            .push($tower::spawn(position, state.tower_placement_direction));
                        state.tower_placement_direction = 0.0;
                        state.tower_placement_range = None;
                        state.path.recalculate_weights(|a, b| {
                            (a - b).length()
                                + state
                                    .towers
                                    .iter()
                                    .map(|tower| tower.visible_area(a, b))
                                    .sum::<f32>()
                                    * 5.0
                        });
                        state.path.pathfind();
                    }
                }
            },
            $name,
        )
    };
}

impl MainState {
    /// Initialises the game and window
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let menu: Rc<RefCell<Menu<GameState>>> =
            Rc::new(RefCell::new(Menu::new(vec2d!(0.0, 0.0), 1.0, None)));
        let buttons = vec![
            tower_button![vec2d![10.0, 100.0], SectorTower "Sector" menu].into(),
            tower_button![vec2d![10.0, 200.0], TestTower "Test" menu].into(),
            Button::new(
                vec2d![0.0, 0.0],
                vec2d![100.0, 80.0],
                Rc::clone(&menu),
                |state| {
                    state.mode = GameMode::MainMenu;
                },
                "Pause",
            )
            .into(),
        ];
        menu.borrow_mut().add_elements(buttons);

        let main_menu = Rc::new(RefCell::new(Menu::new(
            vec2d!(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0),
            1.0,
            None,
        )));
        let buttons = vec![Button::new(
            vec2d![-100.0, -50.0],
            vec2d![200.0, 100.0],
            Rc::clone(&main_menu),
            |state: &mut GameState| {
                state.mode = GameMode::Play;
                // FIXME: This needs to restart the game or something maybe?
            },
            "Play",
        )];
        main_menu
            .borrow_mut()
            .add_elements(buttons.into_iter().map(Into::into).collect());

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
            main_menu,
            state: GameState::new(),
            score_list: multi_try(move |_| load_from_file("./save-file.csv".into()), (), 3)
                .unwrap_or_else(|| vec![]),
        };
        Ok(s)
    }
}

/// Attempts to call `callback` multiple times
/// until it returns `Result::Ok(O)`,
/// or the maximum number of attempts (`tries`)
/// has been reached.
/// Returns the first `Ok` value, if there was one.
pub fn multi_try<I: Clone, O, E>(
    callback: impl Fn(I) -> Result<O, E>,
    input: I,
    tries: usize,
) -> Option<O> {
    for _ in 0..tries {
        if let Ok(value) = callback(input.clone()) {
            return Some(value);
        }
    }
    None
}

impl event::EventHandler<ggez::GameError> for MainState {
    /// Updates the bullets, enemies and towers
    /// moving them one step forward in time
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let size = graphics::drawable_size(_ctx);
        match self.state.mode {
            GameMode::MainMenu => Ok(()),
            GameMode::Play => {
                let (lives_lost, enemies_killed) =
                    self.state
                        .round
                        .update(&self.state.path, &mut self.state.towers, size);
                self.state.lives = self.state.lives.saturating_sub(lives_lost);
                self.state.score = self.state.score.saturating_add(enemies_killed);
                self.state.money += enemies_killed * 2;

                if self.state.lives == 0 {
                    let save_data = SaveData::new(chrono::Utc::now(), self.state.score);
                    multi_try(
                        |_| save_to_file("./save-file.csv".into(), save_data.clone()),
                        (),
                        3,
                    );
                    self.score_list.push(save_data);
                    self.state = GameState::new();
                    return Ok(());
                }

                if self.state.round.complete() {
                    self.state.round.next();
                }
                Ok(())
            }
        }
    }

    /// Draws the main menu, or the current state of the game
    /// depending on the current `GameMode`
    /// to the canvas, then draws the canvas to the screen
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
                self.main_menu.borrow().draw(ctx);
                for (y_offset, info) in
                    self.score_list
                        .iter()
                        .enumerate()
                        .map(|(i, SaveData { date, score })| {
                            (
                                i as f32 * 30.0,
                                format!(
                                    "{score} points at {date}",
                                    date = date.format("%d/%m/%Y %H:%M:%S")
                                ),
                            )
                        })
                {
                    draw_text(
                        ctx,
                        info.as_str(),
                        vec2d![10.0, 10.0 + y_offset],
                        None,
                        None,
                        Color::WHITE,
                    );
                }
            }
            GameMode::Play => {
                self.state.path.draw(ctx);
                for enemy in self.state.round.enemies().iter() {
                    enemy.draw(ctx);
                }
                // for bullet in self.state.round.bullets().iter() {
                //     bullet.draw(ctx);
                // }
                for tower in &self.state.towers {
                    tower.draw(ctx);
                }
                if let Some(position) = self.state.hover_position {
                    draw_circle(ctx, position, 10.0, Color::WHITE);
                    if let Some(range) = self.state.tower_placement_range.as_mut() {
                        range.draw(ctx);
                    }
                }
                self.menu.borrow().draw(ctx);
                draw_text(
                    ctx,
                    format!(
                        "Lives: {lives}\nScore: {score}\nMoney: {money}",
                        lives = self.state.lives,
                        score = self.state.score,
                        money = self.state.money,
                    )
                    .as_str(),
                    vec2d![SCREEN_WIDTH as f32 - 250.0, 30.0],
                    None,
                    None,
                    Color::WHITE,
                );

                // let radius = 50.0;
                // let position = vec2d![SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32] / 2.0;
                // let (direction, fov) = (0.0, PI / 2.0);
                // let (start_angle, end_angle) = (direction - fov / 2.0, direction + fov / 2.0);

                // draw_sector(
                //     ctx,
                //     position,
                //     radius,
                //     start_angle,
                //     end_angle,
                //     100,
                //     Color::RED,
                // );

                // if let Some(a) = self.t_point_a {
                //     if let Some(b) = self.t_point_b {
                //         draw_line(ctx, a, b, 3.0, Color::GREEN);
                //         let points = line_sector_collision(position, radius, direction, fov, a, b);
                //         for point in points {
                //             draw_circle(ctx, point, 10.0, Color::WHITE);
                //         }
                //     }
                // }
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

    /// Handles clicking on buttons on the menu
    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        if button == event::MouseButton::Left {
            match self.state.mode {
                GameMode::MainMenu => {
                    vec![&mut self.main_menu]
                }
                GameMode::Play => {
                    vec![&mut self.menu] // maybe I cna do this with a slice?
                }
            }
            .into_iter()
            .for_each(|x| {
                x.borrow_mut()
                    .input_start(mouse_position(ctx), &mut self.state)
            });
        }
    }

    /// Handles dragging from menu buttons
    fn mouse_motion_event(&mut self, ctx: &mut Context, _x: f32, _y: f32, dx: f32, dy: f32) {
        match self.state.mode {
            GameMode::MainMenu => {
                vec![&mut self.main_menu]
            }
            GameMode::Play => {
                vec![&mut self.menu] // maybe I cna do this with a slice?
            }
        }
        .into_iter()
        .for_each(|x| {
            x.borrow_mut()
                .input_moved(mouse_position(ctx), vec2d![dx, dy], &mut self.state);
        });
    }

    /// Handles releasing menu buttons
    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        if button == event::MouseButton::Left {
            match self.state.mode {
                GameMode::MainMenu => {
                    vec![&mut self.main_menu]
                }
                GameMode::Play => {
                    vec![&mut self.menu] // maybe I cna do this with a slice?
                }
            }
            .into_iter()
            .for_each(|x| {
                x.borrow_mut()
                    .input_released(mouse_position(ctx), &mut self.state);
            });
        }
    }

    /// Handles rotating the tower
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymods: event::KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            event::KeyCode::A => {
                self.state.tower_placement_direction -= TOWER_PLACEMENT_DIRECTION_CHANGE_SPEED;
                self.state
                    .tower_placement_range
                    .as_mut()
                    .map(|x| x.set_direction(self.state.tower_placement_direction));
            }
            event::KeyCode::D => {
                self.state.tower_placement_direction += TOWER_PLACEMENT_DIRECTION_CHANGE_SPEED;
                self.state
                    .tower_placement_range
                    .as_mut()
                    .map(|x| x.set_direction(self.state.tower_placement_direction));
            }
            _ => {
                return;
            }
        }
    }
}

/// Initialises and starts the game
pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("ACROSS", "Cosmo Brain");
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
