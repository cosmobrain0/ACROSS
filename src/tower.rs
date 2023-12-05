use std::{cell::RefCell, f32::consts::PI};

use ggez::{graphics::Color, Context};

use crate::{
    bullet::{Bullet, BulletTrait, Lightning, Projectile},
    collisions::{
        line_circle_collision, line_sector_collision, point_circle_collision,
        point_sector_collision, LineCircleCollision,
    },
    enemy::Enemy,
    renderer::{draw_circle, draw_sector},
    vec2d,
    vector::Vector,
    Alive,
};
use std::fmt::Debug;

/// Represents any kind of tower
pub trait Tower<'t>: Debug {
    /// This is how much this tower costs (money)
    fn price() -> usize
    where
        Self: Sized;
    /// How long this tower has until it shoots (frames)
    fn time_until_shot(&self) -> f32;
    /// Update the tower and its bullets
    fn update<'a>(
        &mut self,
        enemies: Vec<Enemy<'a, Alive>>,
        bounds: Vector,
    ) -> Vec<Enemy<'a, Alive>>;
    /// Draw the tower to the screen
    fn draw(&self, ctx: &mut Context);
    /// Get the position of the tower
    fn position(&self) -> Vector;
    /// Get the radius of the tower
    fn radius(&self) -> f32;
    /// Get the `Range` of the tower
    fn range(&self) -> &dyn Range;
    /// Spawn a new tower at a given position, facing a given direction
    fn spawn(position: Vector, direction: f32) -> Box<dyn Tower<'t> + 't>
    where
        Self: Sized;
    /// Get the living bullets of this tower
    fn bullets(&self) -> &RefCell<Vec<Bullet<'t, Alive>>>;
    /// Returns the length of the line segment from a to b
    /// which can be seen by this tower
    fn visible_area(&self, a: Vector, b: Vector) -> f32 {
        self.range().line_intersection(a, b)
    }
    /// Returns a new range which can be used for stuff
    fn new_range(position: Vector, direction: f32) -> Box<dyn Range>
    where
        Self: Sized;
}

/// The view of a tower
pub trait Range: Debug {
    /// Draw the range to the screen
    fn draw(&self, ctx: &mut Context);
    /// Choose an enemy which this range can see, to shoot at
    fn get_target<'a, 'b>(&self, enemies: &'b [Enemy<'a, Alive>]) -> Option<&'b Enemy<'a, Alive>>
    where
        'a: 'b;
    /// Returns the length of line segment which this range can see
    fn line_intersection(&self, a: Vector, b: Vector) -> f32;
    fn set_position(&mut self, position: Vector);
    fn set_direction(&mut self, direction: f32);
}
/// Represents a circular range
#[derive(Debug)]
pub struct CircularRange {
    position: Vector,
    radius: f32,
}
/// See docs for `Range`
impl Range for CircularRange {
    fn draw(&self, ctx: &mut Context) {
        draw_circle(
            ctx,
            self.position,
            self.radius,
            Color::from_rgba(255, 255, 255, 20),
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

    fn line_intersection(&self, a: Vector, b: Vector) -> f32 {
        let collisions = line_circle_collision(self.position, self.radius, a, b);
        match collisions {
            LineCircleCollision::None | LineCircleCollision::One(_) => 0.0,
            LineCircleCollision::Two(a, b) => (a - b).length(),
        }
    }

    fn set_position(&mut self, position: Vector) {
        self.position = position;
    }

    fn set_direction(&mut self, direction: f32) {}
}

/// Represents a range in the shape of a sector
#[derive(Debug)]
pub struct SectorRange {
    position: Vector,
    radius: f32,
    /// The direction in which this sector faces
    direction: f32,
    /// The field of view (an angle) of this range
    fov: f32,
}
/// See docs for `Range`
impl Range for SectorRange {
    fn draw(&self, ctx: &mut Context) {
        draw_sector(
            ctx,
            self.position,
            self.radius,
            self.direction - self.fov / 2.0,
            self.direction + self.fov / 2.0,
            200,
            Color::from_rgba(255, 255, 255, 20),
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
                if angle.abs() <= self.fov / 2.0 {
                    return Some(enemy);
                }
            }
        }
        None
    }

    fn line_intersection(&self, a: Vector, b: Vector) -> f32 {
        let collisions =
            line_sector_collision(self.position, self.radius, self.direction, self.fov, a, b);
        assert!(collisions.len() <= 2);
        match collisions.len() {
            0 | 1 => 0.0,
            2 => (collisions[0] - collisions[1]).length(),
            _ => unreachable!("Line/Sector collision shouldn't return more than two points!"),
        }
    }

    fn set_position(&mut self, position: Vector) {
        self.position = position;
    }

    fn set_direction(&mut self, direction: f32) {
        self.direction = direction;
    }
}

/// Returns the signed shortest angle between two angles
pub fn shortest_angle_distance(theta1: f32, theta2: f32) -> f32 {
    let distance = if theta2 > theta1 {
        (theta2 - theta1) % (2.0 * PI)
    } else {
        (theta1 - theta2) % (2.0 * PI)
    };
    (if distance.abs() > PI {
        (2.0 * PI - distance.abs()) * -distance.signum()
    } else {
        distance
    }) * (theta2 - theta1).signum()
}

/// Represents a simple tower with a circular range
#[derive(Debug)]
pub struct TestTower<'t> {
    time_to_next_shot: usize,
    position: Vector,
    bullets: RefCell<Vec<Bullet<'t, Alive>>>,
    range: CircularRange,
}
impl<'t> TestTower<'t> {
    /// The time between shots for this tower (frames)
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
/// See docs for `Tower`
impl<'t> Tower<'t> for TestTower<'t> {
    #[inline(always)]
    fn price() -> usize
    where
        Self: Sized,
    {
        25
    }

    fn update<'b>(
        &mut self,
        enemies: Vec<Enemy<'b, Alive>>,
        bounds: Vector,
    ) -> Vec<Enemy<'b, Alive>> {
        if self.time_to_next_shot == 0 {
            // shoot!
            if let Some(enemy) = self.range.get_target(&enemies) {
                self.bullets.borrow_mut().push(Bullet::new(Lightning::spawn(
                    self,
                    enemy.position(),
                    enemy.velocity(),
                )));
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

    fn spawn(position: Vector, _direction: f32) -> Box<dyn Tower<'t> + 't>
    where
        Self: Sized,
    {
        Box::new(Self::new(position)) as Box<dyn Tower + 't>
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

    fn new_range(position: Vector, _direction: f32) -> Box<dyn Range>
    where
        Self: Sized,
    {
        Box::new(CircularRange {
            position,
            radius: 150.0,
        }) as Box<dyn Range>
    }
}

/// A simple tower with a range in the shape of a sector
#[derive(Debug)]
pub struct SectorTower<'t> {
    time_to_next_shot: usize,
    position: Vector,
    bullets: RefCell<Vec<Bullet<'t, Alive>>>,
    range: SectorRange,
}
impl<'t> SectorTower<'t> {
    /// The time between shots for this tower
    #[inline(always)]
    pub fn cooldown() -> usize {
        20
    }

    pub fn new(position: Vector, direction: f32) -> Self {
        Self {
            time_to_next_shot: Self::cooldown(),
            position,
            bullets: RefCell::new(vec![]),
            range: SectorRange {
                position,
                radius: 200.0,
                direction,
                fov: PI / 2.0,
            },
        }
    }
}
/// See docs for `Tower`
impl<'t> Tower<'t> for SectorTower<'t> {
    fn price() -> usize
    where
        Self: Sized,
    {
        15
    }

    fn time_until_shot(&self) -> f32 {
        self.time_to_next_shot as f32
    }

    fn update<'a>(
        &mut self,
        enemies: Vec<Enemy<'a, Alive>>,
        bounds: Vector,
    ) -> Vec<Enemy<'a, Alive>> {
        if self.time_to_next_shot == 0 {
            if let Some(enemy) = self.range.get_target(&enemies) {
                self.bullets
                    .borrow_mut()
                    .push(Bullet::new(Projectile::spawn(
                        self,
                        enemy.position(),
                        enemy.velocity(),
                    )));
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

    fn spawn(position: Vector, direction: f32) -> Box<dyn Tower<'t> + 't>
    where
        Self: Sized,
    {
        Box::new(SectorTower::new(position, direction)) as Box<dyn Tower<'t> + 't>
    }

    fn bullets(&self) -> &RefCell<Vec<Bullet<'t, Alive>>> {
        &self.bullets
    }

    fn new_range(position: Vector, direction: f32) -> Box<dyn Range>
    where
        Self: Sized,
    {
        Box::new(SectorRange {
            position,
            radius: 200.0,
            direction,
            fov: PI / 2.0,
        }) as Box<dyn Range>
    }
}

/// Returns the velocity with which a projectile should be shot
/// to hit a target which is currently at `target_position`
/// and has a constant velocity of `target_velocity`
/// if the projectile will be shot from `start`
/// with a speed of `projectile_speed`
pub fn aim_towards(
    start: Vector,
    target_position: Vector,
    target_velocity: Vector,
    projectile_speed: f32,
) -> Vector {
    // sin(alpha)/kd_E = sin(theta)/d_E
    let k = projectile_speed / target_velocity.length();
    let alpha = PI - (target_position - start).angle_between(target_velocity);
    let sin_theta = alpha.sin() / k;

    // o^2 = x^2(1 + k^2 - 2kcos(theta))
    // o^2 / x^2 > 0, 1 + k^2 - 2kcos(theta) > 0
    // (1+k^2)/2k > cos(theta)
    let cos_theta = 1.0 - sin_theta * sin_theta;
    let cos_theta = [cos_theta.sqrt(), -cos_theta.sqrt()]
        .into_iter()
        .filter(|&cos_theta| cos_theta < (1.0 + k * k) / (2.0 * k));

    cos_theta
        .map(|cos_theta| vec2d![cos_theta, sin_theta] * projectile_speed)
        .map(|velocity| {
            if (target_position - start)
                .clockwise_90deg()
                .dot(target_velocity)
                > 0.0
            {
                velocity.rotate(-velocity.angle() * 2.0)
            } else {
                velocity
            }
        })
        .map(|velocity| velocity.rotate((target_position - start).angle()))
        .next()
        .expect("Some valid solutions")
}
