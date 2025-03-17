#![no_std]

use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::{println, serial_println, vga_buffer::{plot, plot_str, plot_num, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH}};
use rand::{Rng, SeedableRng, rngs::SmallRng};

use core::{
    clone::Clone,
    marker::Copy,
    prelude::rust_2024::derive,
};

    pub struct Room {
        pub width: usize,
        pub height: usize,
        pub x: usize,
        pub y: usize,
        pub doors: [(usize, usize); 4], // Up to 4 doors
        pub locked: bool, // Doors lock when enemies are present
        pub seed: SmallRng,

    }

    impl Room {
        pub fn new(width: usize, height: usize) -> Self {
            let x = (BUFFER_WIDTH - width) / 2;
            let y = (BUFFER_HEIGHT - height) / 2;

            let mut seed = SmallRng::seed_from_u64(unsafe { core::arch::x86_64::_rdtsc() });

            let mut doors = [(0, 0); 4];
            let door_count = seed.gen_range(2..=4);

            for i in 0..door_count {
                match seed.gen_range(0..4) {
                    0 => doors[i] = (x + seed.gen_range(1..width - 1), y),           // Top wall
                    1 => doors[i] = (x + seed.gen_range(1..width - 1), y + height - 1), // Bottom wall
                    2 => doors[i] = (x, y + seed.gen_range(1..height - 1)),           // Left wall
                    3 => doors[i] = (x + width - 1, y + seed.gen_range(1..height - 1)), // Right wall
                    _ => {}
                }
            }

            Self { width, height, x, y, doors, locked: false, seed}
        }

        pub fn draw(&self) {
            for col in self.x..self.x + self.width {
                plot('#', col, self.y, ColorCode::new(Color::White, Color::Black));
                plot('#', col, self.y + self.height - 1, ColorCode::new(Color::White, Color::Black));
            }

            for row in self.y..self.y + self.height {
                plot('#', self.x, row, ColorCode::new(Color::White, Color::Black));
                plot('#', self.x + self.width - 1, row, ColorCode::new(Color::White, Color::Black));
            }

            for row in (self.y + 1)..(self.y + self.height - 1) {
                for col in (self.x + 1)..(self.x + self.width - 1) {
                    plot('.', col, row, ColorCode::new(Color::DarkGray, Color::Black));
                }
            }

            let door_color = if self.locked { Color::Red } else { Color::Green };

            for &(dx, dy) in &self.doors {
                if dx != 0 || dy != 0 {
                    plot('+', dx, dy, ColorCode::new(door_color, Color::Black));
                }
            }
        }

        pub fn clear(&self) {
            for col in self.x..self.x + self.width {
                plot(' ', col, self.y, ColorCode::new(Color::White, Color::Black));
                plot(' ', col, self.y + self.height - 1, ColorCode::new(Color::White, Color::Black));
            }

            for row in self.y..self.y + self.height {
                plot(' ', self.x, row, ColorCode::new(Color::White, Color::Black));
                plot(' ', self.x + self.width - 1, row, ColorCode::new(Color::White, Color::Black));
            }

            for row in (self.y + 1)..(self.y + self.height - 1) {
                for col in (self.x + 1)..(self.x + self.width - 1) {
                    plot(' ', col, row, ColorCode::new(Color::DarkGray, Color::Black));
                }
            }
        }

        pub fn is_wall(&self, x: usize, y: usize) -> bool {
            x == self.x || x == self.x + self.width - 1 || y == self.y || y == self.y + self.height - 1
        }

        pub fn is_door(&self, x: usize, y: usize) -> bool {
            for &(dx, dy) in &self.doors {
                if (x, y) == (dx, dy) || (x + 1, y) == (dx, dy) || (x, y + 1) == (dx, dy) || (x + 1, y + 1) == (dx, dy){
                    return true;
                    //println!("thats a fuckin door")
                }
            }
            false
        }

        pub fn unlock(&mut self) {
            self.locked = false;
        }
    }

    pub struct Player {
        pub x: usize,
        pub y: usize,
        pub health: usize,
        pub last_hit:usize,
        pub timer: usize,
        pub bullets: [Bullet; 10],
        pub i_bullet: usize,
    }

    impl Player {
        pub fn key(&mut self, key: DecodedKey, game_state: &mut GameState) {
            match key {
                DecodedKey::RawKey(code) => self.handle_raw(code, game_state),
                DecodedKey::Unicode(c) => self.handle_unicode(c, game_state),
            }
        }

        pub fn update(&mut self) {
            self.timer();
        }

        pub fn timer(&mut self) {
            self.timer += 1;
        }

        fn handle_unicode(&mut self, key: char, game_state: &mut GameState) {
            match key {
                'w' => {
                    self.shoot(0, -1); // Shoot Up
                }
                'a' => {
                    self.shoot(-1, 0); // Shoot Left
                }
                's' => {
                    self.shoot(0, 1); // Shoot Down
                }
                'd' => {
                    self.shoot(1, 0); // Shoot Right
                }
                'r' => {
                    game_state.restart(self);
                }
                _ => {}
            }
        }

        fn handle_raw(&mut self, key: KeyCode, game_state: &mut GameState) {
            match key {
                KeyCode::ArrowLeft => {
                    self.move_to(self.x.saturating_sub(1), self.y, game_state);
                }
                KeyCode::ArrowRight => {
                    self.move_to(self.x + 1, self.y, game_state);
                }
                KeyCode::ArrowUp => {
                    self.move_to(self.x, self.y.saturating_sub(1), game_state);
                }
                KeyCode::ArrowDown => {
                    self.move_to(self.x, self.y + 1, game_state);
                }
                _ => {}
            }
        }

        pub fn shoot(&mut self, dx: isize, dy:isize) {
            if self.i_bullet < 10 {
                self.bullets[self.i_bullet] = Bullet::new(self.x, self.y, dx, dy, true); // Ensure bullet is active
                self.i_bullet += 1;
            } else {
                self.i_bullet = 0;
            }
        }

        pub fn update_bullets(&mut self, game_state: &mut GameState) {

            for bullet in &mut self.bullets {
                bullet.clear();
                bullet.move_forward(game_state);
                bullet.draw();
            }

        }

        pub fn new(room: &Room) -> Self {
            Self {
                x: room.x + room.width / 2,
                y: room.y + room.height / 2,
                health: 4,
                last_hit: 0,
                timer: 0,
                bullets: [Bullet::new(0, 0, 0, 0, false); 10],
                i_bullet: 0,
            }
        }

        pub fn draw(&self) {
            plot(':', self.x, self.y, ColorCode::new(Color::Yellow, Color::Black));
            plot('3', self.x + 1, self.y, ColorCode::new(Color::Yellow, Color::Black));
            self.drawhealth()
        }

        pub fn drawhealth(&self) {
            let mut temp: usize = 2 * self.health;
            for i in 1..=16 {
                plot(' ', i, 1, ColorCode::new(Color::Black, Color::Black));
            }
            plot('H', 1, 1, ColorCode::new(Color::Red, Color::Black));
            plot('E', 2, 1, ColorCode::new(Color::Red, Color::Black));
            plot('A', 3, 1, ColorCode::new(Color::Red, Color::Black));
            plot('L', 4, 1, ColorCode::new(Color::Red, Color::Black));
            plot('T', 5, 1, ColorCode::new(Color::Red, Color::Black));
            plot('H', 6, 1, ColorCode::new(Color::Red, Color::Black));
            plot(':', 7, 1, ColorCode::new(Color::Red, Color::Black));
            for _num in (0)..(self.health) {
                plot('<', 7 + temp, 1, ColorCode::new(Color::Red, Color::Black));
                plot('3', 7 + temp + 1, 1, ColorCode::new(Color::Red, Color::Black));
                temp -= 2;
            }
        }

        pub fn move_to(&mut self, new_x: usize, new_y: usize, game_state: &mut GameState) {
            if game_state.active {
                let mut canmove = false;
                if game_state.current_room.is_door(new_x, new_y)  {
                    game_state.transition();
                    self.clear();
                }
                if !game_state.current_room.is_wall(new_x, new_y) && !game_state.current_room.is_wall(new_x + 1, new_y) {
                    for enemy in game_state.enemies {
                        if enemy.is_collision(new_x, new_y) || enemy.is_collision(new_x + 1, new_y) {
                            canmove = false;
                        } else {
                            canmove = true;
                        }
                    }
                    if canmove {
                        self.x = new_x;
                        self.y = new_y;
                    }
                }
                for enemy in game_state.enemies {
                    if enemy.is_collision(new_x, new_y) || enemy.is_collision(new_x + 1, new_y) {
                        if !enemy.dead {
                            if self.health > 0 && (self.timer - self.last_hit > 8){
                                self.health -= 1;
                                self.last_hit = self.timer;
                            }
                        }
                    }
                }
            }
        }

        pub fn is_collision(&self, x: usize, y: usize) -> bool {
            (x == self.x || x == self.x + 1) && y == self.y
        }

        pub fn clear(&mut self) {
            for bullet in &mut self.bullets {
                bullet.clear();
                bullet.active = false;
            }
        }
    }
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Mouse {
        pub x: usize,
        pub y: usize,
        pub dead: bool,
        pub active: bool,
    }

    impl Mouse {

        pub fn new(x: usize, y: usize, dead: bool, active: bool) -> Self {
            Self {
                dead: dead,
                x: x,
                y: y,
                active: active,
            }
        }

        pub fn die(&mut self) {
            self.dead = true;
        }

        pub fn draw(&self) {
            if !self.dead {
                plot('~', self.x, self.y, ColorCode::new(Color::Yellow, Color::Black));
                plot('o', self.x + 1, self.y, ColorCode::new(Color::Yellow, Color::Black));
            } else {
                plot('O', self.x, self.y, ColorCode::new(Color::Yellow, Color::Black));
                plot('X', self.x + 1, self.y, ColorCode::new(Color::Yellow, Color::Black));
            }

        }

        pub fn clear(&self) {
            if self.active {
                plot(' ', self.x, self.y, ColorCode::new(Color::Black, Color::Black));
            }
        }

        pub fn random_move(&mut self, room: &mut Room, player: &mut Player, timer: usize) {
            if !self.dead {
                if timer % 5 == 0 {
                    let (dx, dy) = match room.seed.gen_range(0..4) {
                        0 => (-1, 0),
                        1 => (1, 0),
                        2 => (0, -1),
                        3 => (0, 1),
                        _ => (0, 0),
                    };

                    let new_x = self.x.saturating_add_signed(dx);
                    let new_y = self.y.saturating_add_signed(dy);

                    self.move_to(new_x, new_y, room, player, timer);
                }
            }
        }

        pub fn move_to(&mut self, new_x: usize, new_y: usize, room: &Room, player: &mut Player, timer: usize) {
            if !room.is_wall(new_x, new_y) && !room.is_wall(new_x + 1, new_y) && !player.is_collision(new_x, new_y) && !player.is_collision(new_x + 1, new_y){
                self.x = new_x;
                self.y = new_y;
            }
            if player.is_collision(new_x, new_y) || player.is_collision(new_x + 1, new_y) {
                if self.dead {
                    return;
                }
                if player.health > 0 && (player.timer - player.last_hit > 8){
                    player.health -= 1;
                    player.last_hit = timer;
                }
            }
        }

        pub fn is_collision(&self, x: usize, y: usize) -> bool {
            (x == self.x || x == self.x + 1) && y == self.y
        }

    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Bullet {
        pub x: usize,
        pub y: usize,
        pub dx: isize, // Direction X (-1 left, +1 right, 0 no movement)
        pub dy: isize, // Direction Y (-1 up, +1 down, 0 no movement)
        pub active: bool,
    }

    impl Bullet {
        pub fn new(x: usize, y: usize, dx: isize, dy: isize, active: bool) -> Self {
            Self { x, y, dx, dy, active }
        }

        pub fn move_forward(&mut self, gamestate: &mut GameState) {
            if self.active {
                let new_x = self.x.saturating_add_signed(self.dx);
                let new_y = self.y.saturating_add_signed(self.dy);

                if gamestate.current_room.is_wall(new_x, new_y) {
                    self.active = false;
                } else {
                    self.x = new_x;
                    self.y = new_y;
                }

                for enemy in gamestate.enemies.iter_mut() {
                    if enemy.is_collision(self.x, self.y) {
                        if !enemy.dead {
                            self.active = false;
                            enemy.die();
                            gamestate.score += 100;
                        }
                    }
                }
            }
        }

        pub fn draw(&self) {
            if self.active {
                plot('*', self.x, self.y, ColorCode::new(Color::White, Color::Black));
            }
        }

        pub fn clear(&self) {
            if self.active {
                plot(' ', self.x, self.y, ColorCode::new(Color::Black, Color::Black));
            }
        }
    }

    pub struct GameState {
        pub current_room: Room,
        pub rng: SmallRng,
        pub timer: usize,
        pub enemies: [Mouse; 10],
        pub score: usize,
        pub active: bool,
    }

    impl GameState {
        pub fn new() -> Self {
            let rng = SmallRng::seed_from_u64(unsafe { core::arch::x86_64::_rdtsc() });
            let initial_room = Room::new(20, 20);
            let timer = 0;
            let enemies = [Mouse::new(10, 10, true, false); 10];
            let score = 0;
            let active = true;
            Self { current_room: initial_room, rng, timer, enemies, score, active}
        }

        pub fn transition(&mut self) {
            for enemy in self.enemies {
                enemy.clear();
            }
            self.current_room.clear();
            self.current_room = Room::new(20, 20);
            self.generate();
        }

        pub fn generate(&mut self) {
            self.enemies = [Mouse::new(10, 10, true, false); 10];
            for i in 0..self.rng.gen_range(3..10) {
                self.enemies[i] = Mouse::new(self.rng.gen_range(self.current_room.x+5..self.current_room.x+15),self.rng.gen_range(self.current_room.y+5..self.current_room.y+15), false, true);
            }
        }

        pub fn draw(&mut self) {
            let score_text = "Score:";
            plot_str(score_text, 30, 0, ColorCode::new(Color::Blue, Color::Black));
            plot_num(self.score as isize, 30 + score_text.len() + 1, 0, ColorCode::new(Color::Blue, Color::Black));
        }

        pub fn restart(&mut self, player: &mut Player) {
            if !self.active {
                player.health = 4;
                self.score = 0;
                self.timer = 0;
                self.current_room.clear();
                self.current_room = Room::new(20, 20);
                self.enemies = [Mouse::new(10, 10, true, false); 10];
                self.active = true;
                for i in 25..60{
                    plot(' ', i, 0, ColorCode::new(Color::Red, Color::Black));
                }
            }
        }

        pub fn update(&mut self, player: &mut Player) {
            if self.active {
                if player.health == 0 {
                    self.active = false;
                }
                for enemy in self.enemies.iter_mut() {
                    enemy.random_move(&mut self.current_room, player, self.timer);
                }
                self.current_room.draw();
                for enemy in self.enemies {
                    enemy.clear();
                    if enemy.active {
                        enemy.draw();
                    }
                }
                //println!("door {:?}, player {x}, {y}", self.current_room.doors[0], x = player.x, y = player.y);
                player.draw();
                self.draw();
                self.timer += 1;
                self.score += 1;
            } else {
                let score_text = "Game Over, press R to restart.";
                plot_str(score_text, 30, 0, ColorCode::new(Color::Red, Color::Black));
            }
        }
    }


