#![no_std]

use gstd::msg;
use pebbles_game_io::*;

static mut PEBBLES_GAME: Option<GameState> = None;

fn get_pebbles_game_mut() -> &'static mut GameState {
    unsafe {
        PEBBLES_GAME
            .as_mut()
            .expect("PEBBLES_GAME is not initialized")
    }
}
fn get_pebbles_game() -> &'static GameState {
    unsafe {
        PEBBLES_GAME
            .as_ref()
            .expect("PEBBLES_GAME is not initialized")
    }
}

fn init_game_state(pebbles_init: PebblesInit) {
    let mut game_state: GameState = pebbles_init.into();
    // Chooses the first player
    game_state.chooses_first_player();
    // Processes the first turn if the first player is Program
    if matches!(game_state.first_player, Player::Program) {
        game_state.processes_program_turn();
    }
    // Fills the GameState structure.
    unsafe {
        PEBBLES_GAME = Some(game_state);
    };
}

fn processes_program_turn(pebbles_game: &mut GameState) {
    // Processes the Program turn and check whether it wins;
    let program_remove_amount = pebbles_game.processes_program_turn();
    // Send a message to the user with the correspondent PebblesEvent;
    if pebbles_game.is_over() {
        msg::reply(PebblesEvent::Won(Player::Program), 0)
            .expect("Error in sending PebblesEvent::Won");
    } else {
        msg::reply(PebblesEvent::CounterTurn(program_remove_amount), 0)
            .expect("Error in sending PebblesEvent::CounterTurn");
    }
}

#[no_mangle]
extern "C" fn init() {
    // Receives PebblesInit using the msg::load function
    let pebbles_init: PebblesInit = msg::load().expect("Unable to decode `PebblesInit`");
    // Checks input data for validness
    pebbles_init.assert_valid();
    init_game_state(pebbles_init);
}

#[no_mangle]
extern "C" fn handle() {
    // Receives PebblesAction using msg::load function
    let pebbles_action: PebblesAction = msg::load().expect("Unable to decode `PebblesAction`");
    let pebbles_game = get_pebbles_game_mut();
    if !matches!(pebbles_action, PebblesAction::Restart { .. }) {
        assert!(!pebbles_game.is_over(), "Game is over");
    }

    // Checks input data for validness;
    pebbles_action.assert_valid(pebbles_game.max_pebbles_per_turn);

    match pebbles_action {
        PebblesAction::Turn(remove_amount) => {
            // Processes the User's turn and check whether they win;
            pebbles_game.processes_user_turn(remove_amount);
            if pebbles_game.is_over() {
                // Send a message to the user with the correspondent PebblesEvent;
                msg::reply(PebblesEvent::Won(Player::User), 0)
                    .expect("Error in sending PebblesEvent::Won");
            } else {
                processes_program_turn(pebbles_game);
            }
        }
        PebblesAction::GiveUp => {
            processes_program_turn(pebbles_game);
        }
        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => {
            let pebbles_init = PebblesInit {
                difficulty,
                pebbles_count,
                max_pebbles_per_turn,
            };
            // Checks input data for validness
            pebbles_init.assert_valid();
            init_game_state(pebbles_init);
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    msg::reply(get_pebbles_game(), 0).expect("failed to encode or reply from `state()`");
}
