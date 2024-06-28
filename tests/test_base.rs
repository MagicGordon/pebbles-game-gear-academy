use gstd::{
    errors::{ErrorReplyReason, ReplyCode, SimpleExecutionError, SuccessReplyReason},
    Encode,
};
use gtest::{Log, Program, System};
use pebbles_game_io::*;

const USER: u64 = 100;
const PORGRAM: u64 = 1;

#[test]
fn test_init_failed() {
    let system = System::new();
    system.init_logger();

    let program = Program::current(&system);
    let res = program.send(
        USER,
        PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 20,
            max_pebbles_per_turn: 50,
        },
    );
    let error_log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .payload_bytes(
            "Panic occurred: panicked with 'pebbles_count must greater than max_pebbles_per_turn'",
        )
        .reply_code(ReplyCode::Error(ErrorReplyReason::Execution(
            SimpleExecutionError::UserspacePanic,
        )));
    assert!(res.main_failed() && !res.others_failed() && res.contains(&error_log));

    let res = program.send(
        USER,
        PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 100,
            max_pebbles_per_turn: 20,
        },
    );
    let error_log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .reply_code(ReplyCode::Error(ErrorReplyReason::InactiveActor));
    assert!(!res.main_failed() && !res.others_failed() && res.contains(&error_log));
}

#[test]
fn test_init() {
    let system = System::new();
    system.init_logger();

    let program = Program::current(&system);
    let res = program.send(
        USER,
        PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 100,
            max_pebbles_per_turn: 20,
        },
    );
    let log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .reply_code(ReplyCode::Success(SuccessReplyReason::Auto));
    assert!(!res.main_failed() && !res.others_failed() && res.contains(&log));

    let state: GameState = program.read_state(b"").unwrap();
    assert_eq!(
        state.encode(),
        GameState {
            pebbles_count: 100,
            max_pebbles_per_turn: 20,
            pebbles_remaining: 100,
            difficulty: DifficultyLevel::Easy,
            first_player: Player::User,
            winner: None,
        }
        .encode()
    );
}

#[test]
fn test_handle() {
    let system = System::new();
    system.init_logger();

    let program = Program::current(&system);
    let res = program.send(
        USER,
        PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 100,
            max_pebbles_per_turn: 20,
        },
    );
    let auto_log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .reply_code(ReplyCode::Success(SuccessReplyReason::Auto));
    assert!(!res.main_failed() && !res.others_failed() && res.contains(&auto_log));
    let state: GameState = program.read_state(b"").unwrap();
    assert_eq!(
        state.encode(),
        GameState {
            pebbles_count: 100,
            max_pebbles_per_turn: 20,
            pebbles_remaining: 100,
            difficulty: DifficultyLevel::Easy,
            first_player: Player::User,
            winner: None,
        }
        .encode()
    );

    // Restart failed
    let res = program.send(
        USER,
        PebblesAction::Restart {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 5,
            max_pebbles_per_turn: 10,
        },
    );
    let error_log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .payload_bytes(
            "Panic occurred: panicked with 'pebbles_count must greater than max_pebbles_per_turn'",
        )
        .reply_code(ReplyCode::Error(ErrorReplyReason::Execution(
            SimpleExecutionError::UserspacePanic,
        )));
    assert!(res.main_failed() && !res.others_failed() && res.contains(&error_log));
    let state: GameState = program.read_state(b"").unwrap();
    assert_eq!(
        state.encode(),
        GameState {
            pebbles_count: 100,
            max_pebbles_per_turn: 20,
            pebbles_remaining: 100,
            difficulty: DifficultyLevel::Easy,
            first_player: Player::User,
            winner: None,
        }
        .encode()
    );

    // Restart success
    let res = program.send(
        USER,
        PebblesAction::Restart {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 50,
            max_pebbles_per_turn: 10,
        },
    );
    assert!(!res.main_failed() && !res.others_failed() && res.contains(&auto_log));
    let state: GameState = program.read_state(b"").unwrap();
    assert_eq!(
        state.encode(),
        GameState {
            pebbles_count: 50,
            max_pebbles_per_turn: 10,
            pebbles_remaining: 50,
            difficulty: DifficultyLevel::Easy,
            first_player: Player::User,
            winner: None,
        }
        .encode()
    );

    // Turn failed
    let res = program.send(USER, PebblesAction::Turn(15));
    let error_log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .payload_bytes("Panic occurred: panicked with 'Invalid turn amount'")
        .reply_code(ReplyCode::Error(ErrorReplyReason::Execution(
            SimpleExecutionError::UserspacePanic,
        )));
    assert!(res.main_failed() && !res.others_failed() && res.contains(&error_log));

    // Turn success
    let res = program.send(USER, PebblesAction::Turn(6));
    let log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .payload(PebblesEvent::CounterTurn(8))
        .reply_code(ReplyCode::Success(SuccessReplyReason::Manual));
    assert!(!res.main_failed() && !res.others_failed() && res.contains(&log));

    // GiveUp success
    let res = program.send(USER, PebblesAction::GiveUp);
    let log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .payload(PebblesEvent::CounterTurn(7))
        .reply_code(ReplyCode::Success(SuccessReplyReason::Manual));
    assert!(!res.main_failed() && !res.others_failed() && res.contains(&log));
    let state: GameState = program.read_state(b"").unwrap();
    assert_eq!(
        state.encode(),
        GameState {
            pebbles_count: 50,
            max_pebbles_per_turn: 10,
            pebbles_remaining: 29,
            difficulty: DifficultyLevel::Easy,
            first_player: Player::User,
            winner: None,
        }
        .encode()
    );

    // User win
    let res = program.send(USER, PebblesAction::Turn(7));
    let log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .payload(PebblesEvent::CounterTurn(3))
        .reply_code(ReplyCode::Success(SuccessReplyReason::Manual));
    assert!(!res.main_failed() && !res.others_failed() && res.contains(&log));
    let res = program.send(USER, PebblesAction::Turn(8));
    let log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .payload(PebblesEvent::CounterTurn(4))
        .reply_code(ReplyCode::Success(SuccessReplyReason::Manual));
    assert!(!res.main_failed() && !res.others_failed() && res.contains(&log));
    let res = program.send(USER, PebblesAction::Turn(7));
    let log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .payload(PebblesEvent::Won(Player::User))
        .reply_code(ReplyCode::Success(SuccessReplyReason::Manual));
    assert!(!res.main_failed() && !res.others_failed() && res.contains(&log));
    let state: GameState = program.read_state(b"").unwrap();
    assert_eq!(
        state.encode(),
        GameState {
            pebbles_count: 50,
            max_pebbles_per_turn: 10,
            pebbles_remaining: 0,
            difficulty: DifficultyLevel::Easy,
            first_player: Player::User,
            winner: Some(Player::User),
        }
        .encode()
    );

    // Turn failed after game over
    let res = program.send(USER, PebblesAction::Turn(1));
    let error_log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .payload_bytes("Panic occurred: panicked with 'Game is over'")
        .reply_code(ReplyCode::Error(ErrorReplyReason::Execution(
            SimpleExecutionError::UserspacePanic,
        )));
    assert!(res.main_failed() && !res.others_failed() && res.contains(&error_log));

    // GiveUp failed after game over
    let res = program.send(USER, PebblesAction::GiveUp);
    let error_log = Log::builder()
        .dest(USER)
        .source(PORGRAM)
        .payload_bytes("Panic occurred: panicked with 'Game is over'")
        .reply_code(ReplyCode::Error(ErrorReplyReason::Execution(
            SimpleExecutionError::UserspacePanic,
        )));
    assert!(res.main_failed() && !res.others_failed() && res.contains(&error_log));

    // Restart success after game over
    let res = program.send(
        USER,
        PebblesAction::Restart {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 60,
            max_pebbles_per_turn: 20,
        },
    );
    assert!(!res.main_failed() && !res.others_failed() && res.contains(&auto_log));
    let state: GameState = program.read_state(b"").unwrap();
    assert_eq!(
        state.encode(),
        GameState {
            pebbles_count: 60,
            max_pebbles_per_turn: 20,
            pebbles_remaining: 46,
            difficulty: DifficultyLevel::Easy,
            first_player: Player::Program,
            winner: None,
        }
        .encode()
    );
}
