#![no_std]

use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{plot, Color, ColorCode, BUFFER_WIDTH, BUFFER_HEIGHT};
use rand::{Rng, SeedableRng, rngs::SmallRng};

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
    }

    impl Player {
        pub fn key(&mut self, key: DecodedKey, room: &Room, mouse: &Mouse) {
            match key {
                DecodedKey::RawKey(code) => self.handle_raw(code, room, mouse),
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

        pub fn new(room: &Room) -> Self {
            Self {
                x: room.x + room.width / 2,
                y: room.y + room.height / 2,
            }
        }

        pub fn draw(&self) {
            plot(':', self.x, self.y, ColorCode::new(Color::Yellow, Color::Black));
            plot('3', self.x + 1, self.y, ColorCode::new(Color::Yellow, Color::Black));
        }

        pub fn move_to(&mut self, new_x: usize, new_y: usize, room: &Room, mouse: &Mouse) {
            if !room.is_wall(new_x, new_y) && !room.is_wall(new_x + 1, new_y) && !mouse.is_collision(new_x, new_y) && !mouse.is_collision(new_x + 1, new_y){
                self.x = new_x;
                self.y = new_y;
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
        timer: usize
    }

    impl Mouse {

        pub fn new(room: &Room) -> Self {
            Self {
                x: room.x + 5,
                y: room.y + 5,
                seed: SmallRng::seed_from_u64(unsafe { core::arch::x86_64::_rdtsc() }),
                timer: 0
            }
        }

        pub fn draw(&self) {
            plot('~', self.x, self.y, ColorCode::new(Color::Yellow, Color::Black));
            plot('o', self.x + 1, self.y, ColorCode::new(Color::Yellow, Color::Black));
        }

        pub fn random_move(&mut self, room: &Room, player: &Player) {
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

        pub fn move_to(&mut self, new_x: usize, new_y: usize, room: &Room, player: &Player) {
            if !room.is_wall(new_x, new_y) && !room.is_wall(new_x + 1, new_y) && !player.is_collision(new_x, new_y) && !player.is_collision(new_x + 1, new_y){
                self.x = new_x;
                self.y = new_y;
            }
        }

        pub fn is_collision(&self, x: usize, y: usize) -> bool {
            (x == self.x || x == self.x + 1) && y == self.y
        }

    }


