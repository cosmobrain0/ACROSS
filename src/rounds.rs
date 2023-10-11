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
            time_to_next_shot: Self::time_between_enemies(round_number),
            enemies: RefCell::new(vec![]),
            bullets: RefCell::new(vec![]),
        }
    }

    pub fn next(&mut self) {
        *self = Round::new(self.round_number + 1);
    }

    /// maybe just make this take &self?
    pub fn time_between_enemies(round_number: usize) -> usize {
        60 * round_number
    }

    pub fn update(
        &mut self,
        path: &Web,
        towers: &mut Vec<Box<dyn Tower>>,
        size: (f32, f32),
    ) -> usize {
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
            self.time_to_next_shot = Self::time_between_enemies(self.round_number);
        }

        let enemies = Enemy::update_all(self.enemies.replace(Vec::new()));
        let lives_lost = self.enemies.borrow().len() - enemies.len();
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
        lives_lost
    }

    pub fn enemies<'b>(&'b self) -> core::cell::Ref<'b, Vec<Enemy<'a, Alive>>> {
        self.enemies.borrow()
    }

    pub fn bullets<'b>(&'b self) -> core::cell::Ref<'b, Vec<Bullet<'a, Alive>>> {
        self.bullets.borrow()
    }

    pub fn complete(&self) -> bool {
        self.enemies_left == 0 && self.enemies.borrow().len() == 0
    }
}
