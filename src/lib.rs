#![no_std]

use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{plot, Color, ColorCode};




    pub struct Room {
        pub width: usize,
        pub height: usize,
    }

    impl Room {
        pub fn new(width: usize, height: usize) -> Self {
            Self { width, height }
        }

        pub fn draw(&self) {
            for y in 0..self.height {
                for x in 0..self.width {
                    let ch = if x == 0 || x == self.width - 1 || y == 0 || y == self.height - 1 {
                        '#' // Walls
                    } else {
                        '.' // Floor
                    };
                    plot(ch, x, y, ColorCode::new(Color::White, Color::Black));
                }
            }
        }

        pub fn is_wall(&self, x: usize, y: usize) -> bool {
            x == 0 || x == self.width - 1 || y == 0 || y == self.height - 1
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

        pub fn new(start_x: usize, start_y: usize) -> Self {
            Self { x: start_x, y: start_y }
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
