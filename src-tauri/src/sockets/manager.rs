use std::sync::Arc;
use tauri_plugin_store::StoreExt;
use tokio::{net::TcpStream, sync::Mutex};

use crate::{state::ManagedApplicationState, STORE_URI};
use tauri::Manager;

use super::messages::SocketMessage;

pub const STORED_MESSAGES: &'static str = "STORED_MESSAGES";

pub async fn manage(owned_handle: tauri::AppHandle) {
    let handle = &owned_handle;
    let state = handle.state::<ManagedApplicationState>();
    loop {
        // TODO: WTF is going on here??
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
        let url = format!(
            "{root}dressage/application/v2/{judge_id}/{permenant_id}?tk={token}",
            root = dotenv_codegen::dotenv!("API_SOCKET"),
            judge_id = judge_id.map_or_else(String::new, |j| j.id())
        );
        socket_manager::SocketManagerBuilder::new(&url)
            .receive_handler(|msg| async move {
                println!("{:?}", msg);
                Ok::<(), tauri::Error>(())
            })
            .run()
            .await;
    }
}
