pub mod enemy {
    use std::path::Path;

    use ggez::{graphics::Color, Context};

    use crate::{path::Route, renderer::draw_circle, vector::Vector};

    #[derive(Debug)]
    pub enum UpdatedEnemy<'a> {
        Alive(Enemy<'a, AliveEnemy>),
        Dead(Enemy<'a, DeadEnemy>),
    }
    use UpdatedEnemy::*;

    #[derive(Debug, Clone, Copy)]
    pub struct AliveEnemy;
    #[derive(Debug, Clone, Copy)]
    pub struct DeadEnemy;
    #[derive(Debug)]
    pub struct Enemy<'a, State> {
        enemy: Box<dyn EnemyTrait<'a> + 'a>,
        state: std::marker::PhantomData<State>,
    }

    impl<'a> Enemy<'a, AliveEnemy> {
        pub fn new_random(route: Route) -> Enemy<'a, AliveEnemy> {
            TestEnemy::spawn(route)
        }

        pub fn new(enemy: Box<dyn EnemyTrait<'a> + 'a>) -> Enemy<'a, AliveEnemy> {
            Enemy {
                enemy,
                state: std::marker::PhantomData::<AliveEnemy>,
            }
        }

        pub fn update_all(mut enemies: Vec<Enemy<'a, AliveEnemy>>) -> Vec<Enemy<'a, AliveEnemy>> {
            let mut new_enemies = Vec::with_capacity(enemies.len());
            while let Some(enemy) = enemies.pop() {
                let updated = enemy.update();
                if let Alive(enemy) = updated {
                    new_enemies.push(enemy);
                }
            }
            new_enemies
        }
    }

    impl<'a> Enemy<'a, AliveEnemy> {
        pub fn update(mut self) -> UpdatedEnemy<'a> {
            let alive = self.enemy.update();
            if alive {
                UpdatedEnemy::Alive(Enemy::new(self.enemy))
            } else {
                UpdatedEnemy::Dead(Enemy {
                    enemy: self.enemy,
                    state: std::marker::PhantomData::<DeadEnemy>,
                })
            }
        }

        pub fn draw(&self, ctx: &mut Context) {
            self.enemy.draw(ctx);
        }
    }

    pub trait EnemyTrait<'a>: std::fmt::Debug {
        /// Draw the enemy to the screen
        fn draw(&self, ctx: &mut Context);
        /// Spawn an enemy on a path
        fn spawn(route: Route) -> Enemy<'a, AliveEnemy>
        where
            Self: Sized;
        /// Update the enemy (move it one frame forward)
        /// Return true if the enemy is still alive
        fn update(&mut self) -> bool;
        /// Get the health of the enemy, normalised [0-1]
        fn health(&self) -> f32;
        /// Get the progress along the route, normalised [0-1]
        fn progress(&self) -> f32;
        /// Is the enemy dead? should it be updated?
        fn dead(&self) -> bool {
            self.health() <= 0.0 || self.progress() >= 1.0
        }
        /// The radius of the enemy
        /// TODO: consider replacing this with a Collider struct?
        fn radius(&self) -> f32;
        /// Damage the enemy
        fn damage(&mut self, dmg: f32);
        /// Get the route this enemy is following
        fn route(&'a self) -> &'a Route;
        /// Get the position of the enemy
        fn position(&'a self) -> Vector {
            self.route().get_position(self.progress()).unwrap()
        }
    }

    #[derive(Debug)]
    struct TestEnemy {
        path: Route,
        progress: f32,
        health: f32,
    }

    impl<'a> EnemyTrait<'a> for TestEnemy {
        fn draw(&self, ctx: &mut Context) {
            draw_circle(ctx, self.position(), self.radius(), Color::RED);
        }

        fn spawn(path: Route) -> Enemy<'a, AliveEnemy> {
            Enemy {
                enemy: Box::new(Self {
                    path,
                    progress: 0.0,
                    health: 1.0,
                }) as Box<dyn EnemyTrait<'a> + 'a>,
                state: std::marker::PhantomData::<AliveEnemy>,
            }
        }

        fn update(&mut self) -> bool {
            self.progress += 0.0012;
            self.progress < 1.0
        }

        fn health(&self) -> f32 {
            self.health
        }

        fn progress(&self) -> f32 {
            self.progress
        }

        fn radius(&self) -> f32 {
            15.0
        }

        fn damage(&mut self, dmg: f32) {
            self.health = 0.0f32.max(self.health - dmg);
        }

        fn route(&'a self) -> &'a Route {
            &self.path
        }
    }
}
