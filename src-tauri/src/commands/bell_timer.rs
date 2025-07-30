use super::{
    replace_director::{ReplaceDirector, ResponseDirector},
    PAGE_UPDATE,
};
use crate::{commands::replace_director::PageLocation, templates::icons};
use crate::{
    state::ManagedApplicationState,
    templates::{error::screen_error, html_elements, TxAttributes},
};
use hypertext::{rsx_move, GlobalAttributes, Renderable};
use std::sync::{Arc, Mutex};
use tauri::Emitter as _;

#[tauri::command]
pub async fn ring_bell(_app: tauri::AppHandle) -> Result<(), String> {
    // will eventually ring bell
    Ok(())
}

#[derive(Default)]
pub struct InnerTimer {
    normal_running: bool,
    music_running: bool,
    normal_counter: u8,
    music_counter: u8,
    test_time_running: bool,
    test_time_counter: i16,
}
pub struct Timer(Arc<Mutex<InnerTimer>>);
impl Default for Timer {
    fn default() -> Self {
        Timer(Arc::new(Mutex::new(InnerTimer::default())))
    }
}
impl Clone for Timer {
    fn clone(&self) -> Self {
        Timer(self.0.clone())
    }
}
impl Timer {
    pub fn start_normal(&self, time: u8) {
        let mut inner = self.0.lock().expect("To be able to lock");
        inner.normal_running = true;
        inner.music_running = false;
        inner.test_time_running = false;
        inner.normal_counter = time;
    }
    pub fn sub_normal(&self) -> u8 {
        let mut inner = self.0.lock().expect("To be able to lock");
        if inner.normal_running {
            inner.normal_counter = inner.normal_counter.saturating_sub(1);
        }
        inner.normal_counter
    }
    pub fn get_normal(&self) -> u8 {
        let inner = self.0.lock().expect("To be able to lock");
        inner.normal_counter
    }
    pub fn start_music(&self, time: u8) {
        let mut inner = self.0.lock().expect("To be able to lock");
        inner.normal_running = false;
        inner.music_running = true;
        inner.test_time_running = false;
        inner.music_counter = time;
    }
    pub fn sub_music(&self) -> u8 {
        let mut inner = self.0.lock().expect("To be able to lock");
        if inner.music_running {
            inner.music_counter = inner.music_counter.saturating_sub(1);
        }
        inner.music_counter
    }
    pub fn get_music(&self) -> u8 {
        let inner = self.0.lock().expect("To be able to lock");
        inner.music_counter
    }
    pub fn start_test_time(&self, time: i16) {
        let mut inner = self.0.lock().expect("To be able to lock");

        inner.normal_running = false;
        inner.music_running = false;
        inner.test_time_running = true;
        inner.test_time_counter = time;
    }
    pub fn sub_test_time(&self) -> i16 {
        let mut inner = self.0.lock().expect("To be able to lock");
        if inner.test_time_running {
            inner.test_time_counter = inner.test_time_counter - 1;
            if inner.test_time_counter <= -30 {
                inner.test_time_counter = -30;
                inner.test_time_running = false;
            }
        }
        inner.test_time_counter
    }
    pub fn get_test_time(&self) -> i16 {
        let inner = self.0.lock().expect("To be able to lock");
        inner.test_time_counter
    }
    pub fn pause_normal(&self) -> bool {
        let mut inner = self.0.lock().expect("To be able to lock");
        inner.normal_running = !inner.normal_running;
        inner.normal_running
    }
    pub fn pause_music(&self) -> bool {
        let mut inner = self.0.lock().expect("To be able to lock");
        inner.music_running = !inner.music_running;
        inner.music_running
    }
    pub fn pause_test_time(&self) -> bool {
        let mut inner = self.0.lock().expect("To be able to lock");
        inner.test_time_running = !inner.test_time_running;
        inner.test_time_running
    }
    pub fn is_running_normal(&self) -> bool {
        self.0.lock().expect("To be able to lock").normal_running
    }
    pub fn is_running_music(&self) -> bool {
        self.0.lock().expect("To be able to lock").music_running
    }
    pub fn is_running_test_time(&self) -> bool {
        self.0.lock().expect("To be able to lock").test_time_running
    }
    pub fn formatted_test_time(&self) -> String {
        let inner = self.0.lock().expect("To be able to lock");
        let minutes = i16::abs(inner.test_time_counter / 60);
        let seconds = i16::abs(inner.test_time_counter % 60);
        format!("{minutes}:{seconds:02}")
    }
}

#[tauri::command]
pub async fn start_normal_time(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    state_timer: tauri::State<'_, Timer>,
) -> ResponseDirector {
    let timer_value = state
        .read_async(|app_state| {
            let competition = app_state
                .competition
                .as_ref()
                .ok_or_else(|| screen_error("No competition found"))?;
            let judge = competition
                .jury
                .first()
                .ok_or_else(|| screen_error("Judge not found"))?;
            let [timer_value, _] = competition.get_test(judge).countdowns;

            Ok(timer_value)
        })
        .await??;
    state_timer.start_normal(timer_value + 1);
    let timer = state_timer.inner().clone();
    tauri::async_runtime::spawn(async move {
        let mut delay = tokio::time::interval(std::time::Duration::from_secs(1));

        loop {
            delay.tick().await;
            let timer_value = timer.sub_normal();
            if timer.is_running_normal() {
                app.emit(
                    PAGE_UPDATE,
                    ReplaceDirector::with_target(
                        &PageLocation::NormalCountdown,
                        match timer_value {
                            n @ 1.. => rsx_move!{
                                <button tx-command="pause_normal_time" style="background:orange">{&n}" sec"</button>
                            }.render(),
                            0 => rsx_move!{
                                <button tx-command="start_normal_time" style="background:red">"OUT"</button>
                            }.render(),
                        }
                    )
                ).ok();
            }
            if timer_value == 0 {
                break;
            }
        }
    });
    Ok(ReplaceDirector::none())
}
#[tauri::command]
pub async fn start_music_time(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    state_timer: tauri::State<'_, Timer>,
) -> ResponseDirector {
    let timer_value = state
        .read_async(|app_state| {
            let competition = app_state
                .competition
                .as_ref()
                .ok_or_else(|| screen_error("No competition found"))?;
            let judge = competition
                .jury
                .first()
                .ok_or_else(|| screen_error("Judge not found"))?;
            let [_, timer_value] = competition.get_test(judge).countdowns;
            Ok(timer_value)
        })
        .await??;
    state_timer.start_music(timer_value + 1);

    let timer = state_timer.inner().clone();
    tauri::async_runtime::spawn(async move {
        let mut delay = tokio::time::interval(std::time::Duration::from_secs(1));

        loop {
            delay.tick().await;
            let timer_value = timer.sub_music();
            if timer.is_running_music() {
                app.emit(
                    PAGE_UPDATE,
                    ReplaceDirector::with_target(
                        &PageLocation::MusicCountdown,
                        match timer_value {
                            n @ 1.. => rsx_move!{
                                <button tx-command="pause_music_time" style="background:orange">{&n}" sec"</button>
                            }.render(),
                            0 => rsx_move!{
                                <button tx-command="start_music_time" style="background:red">"OUT"</button>
                            }.render(),
                        }
                    )
                ).ok();
            }
            if timer_value == 0 {
                break;
            }
        }
    });
    Ok(ReplaceDirector::none())
}
#[tauri::command]
pub async fn start_test_time_limit(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    state_timer: tauri::State<'_, Timer>,
) -> ResponseDirector {
    let timer_value = state
        .read_async(|app_state| {
            let competition = app_state
                .competition
                .as_ref()
                .ok_or_else(|| screen_error("No competition found"))?;
            let judge = competition
                .jury
                .first()
                .ok_or_else(|| screen_error("Judge not found"))?;
            let timer_value = competition.get_test(judge).length_in_seconds;
            Ok(timer_value)
        })
        .await??;
    state_timer.start_test_time(timer_value as i16);

    let timer = state_timer.inner().clone();
    tauri::async_runtime::spawn(async move {
        let mut delay = tokio::time::interval(std::time::Duration::from_secs(1));

        loop {
            delay.tick().await;
            let timer_value = timer.sub_test_time();
            let formatted_time = timer.formatted_test_time();
            if timer.is_running_test_time() {
                app.emit(
                    PAGE_UPDATE,
                    ReplaceDirector::with_target(
                        &PageLocation::TestTimeCountdown,
                        match timer_value {
                            31.. => rsx_move!{
                                <button tx-command="pause_test_time_limit" style="background:red">{&formatted_time}" Short"</button>
                            }.render(),
                            0..=30 => rsx_move!{
                                <button tx-command="pause_test_time_limit" style="background:orange">{&formatted_time}" Ok"</button>
                            }.render(),
                            i16::MIN..=-1 => rsx_move!{
                               <button tx-command="pause_test_time_limit" style="background:red">"-"{&formatted_time}" Long"</button>
                            }.render(),
                        }
                    )
                ).ok();
            }
            if timer_value <= -30 {
                break;
            }
        }
    });
    Ok(ReplaceDirector::none())
}

#[tauri::command]
pub async fn pause_normal_time(state_timer: tauri::State<'_, Timer>) -> ResponseDirector {
    let running = state_timer.pause_normal();
    let n = state_timer.get_normal();
    Ok(ReplaceDirector::with_target(
        &PageLocation::NormalCountdown,
        rsx_move!{
        <button tx-command="pause_normal_time" style="background:orange">{&n}@if running {" sec"} @else {" "{icons::PAUSE}}</button>
        }.render()
    ))
}
#[tauri::command]
pub async fn pause_music_time(state_timer: tauri::State<'_, Timer>) -> ResponseDirector {
    let running = state_timer.pause_music();
    let n = state_timer.get_music();
    Ok(ReplaceDirector::with_target(
        &PageLocation::MusicCountdown,
        rsx_move!{
        <button tx-command="pause_music_time" style="background:orange">{&n}@if running {" sec"} @else {" "{icons::PAUSE}}</button>
        }.render()
    ))
}
#[tauri::command]
pub async fn pause_test_time_limit(state_timer: tauri::State<'_, Timer>) -> ResponseDirector {
    let running = state_timer.pause_test_time();
    let n = state_timer.formatted_test_time();
    Ok(ReplaceDirector::with_target(
        &PageLocation::TestTimeCountdown,
        rsx_move!{
        <button tx-command="pause_test_time_limit" style="background:orange">{&n}@if running {" Short"} @else {" "{icons::PAUSE}}</button>
        }.render()
    ))
}
