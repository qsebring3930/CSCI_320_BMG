#![no_std]

use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::{println, serial_println, vga_buffer::{plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH}};
use rand::{Rng, SeedableRng, rngs::SmallRng};

use core::{
    clone::Clone,
    marker::Copy,
    prelude::rust_2024::derive,
};

#[derive(Copy, Clone, Eq, PartialEq)]

    pub struct Room {
        pub width: usize,
        pub height: usize,
        pub x: usize,
        pub y: usize,
    }

    impl Room {
        pub fn new(width: usize, height: usize) -> Self {
            let x = (BUFFER_WIDTH - width) / 2;
            let y = (BUFFER_HEIGHT - height) / 2;
            Self { width, height, x, y }
        }

        pub fn draw(&self) {
            for col in self.x..self.x + self.width {
                plot('#', col, self.y, ColorCode::new(Color::White, Color::Black)); // Top wall
                plot('#', col, self.y + self.height - 1, ColorCode::new(Color::White, Color::Black)); // Bottom wall
            }

            for row in self.y..self.y + self.height {
                plot('#', self.x, row, ColorCode::new(Color::White, Color::Black)); // Left wall
                plot('#', self.x + self.width - 1, row, ColorCode::new(Color::White, Color::Black)); // Right wall
            }

            for row in (self.y + 1)..(self.y + self.height - 1) {
                for col in (self.x + 1)..(self.x + self.width - 1) {
                    plot('.', col, row, ColorCode::new(Color::DarkGray, Color::Black)); // Floor
                }
            }
        }

        pub fn is_wall(&self, x: usize, y: usize) -> bool {
            x == self.x || x == self.x + self.width - 1 || y == self.y || y == self.y + self.height - 1
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
        pub fn key(&mut self, key: DecodedKey, room: &Room, mouse: &Mouse) {
            match key {
                DecodedKey::RawKey(code) => self.handle_raw(code, room, mouse),
                DecodedKey::Unicode(c) => self.handle_unicode(c),
                _ => {}
            }
        }

        pub fn update(&mut self) {
            self.draw();
            self.timer();
        }

        pub fn timer(&mut self) {
            self.timer += 1;
        }

        fn handle_unicode(&mut self, key: char) {
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
                _ => {}
            }
        }

        fn handle_raw(&mut self, key: KeyCode, room: &Room, mouse: &Mouse) {
            match key {
                KeyCode::ArrowLeft => {
                    self.move_to(self.x.saturating_sub(1), self.y, room, mouse);
                }
                KeyCode::ArrowRight => {
                    self.move_to(self.x + 1, self.y, room, mouse);
                }
                KeyCode::ArrowUp => {
                    self.move_to(self.x, self.y.saturating_sub(1), room, mouse);
                }
                KeyCode::ArrowDown => {
                    self.move_to(self.x, self.y + 1, room, mouse);
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

        pub fn update_bullets(&mut self, room: &Room, mouse: &mut Mouse) {

            for bullet in &mut self.bullets {
                bullet.clear();
                bullet.move_forward(room, mouse);
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

        pub fn move_to(&mut self, new_x: usize, new_y: usize, room: &Room, mouse: &Mouse) {
            if !room.is_wall(new_x, new_y) && !room.is_wall(new_x + 1, new_y) && !mouse.is_collision(new_x, new_y) && !mouse.is_collision(new_x + 1, new_y){
                self.x = new_x;
                self.y = new_y;
            }
            if mouse.is_collision(new_x, new_y) || mouse.is_collision(new_x + 1, new_y) {
                if mouse.dead {
                    return;
                }
                if self.health > 0 && (self.timer - self.last_hit > 8){
                    self.health -= 1;
                    self.last_hit = self.timer;
                }
            }
        }

        pub fn is_collision(&self, x: usize, y: usize) -> bool {
            (x == self.x || x == self.x + 1) && y == self.y
        }
    }

    pub struct Mouse {
        pub x: usize,
        pub y: usize,
        seed: SmallRng,
        timer: usize,
        dead: bool,
    }

    impl Mouse {

        pub fn new(room: &Room) -> Self {
            Self {
                x: room.x + 5,
                y: room.y + 5,
                seed: SmallRng::seed_from_u64(unsafe { core::arch::x86_64::_rdtsc() }),
                timer: 0,
                dead: false,
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

        pub fn random_move(&mut self, room: &Room, player: &mut Player) {
            if !self.dead {
                self.timer += 1;
                if self.timer % 5 == 0 {
                    let (dx, dy) = match self.seed.gen_range(0..4) {
                        0 => (-1, 0),
                        1 => (1, 0),
                        2 => (0, -1),
                        3 => (0, 1),
                        _ => (0, 0),
                    };

                    let new_x = self.x.saturating_add_signed(dx);
                    let new_y = self.y.saturating_add_signed(dy);

                    self.move_to(new_x, new_y, room, player);
                }
            }
        }

        pub fn move_to(&mut self, new_x: usize, new_y: usize, room: &Room, player: &mut Player) {
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
                    player.last_hit = self.timer;
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

        pub fn move_forward(&mut self, room: &Room, mouse: &mut Mouse) {
            if self.active {
                let new_x = self.x.saturating_add_signed(self.dx);
                let new_y = self.y.saturating_add_signed(self.dy);

                if room.is_wall(new_x, new_y) {
                    self.active = false;
                } else {
                    self.x = new_x;
                    self.y = new_y;
                }

                if mouse.is_collision(self.x, self.y) {
                    self.active = false;
                    mouse.die();
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


