use state::{ApplicationState, ManagedApplicationState};
use std::sync::RwLock;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

use self::commands::alert_manager::AlertManager;
use self::commands::bell_timer::Timer;

mod commands;
mod domain;
mod sockets;
mod state;
mod templates;
mod traits;

const STORE_URI: &'static str = dotenv_codegen::dotenv!("STORE_URI");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(RwLock::new(ApplicationState::new()))
        .manage(Timer::default())
        .manage(AlertManager::new())
        .setup(move |app| {
            #[cfg(debug_assertions)]
            if let Some(w) = app.get_webview_window("main") {
                w.open_devtools();
            };

            let state = app.handle().state::<ManagedApplicationState>();
            match app
                .store(STORE_URI)?
                .get("state")
                .and_then(|x| serde_json::from_value::<ApplicationState>(x).ok())
            {
                Some(s) => {
                    println!("{} - Judge = {:?}", s.permanent_id, s.get_judge());
                    state.write().and_then(|mut w| Ok(*w = s)).ok();
                }
                None => {
                    let value = state.read().expect("To be able to read this");
                    app.store(STORE_URI)?
                        .set("state", serde_json::to_value((*value).clone()).ok())
                }
            };

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move { sockets::manager::manage(handle).await });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::mark_comment::input_mark,
            commands::mark_comment::input_comment,
            commands::mark_comment::input_attempt,
            commands::mark_comment::confirm_attempt,
            commands::mark_comment::edit_attempt,
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
            commands::navigation::page_x_results,
            commands::warnings::blood::toggle_blood,
            commands::warnings::lameness::toggle_lameness,
            commands::warnings::equipement::toggle_equipment,
            commands::warnings::meeting::toggle_meeting,
            commands::warnings::penalties::plus_error,
            commands::warnings::penalties::sub_error,
            commands::warnings::penalties::plus_technical,
            commands::warnings::penalties::sub_technical,
            commands::warnings::penalties::plus_artistic,
            commands::warnings::penalties::sub_artistic,
            commands::warnings::status::change_competitor_status,
            commands::choose_starter::choose_starter,
            commands::scoresheet::confirm_marks::confirm_marks,
            commands::scoresheet::start_list_bar::filter_starters,
            commands::bell_timer::ring_bell,
            commands::bell_timer::start_normal_time,
            commands::bell_timer::pause_normal_time,
            commands::bell_timer::start_music_time,
            commands::bell_timer::pause_music_time,
            commands::bell_timer::start_test_time_limit,
            commands::bell_timer::pause_test_time_limit,
            commands::update_settings::toggle_freestyle_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
