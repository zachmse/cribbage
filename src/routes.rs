use rocket::{get, State};

use crate::Games;

#[get("/create")]
pub fn create(games: &State<Games>) -> String {
    games.add()
}

#[get("/count")]
pub fn count(games: &State<Games>) -> String {
    games.count().to_string()
}
