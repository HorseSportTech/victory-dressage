use crate::debug;
use crate::sockets::handlers;
use crate::sockets::message_types::server::Payload;
use crate::sockets::message_types::{application, server};
use crate::{state::ManagedApplicationState, STORE_URI};
use socket_manager::SocketError;
use socket_manager::{message::Message, SocketManagerBuilder};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

use super::handlers::HandlerError;

pub const STORED_MESSAGES: &str = "STORED_MESSAGES";

pub async fn manage(owned_handle: tauri::AppHandle) {
    let handle = owned_handle.clone(); //std::sync::Arc::new(owned_handle);
    let duration = std::time::Duration::from_secs(10);
    loop {
        let handle = handle.clone();
        let url = {
            let (judge_id, permenant_id, maybe_token) = {
                let state = handle.state::<ManagedApplicationState>();

                match state
                    .read_async(|x| {
                        (
                            x.get_judge().map_or_else(String::new, |x| x.id.id()),
                            x.permanent_id,
                            x.maybe_token(),
                        )
                    })
                    .await
                {
                    Ok(vals) => vals,
                    Err(_) => {
                        tokio::time::sleep(duration).await;
                        continue;
                    }
                }
            };

            match maybe_token {
                None => {
                    tokio::time::sleep(duration).await;
                    continue;
                }
                Some(token) => format!(
                    concat!(env!("API_SOCKET"), "dressage/application/v2/{}/{}?tk={}"),
                    judge_id, permenant_id, token
                ),
            }
        };

        match SocketManagerBuilder::<_, application::Payload, server::Payload>::new(&url)
            .transform_handler({
                let handle = handle.clone();
                move |msg| {
                    // convert to Message if required
                    let original_message = Message::new(msg.clone());
                    // store message in storage
                    {
                        if let Ok(store) = handle.store(STORE_URI) {
                            let mut prev_messages: Vec<application::Payload> = store
                                .get(STORED_MESSAGES)
                                .and_then(|msg| serde_json::from_value(msg).ok())
                                .unwrap_or_default();

                            prev_messages.push(msg);
                            store.set(
                                STORED_MESSAGES,
                                serde_json::to_value(prev_messages)
                                    .expect("Should be able to parse"),
                            );
                        }
                    }
                    // return message
                    original_message.to_msg()
                }
            })
            .keep_alive_handler({
                let handle = handle.clone();
                move || {
                    let state = handle.state::<ManagedApplicationState>();
                    let socks =
                        handle.state::<socket_manager::SocketManager<application::Payload>>();

                    // TODO: Make this into a handler which batches these message into
                    // a vec before sending. Server also needs to be updated to
                    // handle this batch processing.
                    if let Ok(store) = handle.store(STORE_URI) {
                        let mut prev_messages: Vec<application::Payload> = store
                            .get(STORED_MESSAGES)
                            .and_then(|msg| serde_json::from_value(msg).ok())
                            .unwrap_or_default();

                        for pm in prev_messages.clone().into_iter() {
                            socks.send(pm);
                        }
                    }
                    state
                        .write(|app_state| {
                            app_state.battery.check();
                            Some(app_state.clone().wrap())
                        })
                        .ok()?
                }
            })
            .receive_handler({
                let handle = handle.clone();
                move |msg| {
                    let handle = handle.clone();
                    async move {
                        let asm = handle.state::<socket_manager::SocketManager<Payload>>();
                        _ = asm.send(Payload::Ack(msg.id));
                        use server::CompetitionMessage as CM;
                        match msg.message {
                            Payload::Competition(c) => {
                                let response = match c {
                                    CM::Lock(x) => x.handle(handle),
                                    CM::Trend(x) => x.handle(handle),
                                    CM::Reset(x) => x.handle(handle),
                                    CM::Status(x) => x.handle(handle),
                                    CM::Signal(x) => x.handle(handle),
                                    CM::AlterStarter(x) => x.handle(handle),
                                    CM::Unsubscribe => Err(MessageError::ClosedByServer)?,
                                };
                                match response {
                                    Err(x) if matches!(x, handlers::HandlerError::Fatal(_)) => {
                                        Err(MessageError::Handler(x))?
                                    }
                                    _ => (),
                                }
                            }
                            a @ Payload::ApplicationState { .. } => {
                                debug!(green, "App State {a:?}")
                            }
                            Payload::Ack(k) => debug!(dim, "Ack {k}"),
                        }
                        Ok::<(), MessageError>(())
                    }
                }
            })
            .run()
        {
            Ok((sender, manager)) => {
                handle.manage(sender);
                manager.await.ok();
                let s = handle.state::<socket_manager::SocketManager<application::Payload>>();
            }
            Err(err) => debug!("{err:?}"),
        }
    }
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
