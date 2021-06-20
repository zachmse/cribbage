mod matchmaker;

use matchmaker::{start_matchmaker, Command};

use std::convert::Infallible;

use tokio::spawn;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::oneshot;
use warp::ws::WebSocket;
use warp::Filter;
use warp::Reply;

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Warp Chat</title>
    </head>
    <body>
        <h1>Warp chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="join">Join</button>
        <button type="button" id="send" style="display: none;">Send</button>
        <script type="text/javascript">
        const chat = document.getElementById('chat');
        const text = document.getElementById('text');
        var ws;

        function message(data) {
            const line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }

        join.onclick = function() {
            ws = new WebSocket('ws://' + location.host + '/join' + '/' + text.value);
            ws.onopen = function() {
                join.style = "display: none;";
                send.style = "";
                text.value = "";
                chat.innerHTML = '<p><em>Connected!</em></p>';
            };

            ws.onmessage = function(msg) {
                message('<Them>: ' + msg.data);
            };

            ws.onclose = function() {
                chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
            };
        };

        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            text.value = '';
            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;

async fn create_game(tx: Sender<Command>) -> Result<impl Reply, Infallible> {
    let (resp_tx, resp_rx) = oneshot::channel();
    let cmd = Command::CreateRoom { responder: resp_tx };

    let _ = tx.send(cmd).await;
    Ok(resp_rx.await.unwrap().unwrap())
}

async fn join_game(tx: Sender<Command>, password: String, websocket: WebSocket) {
    let (resp_tx, resp_rx) = oneshot::channel();
    let cmd = Command::JoinRoom {
        password,
        websocket,
        responder: resp_tx,
    };

    let _ = tx.send(cmd).await;
    let _ = resp_rx.await.unwrap();
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);
    let tx2 = tx.clone();
    let _matchmaker = spawn(start_matchmaker(rx));

    let create = warp::path("create").and_then(move || create_game(tx.clone()));

    let join = warp::path("join")
        .and(warp::path::param::<String>())
        .and(warp::ws())
        .map(move |password: String, ws: warp::ws::Ws| {
            let tx3 = tx2.clone();
            ws.on_upgrade(move |websocket| join_game(tx3, password, websocket))
        });

    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));
    let routes = index.or(create).or(join);
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
