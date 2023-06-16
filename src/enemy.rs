pub mod enemy {
    use std::path::Path;

    use ggez::{graphics::Color, Context};

    use crate::{path::Route, renderer::draw_circle, vector::Vector, Alive, Dead, Updated};

    #[derive(Debug)]
    pub struct Enemy<'a, State> {
        enemy: Box<dyn EnemyTrait<'a> + 'a>,
        state: std::marker::PhantomData<State>,
    }

    impl<'a> Enemy<'a, Alive> {
        /// This is mainly for debugging
        pub fn new_random(route: Route) -> Enemy<'a, Alive> {
            TestEnemy::spawn(route)
        }

        pub fn new(enemy: Box<dyn EnemyTrait<'a> + 'a>) -> Enemy<'a, Alive> {
            Enemy {
                enemy,
                state: std::marker::PhantomData::<Alive>,
            }
        }

        pub fn update_all(mut enemies: Vec<Enemy<'a, Alive>>) -> Vec<Enemy<'a, Alive>> {
            let mut new_enemies = Vec::with_capacity(enemies.len());
            while let Some(enemy) = enemies.pop() {
                let updated = enemy.update();
                if let Updated::Alive(enemy) = updated {
                    new_enemies.push(enemy);
                }
            }
            new_enemies
        }
    }

    impl<'a> Enemy<'a, Alive> {
        pub fn update(mut self) -> Updated<Enemy<'a, Alive>, Enemy<'a, Dead>> {
            let alive = self.enemy.update();
            if alive {
                Updated::Alive(Enemy::new(self.enemy))
            } else {
                Updated::Dead(Enemy {
                    enemy: self.enemy,
                    state: std::marker::PhantomData::<Dead>,
                })
            }
        }

        pub fn draw(&self, ctx: &mut Context) {
            self.enemy.draw(ctx);
        }

        pub fn collides(&self, position: Vector, radius: f32) -> bool {
            self.enemy.collides(position, radius)
        }
    }

    pub trait EnemyTrait<'a>: std::fmt::Debug {
        /// Draw the enemy to the screen
        fn draw(&self, ctx: &mut Context);
        /// Spawn an enemy on a path
        fn spawn(route: Route) -> Enemy<'a, Alive>
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
        /// Originally intended to replace this with a Collider struct but that's overkill.
        /// I can guarantee that I'll always use circles
        fn radius(&self) -> f32;
        /// Damage the enemy
        fn damage(&mut self, dmg: f32);
        /// Get the route this enemy is following
        fn route<'b>(&'b self) -> &'b Route;
        /// Get the position of the enemy
        fn position(&self) -> Vector {
            self.route().get_position(self.progress()).unwrap()
        }
        /// Check if this collides with another circle
        fn collides(&self, position: Vector, radius: f32) -> bool {
            (self.position() - position).sqr_length()
                <= (radius + self.radius()) * (radius + self.radius())
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

        fn spawn(path: Route) -> Enemy<'a, Alive> {
            Enemy {
                enemy: Box::new(Self {
                    path,
                    progress: 0.0,
                    health: 1.0,
                }) as Box<dyn EnemyTrait<'a> + 'a>,
                state: std::marker::PhantomData::<Alive>,
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

        fn route<'b>(&'b self) -> &'b Route {
            &self.path
        }
    }
}
