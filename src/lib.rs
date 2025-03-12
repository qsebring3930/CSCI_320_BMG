#![no_std]

use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{plot, Color, ColorCode, BUFFER_WIDTH, BUFFER_HEIGHT};

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
        pub fn key(&mut self, key: DecodedKey, room: &Room) {
            match key {
                DecodedKey::RawKey(code) => self.handle_raw(code, room),
                _ => {}
            }
        }

        fn handle_raw(&mut self, key: KeyCode, room: &Room) {
            match key {
                KeyCode::ArrowLeft => {
                    self.move_to(self.x.saturating_sub(1), self.y, room);
                }
                KeyCode::ArrowRight => {
                    self.move_to(self.x + 1, self.y, room);
                }
                KeyCode::ArrowUp => {
                    self.move_to(self.x, self.y.saturating_sub(1), room);
                }
                KeyCode::ArrowDown => {
                    self.move_to(self.x, self.y + 1, room);
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

        pub fn move_to(&mut self, new_x: usize, new_y: usize, room: &Room) {
            if !room.is_wall(new_x, new_y) && !room.is_wall(new_x + 1, new_y) {
                self.x = new_x;
                self.y = new_y;
            }
        }
    }

    pub struct Mouse {
        pub x: usize,
        pub y: usize,
    }

    impl Mouse {

        pub fn new(room: &Room) -> Self {
            Self {
                x: room.x + 5,
                y: room.y + 5,
            }
        }

        pub fn draw(&self) {
            plot('~', self.x, self.y, ColorCode::new(Color::Yellow, Color::Black));
            plot('o', self.x + 1, self.y, ColorCode::new(Color::Yellow, Color::Black));
        }

        pub fn move_to(&mut self, new_x: usize, new_y: usize, room: &Room) {
            if !room.is_wall(new_x, new_y) && !room.is_wall(new_x + 1, new_y) {
                self.x = new_x;
                self.y = new_y;
            }
        }
    }


