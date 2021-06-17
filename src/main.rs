mod routes;
mod state;

use state::GameState;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::{launch, routes};

#[derive(Clone, Default)]
pub struct GameReference {
    game_state: Arc<Mutex<GameState>>,
}

impl GameReference {
    pub fn new() -> Self {
        Self {
            game_state: Arc::new(Mutex::new(GameState::NewGame)),
        }
    }
}

#[derive(Default)]
pub struct Games {
    map: RwLock<HashMap<String, GameReference>>,
}

impl Games {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, identifier: &str) -> Option<GameReference> {
        let lock = self.map.read().unwrap();
        lock.get(identifier).cloned()
    }

    pub fn add(&self) -> String {
        let identifier: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let cloned_identifier = identifier.clone();
        let game_reference = GameReference::new();
        let mut games = self.map.write().unwrap();
        games.insert(identifier, game_reference);
        cloned_identifier
    }

    pub fn count(&self) -> usize {
        let games = self.map.read().unwrap();
        games.len()
    }
}

#[launch]
fn server() -> _ {
    rocket::build()
        .manage(Games::new())
        .mount("/game", routes![routes::create, routes::count])
}
