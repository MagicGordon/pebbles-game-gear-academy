
# Pebbles Game

In this game, there are only two players, the user and the program.The program initializes `pebbles_count` pebbles, and two players take turns removing up to `max_pebbles_per_turn` pebbles per turn.The player who takes last pebble(s) is the winner.

### 🏗️ Building

```sh
cargo b --release
```

### ✅ Testing

Run all tests
```sh
cargo t
```

### init function

The init parameter must be pebbles_count > max_pebbles_per_turn; otherwise, panic occurs.
```rust
#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub enum DifficultyLevel {
    #[default]
    Easy,
    Hard,
}


#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct PebblesInit {
    pub difficulty: DifficultyLevel,
    pub pebbles_count: u32,
    pub max_pebbles_per_turn: u32,
}
```

DifficultyLevel:
- Easy: Program will choose the pebbles count to be removed randomly
- Hard: Program will find the best pebbles count by the formula `n % (m + 1)`, where `n` is the current number of pebbles and `m` is the maximum number that can be removed per turn.

### handle function

Receive the PebblesAction structure and execute the corresponding logic
```rust
pub enum PebblesAction {
    Turn(u32),
    GiveUp,
    Restart {
        difficulty: DifficultyLevel,
        pebbles_count: u32,
        max_pebbles_per_turn: u32,
    },
}
```

`Turn(u32)`: In this turn, users remove a specified number of pebbles.

`GiveUp`: In this turn, users do nothing.

`Restart { .. }`: Restart the game according to the specified parameters.

### state function

Returns the GameState structure without parameter.
```rust
#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct GameState {
    pub pebbles_count: u32,
    pub max_pebbles_per_turn: u32,
    pub pebbles_remaining: u32,
    pub difficulty: DifficultyLevel,
    pub first_player: Player,
    pub winner: Option<Player>,
}
```
