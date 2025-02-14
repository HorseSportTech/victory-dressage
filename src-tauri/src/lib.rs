use state::{ApplicationState, ManagedApplicationState};
use tauri::{Manager};
use tauri_plugin_store::StoreExt;
use std::sync::RwLock;
mod templates;
mod state;
mod domain;
mod traits;
mod commands;

const STORE_URI: &'static str = dotenv_codegen::dotenv!("STORE_URI");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(RwLock::new(ApplicationState::new()))
        .setup(move |app| {
            #[cfg(debug_assertions)]
            if let Some(w) = app.get_webview_window("main") {
                w.open_devtools();
            };
            let state = app.handle().state::<ManagedApplicationState>();
            match app.store(STORE_URI)?.get("state") {
                Some(s) => {
                    println!("{s:?}");
                    state.write()
                        .and_then(|mut w| Ok(*w = serde_json::from_value::<ApplicationState>(s).expect("Failed to parse")))
                        .ok();
                }
                None => {
                    let value = state.read().expect("To be able to read this");
                    app.store(STORE_URI)?.set("state", 
                        serde_json::to_value((*value).clone()).ok())
                },
            }


            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::mark_comment::input_mark,
            commands::mark_comment::input_comment,
            commands::logins::login_judge,
            commands::logins::login_user,
            commands::recover::recover,

            commands::log_out::log_out,

            commands::search_for_judge::search_for_judge,

            commands::update_preferences::update_auto_sign,
            commands::update_preferences::update_comment_first,
            commands::update_preferences::update_show_trend,
            commands::signature::draw_signature,
            commands::signature::save_signature,

            commands::navigation::page_x_current,
            commands::navigation::page_x_judge_login,
            commands::navigation::page_x_welcome,
            commands::navigation::page_x_competition_list,
            commands::navigation::page_x_scoresheet,
            commands::navigation::page_x_preferences,
            commands::navigation::page_x_settings,

        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
