use ggez::{graphics::Color, Context};

use crate::{
    enemy::Enemy,
    renderer::draw_circle,
    tower::{aim_towards, Tower},
    vector::Vector,
    Alive, Dead, Updated,
};

pub trait BulletTrait<'a>: std::fmt::Debug {
    /// create a new bullet
    fn spawn(
        tower: &impl Tower<'a>,
        target: Vector,
        target_velocity: Vector,
    ) -> Box<dyn BulletTrait<'a> + 'a>
    where
        Self: Sized;
    fn tower(&self) -> &'a dyn Tower;
    fn update<'b>(
        &mut self,
        enemies: Vec<Enemy<'b, Alive>>,
        bounds: Vector,
    ) -> (bool, Vec<Enemy<'b, Alive>>);
    fn draw(&self, ctx: &mut Context);
}

/// A lot of this is copied from enemy.js. Is there a way to reduce repetition?
#[derive(Debug)]
pub struct Bullet<'a, State> {
    bullet: Box<dyn BulletTrait<'a> + 'a>,
    state: std::marker::PhantomData<State>,
}

type UpdatedBullet<'a> = Updated<Bullet<'a, Alive>, Bullet<'a, Dead>>;

impl<'a> Bullet<'a, Alive> {
    pub fn new(bullet: Box<dyn BulletTrait<'a> + 'a>) -> Bullet<'a, Alive> {
        Bullet {
            bullet,
            state: std::marker::PhantomData::<Alive>,
        }
    }

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

    pub fn draw(&self, ctx: &mut Context) {
        self.bullet.draw(ctx);
    }
}

#[derive(Debug, Clone)]
pub struct Projectile {
    position: Vector,
    velocity: Vector,
    radius: f32,
}

impl<'a> BulletTrait<'a> for Projectile {
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
            // velocity: (target - tower.position()).normalised() * 3.0,
            velocity: aim_towards(tower.position(), target, target_velocity, 3.0),
            radius: 5.0,
        }) as Box<dyn BulletTrait<'a> + 'a>
    }

    fn tower(&self) -> &'a dyn Tower {
        todo!()
    }

    fn update<'b>(
        &mut self,
        mut enemies: Vec<Enemy<'b, Alive>>,
        bounds: Vector,
    ) -> (bool, Vec<Enemy<'b, Alive>>) {
        self.position += self.velocity;

        let mut new_enemies = Vec::with_capacity(enemies.len());
        let mut alive = true;
        while let Some(enemy) = enemies.pop() {
            if enemy.collides(self.position, self.radius) {
                // TODO: damage enemy instead of just killing it
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

    fn draw(&self, ctx: &mut Context) {
        draw_circle(
            ctx,
            self.position,
            self.radius,
            Color::new(0.0, 1.0, 0.5, 1.0),
        );
    }
}
