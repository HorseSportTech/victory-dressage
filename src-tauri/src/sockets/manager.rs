use std::sync::Arc;
use std::sync::RwLock;

use crate::debug;
use crate::sockets::handlers::{self, handle_ack, handle_application_state};
use crate::sockets::message_types::server::Payload;
use crate::sockets::message_types::{application, server};
use crate::state::ManagedApplicationState;
use socket_manager::SocketError;
use socket_manager::{message::Message, SocketManager};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

use super::handlers::HandlerError;

pub const STORED_MESSAGES: &str = "STORED_MESSAGES";
const DURATION: std::time::Duration = std::time::Duration::from_secs(10);

pub struct ManagedSocket(
    pub Arc<RwLock<SocketManager<tauri::AppHandle, application::Payload, server::Payload>>>,
);
impl ManagedSocket {
    pub fn new(s: SocketManager<tauri::AppHandle, application::Payload, server::Payload>) -> Self {
        Self(Arc::new(RwLock::new(s)))
    }
    pub async fn send_raw(&self, msg: Message<application::Payload>) -> Result<(), SocketError> {
        self.0
            .read()
            .map_err(|_| SocketError::Closed)?
            .send_raw(msg)
    }
    pub async fn send(&self, msg: application::Payload) -> Result<(), SocketError> {
        self.0.read().map_err(|_| SocketError::Closed)?.send(msg)
    }
}

pub async fn manage(owned_handle: tauri::AppHandle) {
    let handle = owned_handle.clone();
    let builder = SocketManager::<tauri::AppHandle, application::Payload, server::Payload>::new(
        transform_handler,
        recieve_handler,
        keep_alive_handler,
        handle.clone(),
    );
    loop {
        let mut built_manager = builder.clone();
        let url = match get_url_with_query_token(&handle).await {
            Some(url) => url,
            None => continue,
        };
        match built_manager.connect_and_run(&url) {
            Ok((sender, manager)) => {
                if let Some(ref mut previous_state) = owned_handle.try_state::<ManagedSocket>() {
                    if let Ok(mut l) = previous_state.0.write() {
                        *l = sender;
                    }
                } else {
                    owned_handle.manage(ManagedSocket::new(sender));
                }
                debug!(dim, "Handler managing things");
                let _ = manager.await.inspect_err(|err| debug!(red, "{err:?}"));
            }
            Err(err) => debug!("{err:?}"),
        }
    }
}

async fn get_url_with_query_token(handle: &tauri::AppHandle) -> Option<String> {
    let state = handle.state::<ManagedApplicationState>();
    if let Ok(url) = {
        state
            .read_async(|x| {
                (
                    x.get_judge_id().map_or_else(Default::default, |x| x.id()),
                    x.permanent_id.clone(),
                    x.maybe_token(),
                )
            })
            .await
    }
    .map(|(judge_id, permenant_id, maybe_token)| {
        maybe_token.map(|token| {
            format!(
                concat!(env!("API_SOCKET"), "dressage/application/v2/{}/{}?tk={}"),
                judge_id, permenant_id, token
            )
        })
    }) {
        url
    } else {
        let _ = state.refresh_if_required().await;
        tokio::time::sleep(DURATION).await;
        None
    }
}
async fn recieve_handler(
    msg: socket_manager::message::Message<server::Payload>,
    handle: tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    use server::CompetitionMessage as CM;
    let asm = handle
        .try_state::<ManagedSocket>()
        .expect("To have socket manager available inside recieve handler");
    _ = asm.send(application::Payload::Ack(msg.id));
    match msg.message {
        Payload::Competition(c) => {
            let response = match c {
                CM::Lock(x) => x.handle(handle),
                CM::Trend(x) => x.handle(handle),
                CM::Reset(x) => x.handle(handle),
                CM::Status(x) => x.handle(handle),
                CM::Signal(x) => x.handle(handle),
                CM::AlterStarter(x) => x.handle(handle),
                CM::Unsubscribe => Err(Box::new(MessageError::ClosedByServer))?,
            };
            if let Err(x) = response {
                if let handlers::HandlerError::Fatal(_) = x {
                    Err(Box::new(MessageError::Handler(x)))?
                }
            }
        }
        a @ Payload::ApplicationState { .. } => handle_application_state(a),
        Payload::Ack(k) => handle_ack(k, &handle),
    }
    Ok(())
}
fn transform_handler(
    msg: application::Payload,
    handle: tauri::AppHandle,
) -> socket_manager::tungstenite::Message {
    // convert to Message if required
    let original_message = Message::new(msg.clone());
    // store message in storage
    {
        if let Ok(store) = handle.store(env!("STORE_URI")) {
            let mut prev_messages: Vec<application::Payload> = store
                .get(STORED_MESSAGES)
                .and_then(|msg| serde_json::from_value(msg).ok())
                .unwrap_or_default();

            prev_messages.push(msg);
            store.set(
                STORED_MESSAGES,
                serde_json::to_value(prev_messages).expect("Should be able to parse"),
            );
        }
    }
    // return message
    original_message.to_msg()
}
async fn keep_alive_handler(_: (), handle: tauri::AppHandle) -> Option<application::Payload> {
    let state = handle.state::<ManagedApplicationState>();
    let socks = handle
        .try_state::<ManagedSocket>()
        .expect("To always have this available inside the handler");

    // TODO: Make this into a handler which batches these message into
    // a vec before sending. Server also needs to be updated to
    // handle this batch processing.
    if let Ok(store) = handle.store(env!("STORE_URI")) {
        let prev_messages: Vec<Message<application::Payload>> = store
            .get(STORED_MESSAGES)
            .and_then(|msg| serde_json::from_value(msg).ok())
            .unwrap_or_default();

        for pm in prev_messages.clone().into_iter() {
            if socks.send_raw(pm).await.is_err() {
                return None;
            }
        }
    }
    state
        .write(|app_state| {
            app_state.battery.check();
            app_state.clone().wrap()
        })
        .ok()?
}

#[derive(thiserror::Error, Debug)]
pub enum MessageError {
    #[error("Closed by server")]
    ClosedByServer,
    #[error(transparent)]
    Socket(#[from] SocketError),
    #[error("Error occured in handler {0}")]
    Handler(#[from] HandlerError),
    // other varients depending on the varients
}
