use application_page::ApplicationPage;
use battery::VirtualDeviceBattery;
use dotenv_codegen::dotenv;
use jsonwebtoken::DecodingKey;
use tauri::http::header::{AUTHORIZATION, CONTENT_TYPE};
use tauri_plugin_http::reqwest;

pub mod application_page;
pub mod battery;

use crate::{
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
    sockets::{self, message_types::AppSocketMessage},
    traits::{Entity, Storable},
};
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApplicationState {
    pub permanent_id: ulid::Ulid,
    pub user: UserType,
    #[serde(default)]
    pub token_expires: i64,
    pub show: Option<Show>,
    pub competition: Option<Competition>,
    pub starter: Option<Starter>,
    pub page: ApplicationPage,
    pub battery: VirtualDeviceBattery,
    #[serde(default)]
    pub auto_freestyle: bool,
    // pub socket: WebSocket,??????
}
pub type ManagedApplicationState = std::sync::RwLock<ApplicationState>;

impl ApplicationState {
    pub fn new() -> Self {
        Self {
            permanent_id: ulid::Ulid::new(),
            user: UserType::NotAuthorised,
            token_expires: 0,
            show: None,
            competition: None,
            starter: None,
            page: ApplicationPage::Login,
            battery: VirtualDeviceBattery::new(),
            auto_freestyle: true,
        }
    }
    pub async fn restore() -> Self {
        todo!()
    }
    //pub fn username(&self) -> String {
    //match self.user {
    // UserType::Judge(_, ref user) | UserType::Admin(ref user) => {
    //      user.user.username.to_string()
    //   }
    //    _ => String::new(),
    // }
    //// }
    pub fn token(&self) -> String {
        match self.user {
            UserType::Judge(_, ref user) | UserType::Admin(ref user) => user.token.to_string(),
            _ => String::new(),
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
            UserType::Judge(_, ref user) | UserType::Admin(ref user) => user
                .user
                .refresh_token
                .as_ref()
                .and_then(|x| Some(x.clone()))
                .unwrap_or_default(),
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
    pub async fn refresh(state: &tauri::State<'_, ManagedApplicationState>) -> Result<(), String> {
        let (token, refresh) = {
            let app_state = state
                .read()
                .or_else(|_| {
                    state.clear_poison();
                    state.read()
                })
                .map_err(|e| e.to_string())?;
            if app_state.token_expires > chrono::Utc::now().timestamp() + 10 * 60 * 60 {
                return Ok(());
            }
            (app_state.token(), app_state.refresh_token())
        };
        // println!("{token}, {refresh}");
        let tokens: Tokens = serde_json::from_str(
            &reqwest::Client::new()
                .post(format!("{}refresh", dotenv!("API_URL")))
                .header(CONTENT_TYPE, "Application/json")
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .header("Application-ID", "Victory/Client")
                .body(format!("{{\"refresh\":\"{refresh}\"}}",))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
        )
        .unwrap();

        let mut app_state = state
            .write()
            .or_else(|_| {
                state.clear_poison();
                state.write()
            })
            .map_err(|e| e.to_string())?;

        app_state.set_tokens(tokens);
        Ok(())
    }

    pub fn scoresheet_mut(&mut self) -> Option<&mut Scoresheet> {
        self.starter.as_mut()?.scoresheets.first_mut()
    }
    pub fn scoresheet(&self) -> Option<&Scoresheet> {
        self.starter.as_ref()?.scoresheets.first()
    }
    pub fn get_test(&self) -> Option<&DressageTest> {
        match self.competition {
            Some(ref comp) => match comp.tests.len() {
                0 => None,
                1 => comp.tests.first(),
                _ if self.get_jury_member().is_some_and(|x| x.test.is_some()) => self
                    .get_jury_member()
                    .expect("Should be there")
                    .test
                    .as_ref(),
                _ if self.scoresheet().is_some_and(|x| x.test.is_some()) => {
                    self.scoresheet().expect("Should be there").test.as_ref()
                }
                _ => None,
            },
            None => None,
        }
    }
    pub fn get_jury_member(&self) -> Option<&GroundJuryMember> {
        self.competition.as_ref()?.jury.first()
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum UserType {
    Judge(Judge, TokenUser),
    Admin(TokenUser),
    NotAuthorised,
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

const API_KEY: &'static str = dotenv_codegen::dotenv!("API_KEY");

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
        &token,
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
    pub fn wrap(self) -> sockets::messages::SocketMessage {
        sockets::messages::SocketMessage::new(AppSocketMessage::ApplicationState {
            id: ulid::Ulid::new(),
            judge_id: self
                .get_judge()
                .and_then(|x| Some(x.id.to_owned()))
                .unwrap(),
            show_id: self.show.and_then(|x| Some(x.id)),
            competition_id: self.competition.and_then(|x| Some(x.id)),
            location: self.page,
            state: self.battery,
            competitor_name: None,
        })
    }
}
