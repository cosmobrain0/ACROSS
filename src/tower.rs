pub mod tower {
    use std::{cell::RefCell, f32::consts::PI};

    use ggez::{graphics::Color, Context};
    use rand::random;

    use crate::{
        bullet::bullet::{Bullet, BulletTrait, Projectile},
        enemy::enemy::Enemy,
        renderer::draw_circle,
        vec2d,
        vector::Vector,
        Alive, Updated,
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
        fn range<'a>(&'a self) -> &dyn Range;
        fn spawn(bounds: Vector) -> Box<dyn Tower<'t> + 't>
        where
            Self: Sized;
        fn bullets(&self) -> &RefCell<Vec<Bullet<'t, Alive>>>;
    }

    /// The view of a tower
    pub trait Range {
        fn draw(&self, ctx: &mut Context);
    }
    pub struct CircularRange {
        position: Vector,
    }
    impl Range for CircularRange {
        fn draw(&self, ctx: &mut Context) {
            draw_circle(
                ctx,
                self.position,
                60.0,
                Color::from_rgba(255, 255, 255, 100),
            );
        }
    }

    pub fn spawn_tower<'a>(bounds: Vector) -> Box<dyn Tower<'a> + 'a> {
        TestTower::spawn(bounds)
    }

    pub struct TestTower<'t> {
        time_to_next_shot: usize,
        position: Vector,
        bullets: RefCell<Vec<Bullet<'t, Alive>>>,
        range: CircularRange,
    }
    impl TestTower<'_> {
        #[inline(always)]
        fn cooldown() -> usize {
            60
        }

        fn new(position: Vector) -> Self {
            Self {
                time_to_next_shot: Self::cooldown(),
                position,
                bullets: RefCell::new(vec![]),
                range: CircularRange { position },
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
            match self.time_to_next_shot {
                0 => {
                    // shoot!
                    self.bullets
                        .borrow_mut()
                        .push(Bullet::new(Projectile::spawn(
                            self,
                            self.position + Vector::from_polar(random::<f32>() * 2.0 * PI, 4.0),
                        )));
                    self.time_to_next_shot = TestTower::cooldown();
                }

                _ => self.time_to_next_shot -= 1,
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

        fn spawn(bounds: Vector) -> Box<dyn Tower<'t> + 't> {
            Box::new(Self::new(vec2d![
                random::<f32>() * bounds.x,
                random::<f32>() * bounds.y
            ])) as Box<dyn Tower + 't>
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

        fn range<'b>(&'b self) -> &dyn Range {
            &self.range as &dyn Range
        }

        fn bullets(&self) -> &RefCell<Vec<Bullet<'t, Alive>>> {
            &self.bullets
        }
    }
}
