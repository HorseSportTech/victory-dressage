use std::sync::Arc;
use tauri_plugin_store::StoreExt;
use tokio::{net::TcpStream, sync::Mutex};

use crate::{state::ManagedApplicationState, STORE_URI};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tauri::Manager;
use tokio_tungstenite::{
    tungstenite::{self, Message},
    MaybeTlsStream, WebSocketStream,
};

use super::messages::SocketMessage;

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type TxWS = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type RxWS = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub const STORED_MESSAGES: &'static str = "STORED_MESSAGES";

pub async fn manage(owned_handle: tauri::AppHandle) {
    let handle = &owned_handle;
    let state = handle.state::<ManagedApplicationState>();
    let mut tx: Option<Arc<Mutex<TxWS>>> = None;
    let mut rx: Option<Arc<Mutex<RxWS>>> = None;
    loop {
        match (&tx, &rx) {
            // select if either of these fail, then we loop to try and
            // reestablish the connection
            (Some(ref mtx), Some(ref mrx)) => {
                handle.manage(mtx.clone());
                tokio::select!(
                    Err(_err) = send_loop(mtx.clone().clone(), handle) => {
                        tx = None;
                        rx = None;
                        continue;
                    },
                    Err(_err) = recieve_message(mrx.clone().clone(), handle) => {
                        tx = None;
                        rx = None;
                        continue;
                    },
                );
            }
            _ => {
                let (permenant_id, token, judge_id) = {
                    let Some(token) = state
                        .read()
                        .expect("Better to terminate since it's at the start")
                        .maybe_token()
                    else {
                        eprintln!("No Token found");
                        let mut timer = tokio::time::interval(std::time::Duration::from_secs(10));
                        timer.tick().await;
                        timer.tick().await; // verify this second tick is required for 10 seconds
                        continue;
                    };
                    let x = state
                        .read()
                        .expect("Better to terminate since it's at the start");
                    (
                        x.permanent_id,
                        token,
                        x.get_judge().as_ref().and_then(|j| Some(j.id.clone())),
                    )
                };
                match tokio_tungstenite::connect_async(&format!(
                    "{root}dressage/application/v2/{judge_id}/{permenant_id}?tk={token}",
                    root = dotenv_codegen::dotenv!("API_SOCKET"),
                    judge_id = if let Some(ref j) = judge_id {
                        j.id()
                    } else {
                        String::new()
                    },
                ))
                .await
                {
                    Ok(soc) => {
                        println!("Connect to socket");
                        let (ntx, nrx) = soc.0.split();
                        tx = Some(Arc::new(Mutex::new(ntx)));
                        rx = Some(Arc::new(Mutex::new(nrx)));
                    }
                    Err(err) => {
                        eprintln!("{err:?}");
                        let mut timer = tokio::time::interval(std::time::Duration::from_secs(10));
                        timer.tick().await;
                        timer.tick().await; // verify this second tick is required for 10 seconds
                    }
                };
            }
        }
    }
}
async fn send_loop(
    arc_mut_tx: Arc<Mutex<TxWS>>,
    handle: &tauri::AppHandle,
) -> Result<(), tungstenite::error::Error> {
    let state = handle.state::<ManagedApplicationState>();
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        timer.tick().await; // in theory this is only used after the first time due to weird way
                            // tick works

        // get app state
        let output_state = {
            let s = state.read().unwrap();
            s.clone().wrap()
        };
        let message = Message::Binary(rmp_serde::to_vec(&output_state).expect("Must parse").into());

        // get current status
        let mut tx = arc_mut_tx.lock().await;
        tx.feed(message.clone()).await?;

        // get any unsent messages and feed
        if let Ok(store) = handle.store(STORE_URI) {
            if let Some(messages) = store.get(STORED_MESSAGES) {
                if let Ok(messages) = serde_json::from_value::<Vec<SocketMessage>>(messages) {
                    for message in messages.iter() {
                        let bytes = rmp_serde::to_vec(message)
                            .expect("Should parse as we just deserialized");
                        tx.feed(Message::Binary(bytes.into())).await?;
                    }
                }
            };
        }

        // send all
        tx.flush().await.inspect_err(|err| eprintln!("{err:?}"))?;
    }
}

async fn recieve_message(
    rx: Arc<Mutex<RxWS>>,
    handle: &tauri::AppHandle,
) -> Result<(), tungstenite::Error> {
    let _state = handle.state::<ManagedApplicationState>();
    let mut guard_rx = rx.lock().await;
    while let Some(msg) = guard_rx.next().await {
        println!("{:?}", msg);
    }
    Err(tungstenite::Error::ConnectionClosed)
}

