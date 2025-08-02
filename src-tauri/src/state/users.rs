use jsonwebtoken::DecodingKey;

use crate::domain::judge::Judge;
use crate::domain::user::{IntitialUser, TokenClaims, User, UserRole};
use crate::domain::SurrealId;
use crate::logging::Loggable;

use super::API_KEY;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum UserType {
    Judge(Judge, TokenUser),
    Admin(TokenUser),
    NotAuthorised,
}
impl From<&UserType> for UserRoleTag {
    fn from(other: &UserType) -> Self {
        use UserType::*;
        match other {
            Judge(_, _) => Self::Judge,
            Admin(_) => Self::Admin,
            NotAuthorised => Self::NotAuthorised,
        }
    }
}
impl Loggable for UserType {
    fn to_log(&self) -> String {
        let name = match self {
            UserType::Judge(judge, user) => format!(
                "{} {} ({:?})",
                judge.first_name, judge.last_name, user.user.id
            ),
            UserType::Admin(user) => {
                format!("{} ({:?})", user.user.username.to_string(), user.user.id)
            }
            UserType::NotAuthorised => unreachable!(),
        };
        format!("User: {name}")
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

#[derive(PartialEq, Clone, Copy)]
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
    pub refresh_token: String,
    pub token: String,
}
