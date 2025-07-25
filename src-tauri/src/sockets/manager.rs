use crate::{state::ManagedApplicationState, STORE_URI};
use socket_manager::{message::Message, SocketManagerBuilder};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

use super::message_types::AppSocketMessage;

pub const STORED_MESSAGES: &str = "STORED_MESSAGES";

pub async fn manage(owned_handle: tauri::AppHandle) {
    let handle = &owned_handle;
    let duration = std::time::Duration::from_secs(10);
    loop {
        let handle = handle.clone();
        let url = {
            let (judge_id, permenant_id, maybe_token) = {
                let state = handle.state::<ManagedApplicationState>();
                let Ok(app_state) = state.read() else {
                    tokio::time::sleep(duration).await;
                    continue;
                };
                (
                    app_state
                        .get_judge()
                        .map_or_else(String::new, |x| x.id.id()),
                    app_state.permanent_id,
                    app_state.maybe_token(),
                )
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

        let handle2 = handle.clone();
        let handle3 = std::sync::Arc::new(handle.clone());
        debug!("Starting the socket loop");
        match SocketManagerBuilder::<_, AppSocketMessage>::new(&url)
            .transform_handler(move |msg| {
                // convert to Message if required
                let original_message = Message::new(msg);
                // store message in storage
                {
                    if let Ok(store) = handle2.store(STORE_URI) {
                        let mut prev_messages: Vec<Message<AppSocketMessage>> = store
                            .get(STORED_MESSAGES)
                            .and_then(|msg| serde_json::from_value(msg).ok())
                            .unwrap_or_default();

                        prev_messages.push(original_message.clone());
                        store.set(
                            STORED_MESSAGES,
                            serde_json::to_value(prev_messages).expect("Should be able to parse"),
                        );
                    }
                }
                // return message
                original_message.to_msg()
            })
            .keep_alive_handler({
                let handle3 = std::sync::Arc::clone(&handle3);
                move || {
                    let state = handle3.state::<ManagedApplicationState>();
                    let unlocked_state = state.try_read().ok()?;
                    Some(unlocked_state.clone().wrap())
                }
            })
            .receive_handler(|msg| async move {
                if let Ok(message) = Message::<AppSocketMessage>::try_from_msg(msg) {
                    debug!("{message:?}");
                }
                Ok::<(), tauri::Error>(())
            })
            .run()
        {
            Ok((sender, manager)) => {
                handle.manage(sender);
                manager.await.ok();
                let s = handle.state::<socket_manager::SocketManager<AppSocketMessage>>();
                _ = s.send(AppSocketMessage::NoOp);
            }
            Err(err) => debug!("{err:?}"),
        }
    }
}
