use crate::{
    enemy::{EnemyTrait, TestEnemy},
    path::{Route, Web},
    vector::Vector,
    SCREEN_HEIGHT, SCREEN_WIDTH,
};
use std::cell::RefCell;

use crate::{bullet::Bullet, enemy::Enemy, tower::Tower, vec2d, Alive};

pub struct Round<'a> {
    enemies_left: usize,
    round_number: usize,
    time_to_next_shot: usize,
    enemies: RefCell<Vec<Enemy<'a, Alive>>>,
    bullets: RefCell<Vec<Bullet<'a, Alive>>>,
}

impl<'a> Round<'a> {
    pub fn new(round_number: usize) -> Self {
        Self {
            enemies_left: 10,
            round_number,
            time_to_next_shot: Self::time_between_enemies(),
            enemies: RefCell::new(vec![]),
            bullets: RefCell::new(vec![]),
        }
    }

    pub fn time_between_enemies() -> usize {
        60
    }

    pub fn update(&mut self, path: &Web, towers: &mut Vec<Box<dyn Tower>>, size: (f32, f32)) {
        self.time_to_next_shot = self.time_to_next_shot.saturating_sub(1);
        if self.time_to_next_shot == 0 {
            if self.enemies_left > 0 {
                self.enemies
                    .borrow_mut()
                    .push(TestEnemy::spawn(Route::from_positions_unchecked(
                        path.route(),
                    )));
                self.enemies_left -= 1;
            }
            self.time_to_next_shot = Self::time_between_enemies();
        }

        let enemies = Enemy::update_all(self.enemies.replace(Vec::new()));
        self.enemies.replace(enemies);
        let (bullets, mut enemies) = Bullet::update_all(
            self.bullets.replace(Vec::new()),
            self.enemies.replace(Vec::new()),
            vec2d![size.0, size.1],
        );
        for tower in towers.iter_mut() {
            enemies = tower.update(enemies, vec2d![SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32]);
        }
        self.bullets.replace(bullets);
        self.enemies.replace(enemies);
    }

    pub fn enemies<'b>(&'b self) -> core::cell::Ref<'b, Vec<Enemy<'a, Alive>>> {
        self.enemies.borrow()
    }

    pub fn bullets<'b>(&'b self) -> core::cell::Ref<'b, Vec<Bullet<'a, Alive>>> {
        self.bullets.borrow()
    }
}
