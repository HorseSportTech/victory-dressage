use std::sync::{Arc, RwLock};

use application_page::ApplicationPage;
use battery::VirtualDeviceBattery;
use jsonwebtoken::DecodingKey;
use tauri::http::header::AUTHORIZATION;

pub mod application_page;
pub mod battery;

use crate::{
    commands::{
        fetch::{fetch, Method},
        replace_director::ReplaceDirector,
    },
    domain::{
        competition::Competition,
        dressage_test::DressageTest,
        ground_jury_member::GroundJuryMember,
        judge::Judge,
        scoresheet::Scoresheet,
        show::Show,
        starter::Starter,
        user::{IntitialUser, TokenClaims, User, UserRole},
        SurrealId,
    },
    sockets::message_types::application,
    templates::error::screen_error,
    traits::{Entity, Storable},
};
const API_KEY: &str = env!("API_KEY");

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
    // pub socket: WebSocket,??????
}
pub struct ManagedApplicationState(std::sync::Arc<std::sync::RwLock<ApplicationState>>);
impl ManagedApplicationState {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(ApplicationState::new())))
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
    pub async fn write_async<F, R>(&self, f: F) -> Result<R, ReplaceDirector>
    where
        F: FnOnce(&mut ApplicationState) -> R + Send + 'static,
        R: Send + 'static,
    {
        let inner = self.0.clone();
        tokio::task::spawn_blocking(move || {
            let guard = inner.write();
            let mut state = match guard {
                Ok(state) => state,
                Err(_err) => {
                    inner.clear_poison();
                    inner
                        .try_write()
                        .map_err(|_| screen_error("Could not write to state"))?
                }
            };
            Ok(f(&mut state))
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
        f: impl FnOnce(&mut ApplicationState) -> R,
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
        Ok(f(&mut state))
    }
    pub async fn refresh(&self) -> Result<(), String> {
        const TEN_MINUTES: i64 = 10 * 60;
        let Err((token, refresh_token)) = self
            .read_async(|app_state| {
                if app_state.token_expires > chrono::Utc::now().timestamp() + TEN_MINUTES {
                    return Ok(());
                }
                Err((app_state.token(), app_state.refresh_token()))
            })
            .await
            .map_err(|_| "Could not read state")?
        else {
            return Ok(());
        };

        let tokens: Tokens = fetch(Method::Post, concat!(env!("API_URL"), "refresh"), self)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .body(format!("{{\"refresh\":\"{refresh_token}\"}}"))
            .send()
            .await
            .map_err(|err| err.to_string())?
            .error_for_status()
            .map_err(|err| err.to_string())?
            .json()
            .await
            .map_err(|err| err.to_string())?;

        self.write_async(|app_state| {
            app_state.set_tokens(tokens);
        })
        .await
        .map_err(|_| String::new())?;
        Ok(())
    }
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
        }
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
    pub fn competition_mut(&self) -> Option<&Competition> {
        let id = self.competition_id.as_ref()?;
        let show = self.show.as_ref()?;
        show.competitions.iter().find(|x| x.id == *id)
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

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum UserType {
    Judge(Judge, TokenUser),
    Admin(TokenUser),
    NotAuthorised,
}
#[derive(Copy, Clone)]
pub enum UserTypeOnly {
    Judge,
    Admin,
    NotAuthorised,
}
impl From<&UserType> for UserTypeOnly {
    fn from(other: &UserType) -> Self {
        use UserType::*;
        match other {
            Judge(_, _) => Self::Judge,
            Admin(_) => Self::Admin,
            NotAuthorised => Self::NotAuthorised,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct TokenUser {
    pub token: String,
    pub user: User,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct InitialTokenUser {
    pub token: String,
    pub user: IntitialUser,
}
impl TryFrom<InitialTokenUser> for TokenUser {
    type Error = String;

    fn try_from(other: InitialTokenUser) -> Result<Self, Self::Error> {
        let Ok(claims) = decode_token(&other.token) else {
            return Err(String::from("Cannot decode"));
        };
        Ok(TokenUser {
            token: other.token,
            user: User {
                id: SurrealId::make("user", claims.claims.user_id.to_string().as_str()),
                username: other.user.username,
                email: other.user.email,
                refresh_token: other.user.refresh_token,
            },
        })
    }
}

#[derive(PartialEq)]
pub enum UserRoleTag {
    Judge,
    Admin,
    NotAuthorised,
}

impl TokenUser {
    pub fn get_role_for_user(self) -> UserRoleTag {
        let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512);
        let Ok(data) = jsonwebtoken::decode::<TokenClaims>(
            &self.token,
            &DecodingKey::from_secret(API_KEY.as_bytes()),
            &validation,
        ) else {
            return UserRoleTag::NotAuthorised;
        };
        if self.user.username != data.claims.username {
            return UserRoleTag::NotAuthorised;
        }

        match data.claims.role {
            UserRole::Official => UserRoleTag::Judge,
            UserRole::Scorer | UserRole::ShowOffice | UserRole::Admin => UserRoleTag::Admin,
            _ => UserRoleTag::NotAuthorised,
        }
    }
}

pub fn decode_token(
    token: &str,
) -> Result<jsonwebtoken::TokenData<TokenClaims>, jsonwebtoken::errors::Error> {
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512);
    jsonwebtoken::decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(API_KEY.as_bytes()),
        &validation,
    )
}

#[derive(serde::Deserialize)]
pub struct Tokens {
    refresh_token: String,
    token: String,
}

impl ApplicationState {
    pub fn wrap(self) -> application::Payload {
        application::Payload::ApplicationState {
            id: ulid::Ulid::new(),
            judge_id: self.get_judge().map(|x| x.id.to_owned()).unwrap(),
            show_id: self.show.map(|x| x.id),
            competition_id: self.competition_id,
            location: self.page,
            state: self.battery,
        }
    }
}
