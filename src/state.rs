//use cribbage_core::{Card, Rank, Suit};

pub enum GameState {
    NewGame,
    // WaitingForPlayersToCut(Board),
    // WaitingForPlayer1ToDeal(Board),
    // WaitingForPlayer2ToDeal(Board),
}

impl Default for GameState {
    fn default() -> Self {
        GameState::NewGame
    }
}
