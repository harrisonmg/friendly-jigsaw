use std::{
    env,
    fs::{read_to_string, write},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use clap::Parser;
use futures::future::join;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use tokio::{
    select,
    sync::{
        broadcast::{self, Receiver},
        mpsc::{unbounded_channel, UnboundedSender},
        RwLock,
    },
    time::{sleep, timeout},
};
use uuid::Uuid;
use warp::{
    ws::{Message, WebSocket},
    Filter, Rejection, Reply,
};

use game::{AnyGameEvent, PlayerDisconnectedEvent, Puzzle};

mod puzzle_loader;
use puzzle_loader::PuzzleLoader;

const BROADCAST_CHANNEL_SIZE: usize = 10_000;
const DEFAULT_PORT: u16 = 80;

const CLIENT_TIMEOUT: Duration = Duration::from_secs(60 * 10);

const PUZZLE_BACKUP_INTERVAL: Duration = Duration::from_secs(30);
const PUZZLE_BACKUP_FILE: &str = "puzzle_backup.json";

const COMPLETION_CHECK_INTERVAL: Duration = Duration::from_secs(3);
const COMPLETE_HOLD_TIME: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy)]
struct ServerGameEvent {
    pub client_id: Uuid,
    pub game_event: AnyGameEvent,
}

#[derive(Parser)]
struct Args {
    queue_file: PathBuf,

    #[arg(short, long)]
    load_backup: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = Args::parse();
    let mut puzzle_loader = PuzzleLoader::new(args.queue_file);

    let puzzle = match args.load_backup {
        Some(backup) => Puzzle::deserialize(&read_to_string(backup).unwrap()).unwrap(),
        None => puzzle_loader.next().unwrap(),
    };

    let puzzle = Arc::new(RwLock::new(puzzle));

    let (event_input_tx, mut event_input_rx) = unbounded_channel::<ServerGameEvent>();
    let (event_output_tx, _) = broadcast::channel::<ServerGameEvent>(BROADCAST_CHANNEL_SIZE);

    // route that serves up the client application
    let http_route = warp::fs::dir("dist");

    // client route that gives them a puzzle ref and channel handles
    let puzzle_clone = puzzle.clone();
    let event_output_tx_clone = event_output_tx.clone();
    let client_route = warp::path("client")
        .and(warp::ws())
        .and(warp::any().map(move || puzzle_clone.clone()))
        .and(warp::any().map(move || event_input_tx.clone()))
        .and(warp::any().map(move || event_output_tx_clone.subscribe()))
        .and_then(ws_handler);

    let routes = warp::get().and(http_route).or(client_route);

    // serve that shit up
    let port = env::var("PORT").map_or(DEFAULT_PORT, |var| var.parse().unwrap());
    let serve = warp::serve(routes).run(([0, 0, 0, 0], port));

    // apply events to the puzzle and dispatch the generated events to clients
    let puzzle_clone = puzzle.clone();
    let event_handler = async move {
        while let Some(server_event) = event_input_rx.recv().await {
            let res_events = puzzle_clone
                .write()
                .await
                .apply_event(server_event.game_event);
            for res_event in res_events {
                let _ = event_output_tx.send(ServerGameEvent {
                    client_id: server_event.client_id,
                    game_event: res_event,
                });
            }
        }
    };

    let puzzle_clone = puzzle.clone();
    let puzzle_backup = async move {
        loop {
            sleep(PUZZLE_BACKUP_INTERVAL).await;
            let json = puzzle_clone.read().await.serialize();
            write(PUZZLE_BACKUP_FILE, json).unwrap();
        }
    };

    let completion_handler = async move {
        while !puzzle.read().await.is_complete() {
            sleep(COMPLETION_CHECK_INTERVAL).await;
        }

        info!("Puzzle complete!");
        info!("Shutting down server in {COMPLETE_HOLD_TIME:?}...");

        puzzle_loader.pop_current();
        sleep(COMPLETE_HOLD_TIME).await;
    };

    select! {
        _ = serve => panic!(),
        _ = event_handler => panic!(),
        _ = puzzle_backup => panic!(),
        _ = completion_handler => (),
    }
}

async fn ws_handler(
    ws: warp::ws::Ws,
    puzzle: Arc<RwLock<Puzzle>>,
    event_tx: UnboundedSender<ServerGameEvent>,
    event_rx: Receiver<ServerGameEvent>,
) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(move |warp_ws| client_handler(warp_ws, puzzle, event_tx, event_rx)))
}

async fn client_handler(
    ws: WebSocket,
    puzzle: Arc<RwLock<Puzzle>>,
    event_tx: UnboundedSender<ServerGameEvent>,
    mut event_rx: Receiver<ServerGameEvent>,
) {
    let client_id = Uuid::new_v4();

    info!("client {client_id} connected");

    let (mut ws_tx, mut ws_rx) = ws.split();

    // first, send the puzzle
    let msg = Message::text(&*puzzle.read().await.serialize());

    if ws_tx.send(msg).await.is_err() {
        info!("client {client_id} disconnected");
        return;
    }

    // receive client events and forward them to server event handler
    let client_rx_handler = async move {
        loop {
            if let Ok(item) = timeout(CLIENT_TIMEOUT, ws_rx.next()).await {
                let res = match item {
                    Some(res) => res,
                    None => {
                        error!("no item received from client {client_id}");
                        break;
                    }
                };

                let msg = match res {
                    Ok(msg) => msg,
                    Err(_) => break,
                };

                if msg.is_text() {
                    if let Ok(mut game_event) = AnyGameEvent::deserialize(msg.to_str().unwrap()) {
                        game_event.add_player_id(client_id);

                        let server_event = ServerGameEvent {
                            client_id,
                            game_event,
                        };

                        if let Err(e) = event_tx.send(server_event) {
                            error!(
                            "error sending event to server model in client {client_id} task: {e}"
                        );
                            break;
                        }
                    } else {
                        error!("malformed message from client {client_id}: {msg:?}");
                        break;
                    }
                } else {
                    if !msg.is_close() {
                        error!("unhandled message from client {client_id}: {msg:?}");
                    }
                    break;
                }
            } else {
                info!("client {client_id} timed out");
                break;
            }
        }

        let res = event_tx.send(ServerGameEvent {
            client_id,
            game_event: AnyGameEvent::PlayerDisconnected(PlayerDisconnectedEvent {
                player_id: client_id,
            }),
        });

        match res {
            Ok(()) => info!("client {client_id} disconnected"),
            Err(e) => error!("error sending event to server model in client {client_id} task: {e}"),
        }
    };

    // forward broadcasted events to client
    let client_tx_handler = async move {
        while let Ok(event) = event_rx.recv().await {
            if event.client_id == client_id
                && matches!(event.game_event, AnyGameEvent::PlayerDisconnected(_))
            {
                break;
            }

            // don't echo client events unless they're piece connection events
            // since those are always handled server-side first
            // to prevent non-deterministic connection logic due to rounding errors
            if event.client_id != client_id
                || matches!(event.game_event, AnyGameEvent::PieceConnection(_))
            {
                #[allow(clippy::collapsible_if)]
                if ws_tx
                    .send(Message::text(event.game_event.serialize()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    };

    join(client_rx_handler, client_tx_handler).await;
}
