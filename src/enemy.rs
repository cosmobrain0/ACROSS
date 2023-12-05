use ggez::{graphics::Color, Context};

use crate::{
    collisions::{line_circle_collision, LineCircleCollision},
    path::Route,
    renderer::{draw_circle, draw_progress_bar},
    vec2d,
    vector::Vector,
    Alive, Dead, Updated,
};

/// Represents an enemy with a fixed state `State`
/// either `Alive` or `Dead`
#[derive(Debug)]
pub struct Enemy<'a, State> {
    enemy: Box<dyn EnemyTrait<'a> + 'a>,
    state: std::marker::PhantomData<State>,
}

/// These methods only exist for living enemies
impl<'a> Enemy<'a, Alive> {
    /// Constructs a new enemy. New enemies are always alive
    pub fn new(enemy: Box<dyn EnemyTrait<'a> + 'a>) -> Enemy<'a, Alive> {
        Enemy {
            enemy,
            state: std::marker::PhantomData::<Alive>,
        }
    }

    /// Updates a list of enemies, consuming the list
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

/// These methods only exist for living enemies
impl<'a> Enemy<'a, Alive> {
    /// Updates an enemy
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

    /// Draws an enemy
    pub fn draw(&self, ctx: &mut Context) {
        self.enemy.draw(ctx);
    }

    /// Checks if an enemy collides with a circle
    pub fn collides(&self, position: Vector, radius: f32) -> bool {
        self.enemy.collides(position, radius)
    }

    /// Returns the position of the enemy
    pub fn position(&self) -> Vector {
        self.enemy.position()
    }

    /// Returns the velocity of the enemy (pixels/frame)
    pub fn velocity(&self) -> Vector {
        self.enemy.velocity()
    }

    pub fn damage(mut self, dmg: f32) -> Updated<Enemy<'a, Alive>, Enemy<'a, Dead>> {
        let alive = self.enemy.damage(dmg);
        if alive {
            Updated::Alive(Enemy::new(self.enemy))
        } else {
            Updated::Dead(Enemy {
                enemy: self.enemy,
                state: std::marker::PhantomData,
            })
        }
    }

    /// Checks if this enemy intersects with
    /// a line segment between points `a` and `b`
    pub fn line_collision(&self, a: Vector, b: Vector) -> bool {
        line_circle_collision(self.enemy.position(), self.enemy.radius(), a, b)
            != LineCircleCollision::None
    }
}

pub trait EnemyTrait<'a>: std::fmt::Debug {
    /// Draw the enemy to the screen
    fn draw(&self, ctx: &mut Context);
    /// Spawn an enemy on a path
    fn spawn(route: Route, round_number: usize) -> Enemy<'a, Alive>
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
    fn damage(&mut self, dmg: f32) -> bool;
    /// Get the route this enemy is following
    fn route(&self) -> &Route;
    /// Get the position of the enemy
    fn position(&self) -> Vector {
        self.route().get_position(self.progress())
    }
    /// Get the velocity of theenemy
    fn velocity(&self) -> Vector;
    /// Check if this collides with another circle
    fn collides(&self, position: Vector, radius: f32) -> bool {
        (self.position() - position).sqr_length()
            <= (radius + self.radius()) * (radius + self.radius())
    }
}

/// A simple enemy
#[derive(Debug)]
pub struct TestEnemy {
    path: Route,
    progress: f32,
    health: f32,
    progress_increment: f32,
}

/// See docs for `EnemyTrait`
impl<'a> EnemyTrait<'a> for TestEnemy {
    fn draw(&self, ctx: &mut Context) {
        draw_progress_bar(
            ctx,
            self.position() - vec2d![self.radius(), self.radius() + 5.0 + 10.0],
            vec2d![self.radius() * 3.0, 10.0],
            self.health,
            Color::from_rgb(180, 180, 180),
            Color::from_rgb(0, 200, 230),
            2.0,
        );
        draw_circle(ctx, self.position(), self.radius(), Color::RED);
    }

    fn spawn(path: Route, round_number: usize) -> Enemy<'a, Alive> {
        let speed = 1.0 + round_number as f32 * 0.3;
        let length = path.length();
        Enemy {
            enemy: Box::new(Self {
                path,
                progress: 0.0,
                health: 1.0,
                progress_increment: speed / length,
            }) as Box<dyn EnemyTrait<'a> + 'a>,
            state: std::marker::PhantomData::<Alive>,
        }
    }

    fn update(&mut self) -> bool {
        self.progress += self.progress_increment;
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

    fn damage(&mut self, dmg: f32) -> bool {
        self.health = 0.0f32.max(self.health - dmg);
        self.health > 0.0
    }

    fn route(&self) -> &Route {
        &self.path
    }

    fn velocity(&self) -> Vector {
        self.path
            .get_position(self.progress + self.progress_increment)
            - self.path.get_position(self.progress)
    }
}

/// A harder-to-beat enemy
#[derive(Debug, Clone)]
pub struct FastEnemy {
    path: Route,
    progress: f32,
    health: f32,
    progress_increment: f32,
}

impl<'a> EnemyTrait<'a> for FastEnemy {
    fn draw(&self, ctx: &mut Context) {
        draw_progress_bar(
            ctx,
            self.position() - vec2d![self.radius(), self.radius() + 5.0 + 10.0],
            vec2d![self.radius() * 3.0, 10.0],
            self.health,
            Color::from_rgb(180, 180, 180),
            Color::from_rgb(0, 200, 230),
            2.0,
        );
        draw_circle(
            ctx,
            self.position(),
            self.radius(),
            Color::from_rgb(166, 22, 199),
        );
    }

    fn spawn(route: Route, round_number: usize) -> Enemy<'a, Alive>
    where
        Self: Sized,
    {
        let speed = 1.1 + round_number as f32 * 0.35;
        let length = route.length();
        Enemy {
            enemy: Box::new(Self {
                path: route,
                progress: 0.0,
                health: 1.0,
                progress_increment: speed / length,
            }) as Box<dyn EnemyTrait<'a> + 'a>,
            state: std::marker::PhantomData::<Alive>,
        }
    }

    fn update(&mut self) -> bool {
        self.progress += self.progress_increment;
        self.progress < 1.0
    }

    fn health(&self) -> f32 {
        self.health
    }

    fn progress(&self) -> f32 {
        self.progress
    }

    fn radius(&self) -> f32 {
        25.0
    }

    fn damage(&mut self, dmg: f32) -> bool {
        // the 1.3 here gives the enemy 1.3 times as much
        // health as `TestEnemy`
        self.health = 0.0f32.max(self.health - dmg / 1.3);
        self.health > 0.0
    }

    fn route(&self) -> &Route {
        &self.path
    }

    fn velocity(&self) -> Vector {
        self.path
            .get_position(self.progress + self.progress_increment)
            - self.path.get_position(self.progress)
    }
}
