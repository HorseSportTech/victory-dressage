use self::commands::alert_manager::AlertManager;
use self::commands::bell_timer::Timer;
use state::ManagedApplicationState;
use tauri::{async_runtime as rt, Manager};

mod commands;
mod domain;
mod logging;
mod macros;
mod sockets;
mod state;
mod templates;
mod traits;

const STORE_URI: &str = env!("STORE_URI");
const STATE: &str = "state";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .manage(ManagedApplicationState::new())
        .manage(Timer::default())
        .manage(AlertManager::new())
        .setup(setup_application_state)
        .invoke_handler({
            use commands::*;
            tauri::generate_handler![
                mark_comment::input_mark,
                mark_comment::input_comment,
                mark_comment::input_attempt,
                mark_comment::confirm_attempt,
                mark_comment::edit_attempt,
                logins::login_judge,
                logins::login_user,
                recover::recover,
                log_out::log_out,
                search_for_judge::search_for_judge,
                update_preferences::update_auto_sign,
                update_preferences::update_comment_first,
                update_preferences::update_show_trend,
                signature::draw_signature,
                signature::save_signature,
                navigation::page_x_current,
                navigation::page_x_judge_login,
                navigation::page_x_welcome,
                navigation::page_x_competition_list,
                navigation::page_x_scoresheet,
                navigation::page_x_preferences,
                navigation::page_x_settings,
                navigation::page_x_results,
                warnings::blood::toggle_blood,
                warnings::lameness::toggle_lameness,
                warnings::equipement::toggle_equipment,
                warnings::meeting::toggle_meeting,
                warnings::penalties::plus_error,
                warnings::penalties::sub_error,
                warnings::penalties::plus_technical,
                warnings::penalties::sub_technical,
                warnings::penalties::plus_artistic,
                warnings::penalties::sub_artistic,
                warnings::status::change_competitor_status,
                choose_starter::choose_starter,
                scoresheet::confirm_marks::confirm_marks,
                scoresheet::start_list_bar::filter_starters,
                bell_timer::ring_bell,
                bell_timer::start_normal_time,
                bell_timer::pause_normal_time,
                bell_timer::start_music_time,
                bell_timer::pause_music_time,
                bell_timer::start_test_time_limit,
                bell_timer::pause_test_time_limit,
                update_settings::toggle_freestyle_mode,
                update_settings::clear_data,
                update_settings::download_file,
            ]
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_application_state(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)] // Open dev tray in development build
    app.get_webview_window("main").unwrap().open_devtools();

    let app_handle = app.handle();
    // set up internal state, retriving previous session from memory
    ManagedApplicationState::initialize(app_handle.clone());
    // manage sockets
    rt::spawn(sockets::manager::manage(app_handle.clone()));

    Ok(())
}
