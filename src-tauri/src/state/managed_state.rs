use std::sync::{Arc, RwLock};

use tauri::Manager;

use crate::commands::fetch::{fetch, Method};
use crate::commands::replace_director::ReplaceDirector;
use crate::debug;
use crate::state::application_state::Debouncer;
use crate::state::store::Storable;
use crate::state::users::Tokens;
use crate::templates::error::screen_error;

use super::application_state::{ApplicationId, ApplicationState};

pub struct ManagedApplicationState(std::sync::Arc<std::sync::RwLock<ApplicationState>>);
impl ManagedApplicationState {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(ApplicationState::new())))
    }
    pub fn initialize(app_handle: tauri::AppHandle) {
        // Retrieve new managed state and setup
        let new_state = ManagedApplicationState::new();
        //.state::<ManagedApplicationState>();

        // Setup store and get application ID
        let app_id = ApplicationId::retrieve(&app_handle).unwrap_or_else(|| {
            let app_id = ApplicationId::new();
            app_id.store(&app_handle);
            app_id
        });

        match ApplicationState::retrieve(&app_handle) {
            Some(old_state) => {
                // previous state, recover it and store it in application
                // state for quick access
                debug!(
                    green,
                    "\tApp({}) - Judge = {:?} - Page = {:?}",
                    old_state.permanent_id,
                    old_state.get_judge(),
                    old_state.page
                );
                new_state.write(move |x| {
                    // Overwrite portions of the application
                    // state with stored values
                    *x = ApplicationState {
                        permanent_id: old_state.permanent_id,
                        user: old_state.user,
                        token_expires: old_state.token_expires,
                        show: old_state.show,
                        competition_id: old_state.competition_id,
                        starter_id: old_state.starter_id,
                        page: old_state.page,
                        battery: x.battery.clone(),
                        auto_freestyle: old_state.auto_freestyle,
                        score_debounces: Debouncer::default(),
                        app_handle: x.app_handle.take(), // <-- Copy this from the NEW struct
                                                         // to make sure that we are always
                                                         // using the correct one.
                    }
                })
            }
            None => {
                // no prexisting state, write the new state to disk
                // so it can be referred to next time.
                new_state
                    .read(|x| x.store(&app_handle))
                    .expect("If state error, abort");
                Ok(())
            }
        }
        .expect("That the initial state can be set");
        new_state.add_handle_and_id(app_handle.clone(), app_id);
        app_handle.manage(new_state);
    }
    pub fn get_application_id(&self) -> ApplicationId {
        self.read(|x| x.permanent_id.clone())
            .expect("To get app id")
    }
    pub fn add_handle_and_id(&self, app_handle: tauri::AppHandle, app_id: ApplicationId) {
        self.write(|app_state| {
            app_state.app_handle = Some(app_handle);
            app_state.permanent_id = app_id;
        })
        .expect("To be able to insert app_handle");
    }
    pub async fn read_async<F, R>(&self, f: F) -> Result<R, ReplaceDirector>
    where
        F: FnOnce(&ApplicationState) -> R + Send + 'static,
        R: Send + 'static,
    {
        let inner = self.0.clone();
        tokio::task::spawn_blocking(move || {
            let guard = inner.read();
            let state = match guard {
                Ok(state) => state,
                Err(_err) => {
                    inner.clear_poison();
                    inner
                        .try_read()
                        .map_err(|_| screen_error("Could not read state"))?
                }
            };
            Ok(f(&state))
        })
        .await
        .expect("Spawned handle panicked!")
    }
    pub async fn write_async<F, R>(&self, callback: F) -> Result<R, ReplaceDirector>
    where
        F: FnOnce(&mut ApplicationState) -> R + Send + 'static,
        R: Send + 'static,
    {
        let inner = self.0.clone();
        tokio::task::spawn_blocking(move || {
            let guard = inner.write();
            // try and get state, if not try again
            let mut state = match guard {
                Ok(state) => state,
                Err(_err) => {
                    inner.clear_poison();
                    inner
                        .try_write()
                        .map_err(|_| screen_error("Could not write to state"))?
                }
            };
            let result = callback(&mut state);
            state.store_self()?;
            Ok(result)
        })
        .await
        .expect("Spawned handle panicked!")
    }
    pub fn read<R>(&self, f: impl FnOnce(&ApplicationState) -> R) -> Result<R, ReplaceDirector> {
        let guard = self.0.read();
        let state = match guard {
            Ok(state) => state,
            Err(_err) => {
                self.0.clear_poison();
                self.0
                    .try_read()
                    .map_err(|_| screen_error("Could not read state"))?
            }
        };
        Ok(f(&state))
    }
    pub fn write<R>(
        &self,
        callback: impl FnOnce(&mut ApplicationState) -> R,
    ) -> Result<R, ReplaceDirector> {
        let guard = self.0.write();
        let mut state = match guard {
            Ok(state) => state,
            Err(_err) => {
                self.0.clear_poison();
                self.0
                    .try_write()
                    .map_err(|_| screen_error("Could not read state"))?
            }
        };
        let result = callback(&mut state);
        state.store_self()?;

        Ok(result)
    }
    pub async fn refresh_if_required(&self) -> Result<(), StatefulRequestError> {
        const TEN_MINUTES: i64 = 10 * 60;

        let current_token = self
            .read_async(|app_state| {
                let now_plus_ten = chrono::Utc::now().timestamp() + TEN_MINUTES;
                debug!(
                    "Token Expires - {} \t Time Bound - {}",
                    app_state.token_expires, now_plus_ten
                );
                if app_state.token_expires < now_plus_ten {
                    return Err(app_state.refresh_token());
                }
                Ok(())
            })
            .await?;
        if let Err(refresh_token) = current_token {
            let tokens: Tokens = fetch(Method::Post, concat!(env!("API_URL"), "refresh"), self)
                .body(format!("{{\"refresh\":\"{refresh_token}\"}}"))
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            self.write_async(|app_state| {
                app_state.set_tokens(tokens);
            })
            .await?
        };

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StatefulRequestError {
    #[error(transparent)]
    Http(#[from] tauri_plugin_http::reqwest::Error),
    #[error("There was a problem accessing the application state")]
    State,
    #[error("{0} could not be found")]
    NotFound(&'static str),
}
impl From<ReplaceDirector> for StatefulRequestError {
    fn from(_: ReplaceDirector) -> Self {
        Self::State
    }
}
