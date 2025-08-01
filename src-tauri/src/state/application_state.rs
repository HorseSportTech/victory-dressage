use tauri_plugin_store::StoreExt;

use crate::commands::replace_director::ReplaceDirector;
use crate::domain::competition::Competition;
use crate::domain::dressage_test::DressageTest;
use crate::domain::ground_jury_member::GroundJuryMember;
use crate::domain::judge::Judge;
use crate::domain::scoresheet::Scoresheet;
use crate::domain::show::Show;
use crate::domain::starter::Starter;
use crate::domain::SurrealId;
use crate::sockets::message_types::application;
use crate::templates::error::screen_error;
use crate::traits::{Entity, Storable};
use crate::{STATE, STORE_URI};

use super::application_page::ApplicationPage;
use super::battery::VirtualDeviceBattery;
use super::users::Tokens;
use super::UserType;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApplicationState {
    pub permanent_id: ulid::Ulid,
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
    #[serde(skip)]
    pub app_handle: Option<tauri::AppHandle>,
}
impl ApplicationState {
    pub fn new() -> Self {
        Self {
            permanent_id: ulid::Ulid::new(),
            user: UserType::NotAuthorised,
            token_expires: 0,
            show: None,
            competition_id: None,
            starter_id: None,
            page: ApplicationPage::Login,
            battery: VirtualDeviceBattery::new(),
            auto_freestyle: true,
            app_handle: None,
        }
    }
    pub fn store_self(&self) -> Result<(), ReplaceDirector> {
        let save_state = serde_json::to_value(self)
            .map_err(|_| screen_error("Error persisting change to state"))?;
        let store = self
            .app_handle
            .as_ref()
            .expect("Must have app_handle")
            .store(STORE_URI)
            .map_err(|_| screen_error("Failed to retrieve storage"))?;
        store.set(STATE, save_state);
        store
            .save()
            .map_err(|_| screen_error("Error persisting change to state"))
    }
    pub fn token(&self) -> String {
        match self.user {
            UserType::Judge(_, ref user) | UserType::Admin(ref user) => user.token.to_string(),
            _ => String::new(),
        }
    }
    pub fn get_user_id(&self) -> Option<SurrealId> {
        match self.user {
            UserType::Judge(_, ref user) | UserType::Admin(ref user) => Some(user.user.id.clone()),
            _ => None,
        }
    }
    pub fn maybe_token(&self) -> Option<String> {
        match self.user {
            UserType::Judge(_, ref user) | UserType::Admin(ref user) => {
                Some(user.token.to_string())
            }
            _ => None,
        }
    }
    pub fn refresh_token(&self) -> String {
        match self.user {
            UserType::Judge(_, ref user) | UserType::Admin(ref user) => {
                user.user.refresh_token.clone().unwrap_or_default()
            }
            _ => String::new(),
        }
    }
    pub fn set_tokens(&mut self, value: Tokens) {
        match self.user {
            UserType::Judge(_, ref mut user) | UserType::Admin(ref mut user) => {
                user.token = value.token;
                user.user.refresh_token = Some(value.refresh_token);
            }
            _ => (),
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
    // FIXME: To be used
    #[allow(unused)]
    pub fn get_judge_mut(&mut self) -> Option<&mut Judge> {
        match &mut self.user {
            UserType::Judge(judge, _) => Some(judge),
            _ => None,
        }
    }
}
impl Storable for ApplicationState {}
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
            judge_id: self.get_judge().map(|x| x.id.to_owned())?,
            show_id: self.show.map(|x| x.id),
            competition_id: self.competition_id,
            location: self.page,
            state: self.battery,
        })
    }
}
