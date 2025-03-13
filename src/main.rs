#![no_std]
#![no_main]

use crossbeam::atomic::AtomicCell;
use pc_keyboard::DecodedKey;
use pluggable_interrupt_os::{vga_buffer::clear_screen, HandlerTable};
use CSCI_320_BMG::{GameState, Mouse, Player, Room};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .startup(startup)
        .cpu_loop(cpu_loop)
        .start()
}

static LAST_KEY: AtomicCell<Option<DecodedKey>> = AtomicCell::new(None);
static TICKED: AtomicCell<bool> = AtomicCell::new(false);

fn cpu_loop() -> ! {
    let mut gamestate = GameState::new();
    let mut player = Player::new(&gamestate.current_room);
    let mut mouse = Mouse::new(&gamestate.current_room);

    mouse.draw();
    player.draw();

    loop {
        if let Ok(_) = TICKED.compare_exchange(true, false) {
            mouse.random_move(&gamestate.current_room, &mut player);
            gamestate.update(&mut player);
            mouse.draw();
            player.update();
            player.update_bullets(&gamestate.current_room, &mut mouse);

        }

        if let Ok(k) = LAST_KEY.fetch_update(|k| if k.is_some() { Some(None) } else { None }) {
            if let Some(k) = k {
                player.key(k, &mouse, &mut gamestate);
            }
        }
    }
}

fn key(key: DecodedKey) {
    LAST_KEY.store(Some(key));
}

fn tick() {
    TICKED.store(true);
}

fn startup() {
    clear_screen();
}
