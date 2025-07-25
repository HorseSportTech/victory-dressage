use serde::ser::SerializeStruct;

pub mod competition;
pub mod competitor;
pub mod dressage_test;
pub mod ground_jury_member;
pub mod judge;
pub mod penalties;
pub mod position;
pub mod scoresheet;
pub mod show;
pub mod starter;
pub mod user;

#[derive(Clone, PartialEq)]
pub struct SurrealId {
    tb: String,
    id: SurrealActualId,
}
impl std::fmt::Debug for SurrealId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let SurrealActualId::String(ref id) = self.id;
        write!(f, "{}:{}", &self.tb, id)
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
enum SurrealActualId {
    String(String),
}

impl SurrealId {
    pub fn id(&self) -> String {
        let SurrealActualId::String(ref id) = self.id;
        id.to_string()
    }
    pub fn make(tb: &str, id: &str) -> Self {
        Self {
            tb: tb.to_string(),
            id: SurrealActualId::String(id.to_string()),
        }
    }
}

#[derive(serde::Deserialize)]
struct IdHelper {
    tb: String,
    id: SurrealActualId,
}
impl serde::Serialize for SurrealId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let s = format!("{}:{}", self.tb, self.id());
            serializer.serialize_str(&s)
        } else {
            let mut state = serializer.serialize_struct("SurrealId", 2)?;
            state.serialize_field("id", &self.id)?;
            state.serialize_field("tb", &self.tb)?;
            state.end()
        }
    }
}
impl<'de> serde::Deserialize<'de> for SurrealId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let (tb, id) = s
                .split_once(":")
                .ok_or_else(|| serde::de::Error::custom("Missing [id] part of Record"))?;
            Ok(Self::make(tb, id))
        } else {
            let IdHelper {
                tb,
                id: SurrealActualId::String(id),
            } = IdHelper::deserialize(deserializer)?;
            Ok(Self::make(&tb, &id))
        }
    }
}
