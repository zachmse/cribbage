use std::collections::HashMap;

use futures::{select, SinkExt, StreamExt};
use rand::{thread_rng, Rng};
use tokio::spawn;
use tokio::sync::{mpsc, oneshot};
use warp::ws::WebSocket;

const PASSWORD_CHARSET: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const PASSWORD_LENGTH: usize = 8;

#[derive(Debug)]
pub enum MatchMakerError {}

type Responder<T> = oneshot::Sender<Result<T, MatchMakerError>>;

#[derive(Default)]
struct Game {
    player1: Option<WebSocket>,
}

pub enum Command {
    CreateRoom {
        responder: Responder<String>,
    },
    JoinRoom {
        password: String,
        websocket: WebSocket,
        responder: Responder<()>,
    },
}

async fn create_room(map: &mut HashMap<String, Game>) -> String {
    let mut rng = thread_rng();
    loop {
        let password: String = (0..PASSWORD_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..PASSWORD_CHARSET.len());
                PASSWORD_CHARSET[idx] as char
            })
            .collect();

        if !map.contains_key(&password) {
            map.insert(password.clone(), Game::default());
            break password;
        }
    }
}

async fn join_room(password: &str, map: &mut HashMap<String, Game>, websocket: WebSocket) {
    if map.get(password).unwrap().player1.is_none() {
        map.get_mut(password).unwrap().player1 = Some(websocket);
    } else {
        let game = map.remove(password).unwrap();
        let player1 = game.player1.unwrap();
        let player2 = websocket;
        let (mut player1_tx, player1_rx) = player1.split();
        let (mut player2_tx, player2_rx) = player2.split();
        let mut player1_rx = player1_rx.fuse();
        let mut player2_rx = player2_rx.fuse();
        spawn(async move {
            loop {
                let _ = select! {
                    message_from_player1 = player1_rx.next() => player2_tx.send(message_from_player1.unwrap().unwrap()).await,
                    message_from_player2 = player2_rx.next() => player1_tx.send(message_from_player2.unwrap().unwrap()).await,
                };
            }
        });
    }
}

pub async fn start_matchmaker(mut rx: mpsc::Receiver<Command>) {
    let mut map = HashMap::new();
    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::CreateRoom { responder } => {
                let password = create_room(&mut map).await;
                let _ = responder.send(Ok(password));
            }
            Command::JoinRoom {
                password,
                websocket,
                responder,
            } => {
                join_room(&password, &mut map, websocket).await;
                let _ = responder.send(Ok(()));
            }
        }
    }
}
