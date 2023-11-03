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

use collisions::line_circle_collision;
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
use tower::{spawn_tower, Tower};
use ui::{Button, DragButton, Menu};
use vector::*;

pub const SCREEN_WIDTH: usize = 1920;
pub const SCREEN_HEIGHT: usize = 1080;
pub const STARTING_LIVES: usize = 5;

#[derive(Debug, Clone, Copy)]
pub struct Alive;
#[derive(Debug, Clone, Copy)]
pub struct Dead;

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

pub struct GameState<'a> {
    path: Web,
    towers: Vec<Box<dyn Tower<'a> + 'a>>,
    hover_position: Option<Vector>,
    mode: GameMode,
    round: Round<'a>,
    lives: usize,
    score: usize,
    money: usize,
}

impl<'a> GameState<'a> {
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
            .map(|(a, b)| (a, b, 0.0))
            .map(|(a, b, weight)| [(a, b, weight), (b, a, weight)])
            .flatten()
            .collect(),
            0,
            1,
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
        }
    }
}

impl<'a> Default for GameState<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MainState {
    canvas: graphics::Canvas,
    menu: Rc<RefCell<Menu<'static, GameState<'static>>>>,
    main_menu: Rc<RefCell<Menu<'static, GameState<'static>>>>,
    state: GameState<'static>,
    score_list: Vec<SaveData>,
    t_point_a: Option<Vector>,
    t_point_b: Option<Vector>,
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
            DragButton::new(
                vec2d![0.0, 300.0],
                vec2d![75.0, 75.0],
                Rc::clone(&menu),
                |start, state| state.hover_position = Some(start),
                |_start, position, _movement, state| state.hover_position = Some(position),
                |_start, position, state| {
                    state.hover_position = None;
                    if let Some((tower, price)) = spawn_tower(position, state.money) {
                        state.money -= price;
                        state.towers.push(tower);
                    }
                },
                "Drag!",
            )
            .into(),
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
        let buttons = vec![
            Button::new(
                vec2d![-100.0, -100.0],
                vec2d![200.0, 100.0],
                Rc::clone(&main_menu),
                |state: &mut GameState| {
                    state.mode = GameMode::Play;
                    // FIXME: This needs to restart the game or something maybe?
                },
                "Play",
            ),
            Button::new(
                vec2d![-100.0, 50.0],
                vec2d![200.0, 100.0],
                Rc::clone(&main_menu),
                |state| {
                    println!("Something!");
                },
                "Something",
            ),
        ];
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
            score_list: load_from_file("./save-file.csv".into()).unwrap(),
            t_point_a: None,
            t_point_b: None,
        };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let size = graphics::drawable_size(_ctx);
        match self.state.mode {
            GameMode::MainMenu => Ok(()),
            GameMode::Play => {
                // let (lives_lost, enemies_killed) =
                //     self.state
                //         .round
                //         .update(&self.state.path, &mut self.state.towers, size);
                // self.state.lives = self.state.lives.saturating_sub(lives_lost);
                // self.state.score = self.state.score.saturating_add(enemies_killed);
                // self.state.money += enemies_killed * 2;

                // if self.state.lives == 0 {
                //     // TODO: save score to file
                //     // self.state.mode == GameMode::MainMenu;
                //     // TODO: some error handling?
                //     let save_data = SaveData::new(chrono::Utc::now(), self.state.score);
                //     save_to_file("./save-file.csv".into(), save_data).unwrap();
                //     self.score_list.push(save_data);
                //     self.state = GameState::new();
                //     return Ok(());
                // }

                // if self.state.round.complete() {
                //     self.state.round.next();
                // }
                Ok(())
            }
        }
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
                // self.state.path.draw(ctx);
                // for enemy in self.state.round.enemies().iter() {
                //     enemy.draw(ctx);
                // }
                // for bullet in self.state.round.bullets().iter() {
                //     bullet.draw(ctx);
                // }
                // for tower in &self.state.towers {
                //     tower.draw(ctx);
                // }
                // if let Some(position) = self.state.hover_position {
                //     draw_circle(ctx, position, 10.0, Color::WHITE);
                // }
                // self.menu.borrow().draw(ctx);
                // draw_text(
                //     ctx,
                //     format!(
                //         "Lives: {lives}\nScore: {score}\nMoney: {money}",
                //         lives = self.state.lives,
                //         score = self.state.score,
                //         money = self.state.money,
                //     )
                //     .as_str(),
                //     vec2d![SCREEN_WIDTH as f32 - 250.0, 30.0],
                //     None,
                //     None,
                //     Color::WHITE,
                // );
                if let Some(a) = self.t_point_a {
                    if let Some(b) = self.t_point_b {
                        draw_line(ctx, a, b, 3.0, Color::RED);
                    }
                }
                let centre = vec2d![SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32] / 2.0;
                let colour = self.t_point_a.is_some_and(|a| {
                    self.t_point_b.is_some_and(|b| {
                        line_circle_collision(centre, 20.0, a, b)
                            .into_iter()
                            .count()
                            > 0
                    })
                });
                let colour = if colour { Color::GREEN } else { Color::RED };
                draw_circle(ctx, centre, 20.0, colour);
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
        if button == event::MouseButton::Left {
            match self.state.mode {
                GameMode::MainMenu => {
                    vec![&mut self.main_menu]
                }
                GameMode::Play => {
                    // vec![&mut self.menu] // maybe I cna do this with a slice?
                    vec![]
                }
            }
            .into_iter()
            .for_each(|x| {
                x.borrow_mut()
                    .input_start(mouse_position(ctx), &mut self.state)
            });
        }

        if self.state.mode == GameMode::Play && button == event::MouseButton::Left {
            self.t_point_a = Some(mouse_position(ctx));
        } else if self.state.mode == GameMode::Play && button == event::MouseButton::Right {
            self.t_point_b = Some(mouse_position(ctx));
        }
    }

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
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("ACROSS", "Cosmo Brain");
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}

pub mod test {
    macro_rules! assert_pretty_much_equal {
        ($a:expr, $b:expr) => {
            assert!(
                ($a - $b).abs() <= 0.0001,
                "{a} and {b} aren't pretty much equal",
                a = $a,
                b = $b
            );
        };
    }

    use std::f32::consts::PI;

    use crate::tower::shortest_angle_distance;

    #[test]
    fn shortest_angle_distance_test_from_0() {
        assert_pretty_much_equal!(shortest_angle_distance(0.0, PI / 2.0), PI / 2.0);
        assert_pretty_much_equal!(shortest_angle_distance(0.0, -PI / 2.0), -PI / 2.0);
        assert_pretty_much_equal!(shortest_angle_distance(0.0, PI / 2.0 * 3.0), -PI / 2.0);
    }

    #[test]
    fn shortest_angle_distance_test_from_0_reverse() {
        assert_pretty_much_equal!(shortest_angle_distance(PI / 2.0, 0.0), -PI / 2.0);
        assert_pretty_much_equal!(shortest_angle_distance(-PI / 2.0, 0.0), PI / 2.0);
        assert_pretty_much_equal!(shortest_angle_distance(PI / 2.0 * 3.0, 0.0), PI / 2.0);
    }

    #[test]
    fn shortest_angle_distance_test_2() {
        // assert_pretty_much_equal!(shortest_angle_distance())
    }
}
