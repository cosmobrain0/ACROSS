use std::cell::RefCell;

use ggez::{graphics::Color, Context};

use crate::{
    bullet::{Bullet, BulletTrait, Projectile},
    enemy::Enemy,
    renderer::draw_circle,
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

pub fn spawn_tower<'a>(position: Vector) -> Box<dyn Tower<'a> + 'a> {
    TestTower::spawn(position)
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