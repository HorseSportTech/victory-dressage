use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::oneshot;
use tokio::time::sleep;

use crate::commands::replace_director::ReplaceDirector;
use crate::debug;
use crate::domain::competition::Competition;
use crate::domain::dressage_test::DressageTest;
use crate::domain::ground_jury_member::GroundJuryMember;
use crate::domain::judge::Judge;
use crate::domain::scoresheet::Scoresheet;
use crate::domain::show::Show;
use crate::domain::starter::Starter;
use crate::domain::SurrealId;
use crate::sockets::message_types::application;
use crate::state::users::decode_token;
use crate::traits::Entity;

use super::application_page::ApplicationPage;
use super::battery::VirtualDeviceBattery;
use super::users::{TokenUser, Tokens, UserType};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApplicationState {
    pub permanent_id: ApplicationId,
    pub user: UserType,
    #[serde(default)]
    pub token_expires: i64,
    pub show: Option<Show>,
    pub competition_id: Option<SurrealId>,
    pub starter_id: Option<SurrealId>,
    pub page: ApplicationPage,
    pub battery: VirtualDeviceBattery,
    #[serde(default)]
    pub auto_freestyle: bool,
    #[serde(skip, default)]
    pub app_handle: Option<tauri::AppHandle>,
    #[serde(skip, default)]
    pub score_debounces: Debouncer,
}
impl ApplicationState {
    pub fn new() -> Self {
        debug!("Creating new Application State");
        Self {
            permanent_id: ApplicationId::new(),
            user: UserType::NotAuthorised,
            token_expires: 0,
            show: None,
            competition_id: None,
            starter_id: None,
            page: ApplicationPage::Login,
            battery: VirtualDeviceBattery::new(),
            auto_freestyle: true,
            app_handle: None,
            score_debounces: Debouncer::default(),
        }
    }
    pub fn store_self(&self) -> Result<(), ReplaceDirector> {
        if let Some(ref handle) = self.app_handle {
            super::store::Storable::store(self, handle);
        }
        Ok(())
    }
    pub fn token(&self) -> String {
        self.maybe_token().unwrap_or_default()
    }
    #[allow(unused)]
    pub fn get_user_id(&self) -> Option<SurrealId> {
        self.get_tokenuser().map(|u| u.user.id.clone())
    }
    pub fn get_tokenuser_mut(&mut self) -> Option<&mut TokenUser> {
        match self.user {
            UserType::Judge(_, ref mut user) | UserType::Admin(ref mut user) => Some(user),
            _ => None,
        }
    }
    pub fn get_tokenuser(&self) -> Option<&TokenUser> {
        match self.user {
            UserType::Judge(_, ref user) | UserType::Admin(ref user) => Some(user),
            _ => None,
        }
    }
    pub fn maybe_token(&self) -> Option<String> {
        self.get_tokenuser().map(|u| u.token.to_string())
    }
    pub fn refresh_token(&self) -> String {
        self.get_tokenuser()
            .and_then(|u| u.user.refresh_token.clone())
            .unwrap_or_default()
    }
    pub fn set_tokens(&mut self, value: Tokens) {
        if let Ok(parsed_token) = decode_token(&value.token) {
            self.token_expires = parsed_token.claims.exp;
        }
        if let Some(user) = self.get_tokenuser_mut() {
            user.token = value.token;
            user.user.refresh_token = Some(value.refresh_token);
        };
    }
    pub fn competition(&self) -> Option<&Competition> {
        let id = self.competition_id.as_ref()?;
        let show = self.show.as_ref()?;
        show.competitions.iter().find(|x| x.id == *id)
    }
    // FIXME: To be used
    #[allow(unused)]
    pub fn competition_mut(&self) -> Option<&Competition> {
        let id = self.competition_id.as_ref()?;
        let show = self.show.as_ref()?;
        show.competitions.iter().find(|x| x.id == *id)
    }
    pub fn starter_from_sheet_ulid_mut(&mut self, ulid: &ulid::Ulid) -> Option<&mut Starter> {
        let show = self.show.as_mut()?;
        let mut competitor = None;
        'outer: for competition in show.competitions.iter_mut() {
            for starter in competition.starters.iter_mut() {
                for scoresheet in starter.scoresheets.iter_mut() {
                    if scoresheet.id.ulid() == *ulid {
                        competitor = Some(starter);
                        break 'outer;
                    }
                }
            }
        }
        competitor
    }
    pub fn starter(&self) -> Option<&Starter> {
        let id = self.starter_id.as_ref()?;
        let show = self.show.as_ref()?;
        let mut competitor = None;
        'outer: for competition in show.competitions.iter() {
            for starter in competition.starters.iter() {
                if starter.id == *id {
                    competitor = Some(starter);
                    break 'outer;
                }
            }
        }
        competitor
    }
    pub fn starter_mut(&mut self) -> Option<&mut Starter> {
        let id = self.starter_id.as_mut()?;
        let show = self.show.as_mut()?;
        let mut competitor = None;
        'outer: for competition in show.competitions.iter_mut() {
            for starter in competition.starters.iter_mut() {
                if starter.id == *id {
                    competitor = Some(starter);
                    break 'outer;
                }
            }
        }
        competitor
    }

    pub fn scoresheet_mut(&mut self) -> Option<&mut Scoresheet> {
        self.starter_mut()?.scoresheets.first_mut()
    }
    pub fn scoresheet(&self) -> Option<&Scoresheet> {
        self.starter()?.scoresheets.first()
    }
    pub fn get_test(&self) -> Option<&DressageTest> {
        match self.competition() {
            None => None,
            Some(comp) => match comp.tests.len() {
                0 => None,
                1 => comp.tests.first(),
                _ if self.get_jury_member().is_some_and(|x| x.test.is_some()) => {
                    self.get_jury_member().and_then(|x| x.test.as_ref())
                }
                _ if self.scoresheet().is_some_and(|x| x.test.is_some()) => {
                    self.scoresheet().and_then(|x| x.test.as_ref())
                }
                _ => None,
            },
        }
    }
    pub fn get_jury_member(&self) -> Option<&GroundJuryMember> {
        self.competition()?.jury.first()
    }

    pub fn get_judge(&self) -> Option<&Judge> {
        match &self.user {
            UserType::Judge(judge, _) => Some(judge),
            _ => None,
        }
    }
    pub fn get_judge_id(&self) -> Option<&SurrealId> {
        self.get_judge().map(|x| &x.id)
    }
    pub fn get_judge_mut(&mut self) -> Option<&mut Judge> {
        match &mut self.user {
            UserType::Judge(judge, _) => Some(judge),
            _ => None,
        }
    }
}
impl Entity for ApplicationState {
    fn key(&self) -> String {
        String::from("state")
    }
    fn get_id(&self) -> String {
        String::from("state")
    }
}

impl ApplicationState {
    pub fn wrap(self) -> Option<application::Payload> {
        Some(application::Payload::ApplicationState {
            id: ulid::Ulid::new(),
            judge_id: self.get_judge_id()?.clone(),
            show_id: self.show.map(|x| x.id),
            competition_id: self.competition_id,
            location: self.page,
            state: self.battery,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Debouncer(Arc<Mutex<HashMap<u16, tokio::sync::oneshot::Sender<bool>>>>);
impl Default for Debouncer {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}
impl Debouncer {
    pub fn debounce<F>(&self, index: u16, delay: Duration, callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut tasks = self.0.lock().unwrap();

        if let Some(cancel_sender) = tasks.remove(&index) {
            let _ = cancel_sender.send(false);
        }

        let (cancel_tx, cancel_rx) = oneshot::channel();
        tasks.insert(index, cancel_tx);

        tauri::async_runtime::spawn({
            let tasks = Arc::clone(&self.0);
            async move {
                tokio::select! {
                    _ = sleep(delay) => {
                        callback();
                        tasks.lock().unwrap().remove(&index);
                    },
                    execute = cancel_rx => {
                        if execute.is_ok_and(|x| x) {callback()}
                    }
                }
            }
        });
    }
    pub fn cancel(&self, index: u16) {
        let mut tasks = self.0.lock().unwrap();
        if let Some(cancel_sender) = tasks.remove(&index) {
            let _ = cancel_sender.send(false);
        }
    }
    pub fn execute_immediately(&self, index: u16) {
        let mut tasks = self.0.lock().unwrap();
        if let Some(cancel_sender) = tasks.remove(&index) {
            let _ = cancel_sender.send(true);
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ApplicationId(ulid::Ulid);
impl ApplicationId {
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }
}
impl Deref for ApplicationId {
    type Target = ulid::Ulid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ApplicationId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl std::fmt::Display for ApplicationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<ulid::Ulid> for ApplicationId {
    fn from(value: ulid::Ulid) -> Self {
        Self(value)
    }
}
