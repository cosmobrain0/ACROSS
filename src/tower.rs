use std::{cell::RefCell, f32::consts::PI};

use ggez::{graphics::Color, Context};

use crate::{
    bullet::{Bullet, BulletTrait, Projectile},
    enemy::Enemy,
    renderer::{draw_circle, draw_sector},
    vec2d,
    vector::Vector,
    Alive,
};

pub trait Tower<'t> {
    fn price(&self) -> u64;
    fn time_until_shot(&self) -> f32;
    fn update<'a>(
        &mut self,
        enemies: Vec<Enemy<'a, Alive>>,
        bounds: Vector,
    ) -> Vec<Enemy<'a, Alive>>;
    fn draw(&self, ctx: &mut Context);
    fn position(&self) -> Vector;
    fn radius(&self) -> f32;
    fn range(&self) -> &dyn Range;
    fn spawn(bounds: Vector) -> Box<dyn Tower<'t> + 't>
    where
        Self: Sized;
    fn bullets(&self) -> &RefCell<Vec<Bullet<'t, Alive>>>;
}

/// The view of a tower
pub trait Range {
    fn draw(&self, ctx: &mut Context);
    fn get_target<'a, 'b>(&self, enemies: &'b [Enemy<'a, Alive>]) -> Option<&'b Enemy<'a, Alive>>
    where
        'a: 'b;
}
pub struct CircularRange {
    position: Vector,
    radius: f32,
}
impl Range for CircularRange {
    fn draw(&self, ctx: &mut Context) {
        draw_circle(
            ctx,
            self.position,
            self.radius,
            Color::from_rgba(255, 255, 255, 100),
        );
    }

    fn get_target<'a, 'b>(&self, enemies: &'b [Enemy<'a, Alive>]) -> Option<&'b Enemy<'a, Alive>>
    where
        'a: 'b,
    {
        enemies
            .iter()
            .find(|&enemy| enemy.collides(self.position, self.radius))
    }
}

pub struct SectorRange {
    position: Vector,
    radius: f32,
    /// The direction in which this sector faces
    direction: f32,
    /// The field of view (an angle) of this range
    fov: f32,
}
impl Range for SectorRange {
    fn draw(&self, ctx: &mut Context) {
        draw_sector(
            ctx,
            self.position,
            self.radius,
            self.direction - self.fov / 2.0,
            self.direction + self.fov / 2.0,
            200,
            Color::from_rgba(255, 255, 255, 100),
        );
    }

    fn get_target<'a, 'b>(&self, enemies: &'b [Enemy<'a, Alive>]) -> Option<&'b Enemy<'a, Alive>>
    where
        'a: 'b,
    {
        for enemy in enemies {
            if enemy.collides(self.position, self.radius) {
                let angle = shortest_angle_distance(
                    (enemy.position() - self.position).angle(),
                    self.direction,
                );
                if angle.abs() <= self.fov {
                    return Some(enemy);
                }
            }
        }
        None
    }
}

/// Returns the signed shortest angle between two angles
pub fn shortest_angle_distance(theta1: f32, theta2: f32) -> f32 {
    let mut distance = (theta2 - theta1) % (2.0 * PI);
    distance = if distance < 0.0 {
        2.0 * PI - distance
    } else {
        distance
    };
    let (sign, shortest_distance) = if distance > PI {
        (theta2 < theta1, distance - PI)
    } else {
        (theta2 > theta1, distance)
    };
    let sign = if sign { 1.0 } else { -1.0 };
    shortest_distance * sign
}

pub fn spawn_tower<'a>(position: Vector) -> Box<dyn Tower<'a> + 'a> {
    SectorTower::spawn(position)
}

pub struct TestTower<'t> {
    time_to_next_shot: usize,
    position: Vector,
    bullets: RefCell<Vec<Bullet<'t, Alive>>>,
    range: CircularRange,
}
impl<'t> TestTower<'t> {
    #[inline(always)]
    fn cooldown() -> usize {
        60
    }

    fn new(position: Vector) -> Self {
        Self {
            time_to_next_shot: Self::cooldown(),
            position,
            bullets: RefCell::new(vec![]),
            range: CircularRange {
                position,
                radius: 150.0,
            },
        }
    }
}
impl<'t> Tower<'t> for TestTower<'t> {
    #[inline(always)]
    fn price(&self) -> u64 {
        10
    }

    fn update<'b>(
        &mut self,
        enemies: Vec<Enemy<'b, Alive>>,
        bounds: Vector,
    ) -> Vec<Enemy<'b, Alive>> {
        if self.time_to_next_shot == 0 {
            // shoot!
            if let Some(enemy) = self.range.get_target(&enemies) {
                self.bullets
                    .borrow_mut()
                    .push(Bullet::new(Projectile::spawn(self, enemy.position())));
                self.time_to_next_shot = TestTower::cooldown();
            }
        } else {
            self.time_to_next_shot -= 1
        }
        let bullets = Vec::with_capacity(self.bullets.borrow().len());
        let bullets = self.bullets.replace(bullets);
        let (new_bullets, new_enemies) = Bullet::update_all(bullets, enemies, bounds);
        self.bullets.replace(new_bullets);
        new_enemies
    }

    fn draw(&self, ctx: &mut Context) {
        self.range.draw(ctx);
        self.bullets.borrow().iter().for_each(|x| x.draw(ctx));
        draw_circle(
            ctx,
            self.position(),
            self.radius(),
            Color::from_rgb(255, 255, 255),
        );
    }

    fn spawn(position: Vector) -> Box<dyn Tower<'t> + 't> {
        Box::new(Self::new(vec2d![position.x, position.y])) as Box<dyn Tower + 't>
    }

    fn time_until_shot(&self) -> f32 {
        self.time_to_next_shot as f32
    }

    fn position(&self) -> Vector {
        self.position
    }

    fn radius(&self) -> f32 {
        10.0
    }

    fn range(&self) -> &dyn Range {
        &self.range as &dyn Range
    }

    fn bullets(&self) -> &RefCell<Vec<Bullet<'t, Alive>>> {
        &self.bullets
    }
}

pub struct SectorTower<'t> {
    time_to_next_shot: usize,
    position: Vector,
    bullets: RefCell<Vec<Bullet<'t, Alive>>>,
    range: SectorRange,
}
impl<'t> SectorTower<'t> {
    #[inline(always)]
    pub fn cooldown() -> usize {
        20
    }

    pub fn new(position: Vector) -> Self {
        Self {
            time_to_next_shot: Self::cooldown(),
            position,
            bullets: RefCell::new(vec![]),
            range: SectorRange {
                position,
                radius: 200.0,
                direction: 0.0,
                fov: PI / 2.0,
            },
        }
    }
}
impl<'t> Tower<'t> for SectorTower<'t> {
    fn price(&self) -> u64 {
        20
    }

    fn time_until_shot(&self) -> f32 {
        self.time_to_next_shot as f32
    }

    fn update<'a>(
        &mut self,
        enemies: Vec<Enemy<'a, Alive>>,
        bounds: Vector,
    ) -> Vec<Enemy<'a, Alive>> {
        // TODO: lots of repetition here.
        // can that be cut down?
        if self.time_to_next_shot == 0 {
            // shoot!
            if let Some(enemy) = self.range.get_target(&enemies) {
                self.bullets
                    .borrow_mut()
                    .push(Bullet::new(Projectile::spawn(self, enemy.position())));
                self.time_to_next_shot = Self::cooldown();
            }
        } else {
            self.time_to_next_shot -= 1
        }
        let bullets = Vec::with_capacity(self.bullets.borrow().len());
        let bullets = self.bullets.replace(bullets);
        let (new_bullets, new_enemies) = Bullet::update_all(bullets, enemies, bounds);
        self.bullets.replace(new_bullets);
        new_enemies
    }

    fn draw(&self, ctx: &mut Context) {
        self.range.draw(ctx);
        self.bullets.borrow().iter().for_each(|x| x.draw(ctx));
        draw_circle(
            ctx,
            self.position(),
            self.radius(),
            Color::from_rgb(255, 255, 255),
        );
    }

    fn position(&self) -> Vector {
        self.position
    }

    fn radius(&self) -> f32 {
        10.0
    }

    fn range(&self) -> &dyn Range {
        &self.range as &dyn Range
    }

    fn spawn(position: Vector) -> Box<dyn Tower<'t> + 't>
    where
        Self: Sized,
    {
        Box::new(SectorTower::new(position)) as Box<dyn Tower<'t> + 't>
    }

    fn bullets(&self) -> &RefCell<Vec<Bullet<'t, Alive>>> {
        &self.bullets
    }
}
