#![no_std]

use gmeta::{In, InOut, Metadata, Out};
use gstd::prelude::*;

pub struct PebblesMetadata;

impl Metadata for PebblesMetadata {
    type Init = In<PebblesInit>;
    type Handle = InOut<PebblesAction, PebblesEvent>;
    type State = Out<GameState>;
    type Reply = ();
    type Others = ();
    type Signal = ();
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct PebblesInit {
    pub difficulty: DifficultyLevel,
    pub pebbles_count: u32,
    pub max_pebbles_per_turn: u32,
}

impl PebblesInit {
    pub fn assert_valid(&self) {
        assert!(
            self.pebbles_count > self.max_pebbles_per_turn,
            "pebbles_count must greater than max_pebbles_per_turn"
        );
    }
}

impl From<PebblesInit> for GameState {
    fn from(pebbles_init: PebblesInit) -> Self {
        Self {
            difficulty: pebbles_init.difficulty,
            pebbles_count: pebbles_init.pebbles_count,
            pebbles_remaining: pebbles_init.pebbles_count,
            max_pebbles_per_turn: pebbles_init.max_pebbles_per_turn,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub enum DifficultyLevel {
    #[default]
    Easy,
    Hard,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum PebblesAction {
    Turn(u32),
    GiveUp,
    Restart {
        difficulty: DifficultyLevel,
        pebbles_count: u32,
        max_pebbles_per_turn: u32,
    },
}

impl PebblesAction {
    pub fn assert_valid(&self, max_pebbles_per_turn: u32) {
        match self {
            PebblesAction::Turn(remove_amount) => {
                assert!(
                    remove_amount <= &max_pebbles_per_turn,
                    "Invalid turn amount"
                );
            }
            PebblesAction::GiveUp => {}
            PebblesAction::Restart {
                pebbles_count,
                max_pebbles_per_turn,
                ..
            } => {
                assert!(
                    pebbles_count > max_pebbles_per_turn,
                    "pebbles_count must greater than max_pebbles_per_turn"
                );
            }
        }
    }
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum PebblesEvent {
    CounterTurn(u32),
    Won(Player),
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub enum Player {
    #[default]
    User,
    Program,
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct GameState {
    pub pebbles_count: u32,
    pub max_pebbles_per_turn: u32,
    pub pebbles_remaining: u32,
    pub difficulty: DifficultyLevel,
    pub first_player: Player,
    pub winner: Option<Player>,
}

impl GameState {
    fn get_random_u32(&self) -> u32 {
        let salt = gstd::msg::id();
        let (hash, _num) =
            gstd::exec::random(salt.into()).expect("get_random_u32(): random call failed");
        u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
    }

    fn random_remove_amount(&self) -> u32 {
        if self.max_pebbles_per_turn < self.pebbles_remaining {
            self.get_random_u32() % (self.max_pebbles_per_turn + 1)
        } else {
            self.pebbles_remaining
        }
    }

    fn optimal_remove_amount(&self) -> u32 {
        let remainder = self.pebbles_remaining % (self.max_pebbles_per_turn + 1);
        if remainder == 0 {
            1
        } else {
            remainder
        }
    }

    pub fn is_over(&self) -> bool {
        self.winner.is_some()
    }

    pub fn chooses_first_player(&mut self) {
        let is_even = self.get_random_u32() % 2 == 0;
        if is_even {
            self.first_player = Player::User;
        } else {
            self.first_player = Player::Program;
        }
    }

    pub fn processes_program_turn(&mut self) -> u32 {
        let remove_amount = match self.difficulty {
            DifficultyLevel::Easy => self.random_remove_amount(),
            DifficultyLevel::Hard => self.optimal_remove_amount(),
        };
        self.pebbles_remaining -= remove_amount;
        if self.pebbles_remaining == 0 {
            self.winner = Some(Player::Program);
        }
        remove_amount
    }

    pub fn processes_user_turn(&mut self, remove_amount: u32) {
        self.pebbles_remaining -= remove_amount;
        if self.pebbles_remaining == 0 {
            self.winner = Some(Player::User);
        }
    }
}
