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
}

impl<'a> Round<'a> {
    pub fn new(round_number: usize) -> Self {
        Self {
            enemies_left: 10,
            round_number,
            time_to_next_shot: Self::time_between_enemies(round_number),
            enemies: RefCell::new(vec![]),
        }
    }

    pub fn next(&mut self) {
        *self = Round::new(self.round_number + 1);
    }

    /// maybe just make this take &self?
    pub fn time_between_enemies(round_number: usize) -> usize {
        120 / (round_number + 1)
    }

    pub fn update(
        &mut self,
        path: &Web,
        towers: &mut Vec<Box<dyn Tower>>,
        size: (f32, f32),
    ) -> (usize, usize) {
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

        let initial_enemies = self.enemies.borrow().len();
        self.enemies
            .replace(Enemy::update_all(self.enemies.replace(Vec::new())));
        let enemies_after_movement = self.enemies.borrow().len();

        for tower in towers.iter_mut() {
            self.enemies.replace(tower.update(
                self.enemies.replace(vec![]),
                vec2d![SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32],
            ));
        }
        let enemies_after_shooting = self.enemies.borrow().len();

        let lives_lost = initial_enemies - enemies_after_movement;
        let enemies_killed = enemies_after_movement - enemies_after_shooting;
        (lives_lost, enemies_killed)
    }

    pub fn enemies<'b>(&'b self) -> core::cell::Ref<'b, Vec<Enemy<'a, Alive>>> {
        self.enemies.borrow()
    }

    pub fn complete(&self) -> bool {
        self.enemies_left == 0 && self.enemies.borrow().len() == 0
    }
}
