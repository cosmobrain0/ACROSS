use ggez::{graphics::Color, Context};

use crate::{
    enemy::Enemy,
    renderer::{draw_circle, draw_line},
    tower::{aim_towards, Tower},
    vector::Vector,
    Alive, Dead, Updated, SCREEN_HEIGHT, SCREEN_WIDTH,
};

/// Represents something which a tower can shoot
pub trait BulletTrait<'a>: std::fmt::Debug {
    /// create a new bullet
    fn spawn(
        tower: &impl Tower<'a>,
        target: Vector,
        target_velocity: Vector,
    ) -> Box<dyn BulletTrait<'a> + 'a>
    where
        Self: Sized;

    /// Returns a reference to the tower which shot this bullet
    fn tower(&self) -> &'a dyn Tower;

    /// Updates this bullet
    fn update<'b>(
        &mut self,
        enemies: Vec<Enemy<'b, Alive>>,
        bounds: Vector,
    ) -> (bool, Vec<Enemy<'b, Alive>>);

    /// Renders this bullet
    fn draw(&self, ctx: &mut Context);
}

/// Represents a bullet which must be
/// in the state `State` (either `Alive` or `Dead`)
/// this is proven at compile-time
#[derive(Debug)]
pub struct Bullet<'a, State> {
    bullet: Box<dyn BulletTrait<'a> + 'a>,
    state: std::marker::PhantomData<State>,
}

/// Represents the result of updating a bullet:
/// either a living bullet `Bullet<'a, Alive>`
/// or a dead bullet `Bullet<'a, Dead>`
type UpdatedBullet<'a> = Updated<Bullet<'a, Alive>, Bullet<'a, Dead>>;

/// These methods only exist for bullets
/// which are alive,
/// as dead bullets can't be constructed
/// or used for anything.
impl<'a> Bullet<'a, Alive> {
    /// Constructs a new bullet. New bullets are
    /// guaranteed to be alive
    pub fn new(bullet: Box<dyn BulletTrait<'a> + 'a>) -> Bullet<'a, Alive> {
        Bullet {
            bullet,
            state: std::marker::PhantomData::<Alive>,
        }
    }

    /// Update the bullet. Dead bullets should not be
    /// updated.
    pub fn update<'b>(
        mut self,
        enemies: Vec<Enemy<'b, Alive>>,
        bounds: Vector,
    ) -> (UpdatedBullet<'a>, Vec<Enemy<'b, Alive>>) {
        let (alive, new_enemies) = self.bullet.update(enemies, bounds);
        if alive {
            (Updated::Alive(Bullet::new(self.bullet)), new_enemies)
        } else {
            (
                Updated::Dead(Bullet {
                    bullet: self.bullet,
                    state: std::marker::PhantomData::<Dead>,
                }),
                new_enemies,
            )
        }
    }

    /// Update a list of living bullets, consuming the list
    pub fn update_all<'b>(
        mut bullets: Vec<Bullet<'a, Alive>>,
        enemies: Vec<Enemy<'b, Alive>>,
        bounds: Vector,
    ) -> (Vec<Bullet<'a, Alive>>, Vec<Enemy<'b, Alive>>) {
        let mut new_bullets = Vec::with_capacity(bullets.len());
        let mut new_enemies = enemies;
        while let Some(bullet) = bullets.pop() {
            let (updated, newer_enemies) = bullet.update(new_enemies, bounds);
            if let Updated::Alive(bullet) = updated {
                new_bullets.push(bullet);
            }
            new_enemies = newer_enemies;
        }
        (new_bullets, new_enemies)
    }

    /// Draw a bullet
    pub fn draw(&self, ctx: &mut Context) {
        self.bullet.draw(ctx);
    }
}

/// a slow-moving, circular `BulletTrait`
#[derive(Debug, Clone)]
pub struct Projectile {
    position: Vector,
    velocity: Vector,
    radius: f32,
}

impl<'a> BulletTrait<'a> for Projectile {
    /// Construct a new projectile
    fn spawn(
        tower: &impl Tower<'a>,
        target: Vector,
        target_velocity: Vector,
    ) -> Box<dyn BulletTrait<'a> + 'a>
    where
        Self: Sized,
    {
        Box::new(Self {
            position: tower.position(),
            velocity: aim_towards(tower.position(), target, target_velocity, 3.0),
            radius: 5.0,
        }) as Box<dyn BulletTrait<'a> + 'a>
    }

    fn tower(&self) -> &'a dyn Tower {
        todo!()
    }

    /// Update the bullet
    fn update<'b>(
        &mut self,
        mut enemies: Vec<Enemy<'b, Alive>>,
        bounds: Vector,
    ) -> (bool, Vec<Enemy<'b, Alive>>) {
        self.position += self.velocity;

        let mut new_enemies = Vec::with_capacity(enemies.len());
        let mut alive = true;
        while let Some(enemy) = enemies.pop() {
            if alive && enemy.collides(self.position, self.radius) {
                if let Updated::Alive(enemy) = enemy.damage(0.4) {
                    new_enemies.push(enemy);
                }
                alive = false;
            } else {
                new_enemies.push(enemy);
            }
        }

        if !alive
            || self.position.x + self.radius < 0.0
            || self.position.y + self.radius < 0.0
            || self.position.x - self.radius > bounds.x
            || self.position.y - self.radius > bounds.y
        {
            (false, new_enemies)
        } else {
            (true, new_enemies)
        }
    }

    /// Draw the bullet as a circle
    fn draw(&self, ctx: &mut Context) {
        draw_circle(
            ctx,
            self.position,
            self.radius,
            Color::new(0.0, 1.0, 0.5, 1.0),
        );
    }
}

const LIGHTNING_LIFESPAN: usize = 30;

/// a very fast moving, line-shaped `BulletTrait`
#[derive(Debug, Clone)]
pub struct Lightning {
    start: Vector,
    direction: Vector,
    life: usize,
}

impl<'a> BulletTrait<'a> for Lightning {
    /// Construct a new projectile
    fn spawn(
        tower: &impl Tower<'a>,
        target: Vector,
        _target_velocity: Vector,
    ) -> Box<dyn BulletTrait<'a> + 'a>
    where
        Self: Sized,
    {
        Box::new(Self {
            start: tower.position(),
            direction: target - tower.position(),
            life: LIGHTNING_LIFESPAN,
        }) as Box<dyn BulletTrait<'a> + 'a>
    }

    fn tower(&self) -> &'a dyn Tower {
        todo!()
    }

    /// Update the bullet
    fn update<'b>(
        &mut self,
        mut enemies: Vec<Enemy<'b, Alive>>,
        bounds: Vector,
    ) -> (bool, Vec<Enemy<'b, Alive>>) {
        if self.life == 0 {
            return (false, enemies);
        }
        self.life -= 1;

        let mut new_enemies = Vec::with_capacity(enemies.len());
        while let Some(enemy) = enemies.pop() {
            if enemy.line_collision(
                self.start,
                self.start
                    + self.direction.normalised()
                        * f32::sqrt(
                            (SCREEN_WIDTH * SCREEN_WIDTH * SCREEN_HEIGHT * SCREEN_HEIGHT) as f32,
                        ),
            ) {
                if let Updated::Alive(enemy) = enemy.damage(0.2) {
                    new_enemies.push(enemy);
                }
            } else {
                new_enemies.push(enemy);
            }
        }

        (true, new_enemies)
    }

    /// Draw the bullet as a line
    fn draw(&self, ctx: &mut Context) {
        // draw_circle(
        //     ctx,
        //     self.start,
        //     self.radius,
        //     Color::new(0.0, 1.0, 0.5, 1.0),
        // );
        draw_line(
            ctx,
            self.start,
            self.start
                + self.direction.normalised()
                    * f32::sqrt(
                        (SCREEN_WIDTH * SCREEN_WIDTH + SCREEN_HEIGHT * SCREEN_HEIGHT) as f32,
                    ),
            4.0,
            Color::from_rgb(0, 255, 150),
        );
    }
}
