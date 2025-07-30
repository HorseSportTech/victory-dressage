use decimal::Decimal;

use crate::commands::alert_manager::Alert;

use super::{competitor::Competitor, scoresheet::Scoresheet, SurrealId};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Starter {
    pub id: SurrealId,
    pub competitor: Competitor,
    #[serde(
        deserialize_with = "decimal::parsing::deserialize_opt_from_f64",
        serialize_with = "decimal::parsing::serialize_opt_as_f64"
    )]
    pub score: Option<Decimal>,
    pub status: StarterResult,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub number: u16,
    pub index: u16,
    pub scoresheets: Vec<Scoresheet>,
    #[serde(default)]
    pub warnings: Vec<Alert>,
}

impl Starter {
    pub fn score_or_number(&self) -> String {
        match self.status {
            StarterResult::Upcoming => self.number.to_string(),
            StarterResult::InProgress(_) => {
                if let Some(scoresheet) = self.scoresheets.first() {
                    format!("{:.3}", scoresheet.score.unwrap_or_default())
                } else {
                    format!("{:.3}", self.score.unwrap_or_default())
                }
            }
            StarterResult::Placed(_) | StarterResult::NotPlaced(_) => {
                format!("{:.3}", self.score.unwrap_or_default())
            }
            StarterResult::Eliminated(_) => "Elim".to_string(),
            StarterResult::Withdrawn => "Wdn".to_string(),
            StarterResult::NoShow => "NS".to_string(),
            StarterResult::Retired => "Ret".to_string(),
            StarterResult::Disqualified => "Dsq".to_string(),
        }
    }
    pub fn time_or_rank(&self) -> String {
        match self.status {
            StarterResult::InProgress(r) => format!("Trend {r}"),
            StarterResult::Placed(r) | StarterResult::NotPlaced(r) => format!("Rk {r}"),
            StarterResult::Upcoming => self.start_time.format("%H:%M").to_string(),
            _ => String::new(),
        }
    }
    pub fn name(&self) -> String {
        format!(
            "{} {}",
            self.competitor.first_name, self.competitor.last_name
        )
    }
    pub fn horse(&self) -> String {
        self.competitor.horse_name.to_string()
    }
    pub fn matches_sheet_ulid(&self, other_id: &ulid::Ulid) -> bool {
        self.scoresheets
            .first()
            .is_some_and(|x| x.id.ulid() == *other_id)
    }
    pub fn matches_ulid(&self, other_id: &ulid::Ulid) -> bool {
        self.id.ulid() == *other_id
    }
    pub fn matches_sheet_id(&self, other_id: &SurrealId) -> bool {
        self.scoresheets.first().is_some_and(|x| x.id == *other_id)
    }
    pub fn matches_id(&self, other_id: &SurrealId) -> bool {
        self.id == *other_id
    }
}

impl crate::traits::Storable for Starter {}
impl crate::traits::Entity for Starter {
    fn key(&self) -> String {
        format!("{}:{}", self.id.tb, self.id.id())
    }
    fn get_id(&self) -> String {
        self.id.id()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StarterResult {
    Upcoming,
    InProgress(u16),
    Placed(u16),
    NotPlaced(u16),
    Eliminated(String),
    Withdrawn,
    NoShow,
    Retired,
    Disqualified,
}

impl StarterResult {
    pub fn rank(&self) -> Option<u16> {
        match self {
            Self::InProgress(r) | Self::Placed(r) | Self::NotPlaced(r) => Some(*r),
            _ => None,
        }
    }

    pub fn abbreviate(&self) -> String {
        match self {
            StarterResult::Upcoming => String::new(),
            StarterResult::InProgress(r) => format!("({r})"),
            StarterResult::Placed(r) => r.to_string(),
            StarterResult::NotPlaced(r) => r.to_string(),
            StarterResult::Eliminated(_) => "EL".to_string(),
            StarterResult::Withdrawn => "WD".to_string(),
            StarterResult::NoShow => "NS".to_string(),
            StarterResult::Retired => "RT".to_string(),
            StarterResult::Disqualified => "DQ".to_string(),
        }
    }

    pub fn list_abbreviation(&self) -> String {
        match self {
            StarterResult::Upcoming | StarterResult::InProgress(_) => String::new(),
            StarterResult::Placed(_)
            | StarterResult::NotPlaced(_)
            | StarterResult::Eliminated(_)
            | StarterResult::Withdrawn
            | StarterResult::NoShow
            | StarterResult::Retired
            | StarterResult::Disqualified => "DONE".to_string(),
        }
    }

    pub fn is_finished(&self) -> bool {
        match self {
            StarterResult::Upcoming | StarterResult::InProgress(_) => false,
            StarterResult::Placed(_)
            | StarterResult::NotPlaced(_)
            | StarterResult::Eliminated(_)
            | StarterResult::Withdrawn
            | StarterResult::NoShow
            | StarterResult::Retired
            | StarterResult::Disqualified => true,
        }
    }
}
impl std::fmt::Display for StarterResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Upcoming => "Upcoming",
                Self::InProgress(_) => "InProgress",
                Self::Placed(_) => "Placed",
                Self::NotPlaced(_) => "NotPlaced",
                Self::Eliminated(_) => "Eliminated",
                Self::Withdrawn => "Withdrawn",
                Self::NoShow => "NoShow",
                Self::Retired => "Retired",
                Self::Disqualified => "Disqualified",
            }
        )
    }
}

impl serde::Serialize for StarterResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self {
            Self::Upcoming => (["Upcoming"]).serialize(serializer),
            Self::InProgress(r) => ("InProgress", r).serialize(serializer),
            Self::NoShow => (["NoShow"]).serialize(serializer),
            Self::Withdrawn => (["Withdrawn"]).serialize(serializer),
            Self::Eliminated(s) => ("Eliminated", s).serialize(serializer),
            Self::Disqualified => (["Disqualified"]).serialize(serializer),
            Self::Retired => (["Retired"]).serialize(serializer),
            Self::Placed(r) => ("Placed", r).serialize(serializer),
            Self::NotPlaced(r) => ("NotPlaced", r).serialize(serializer),
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum VariableSize {
    Single([String; 1]),
    Double(String, u16),
    String(String, String),
}
impl<'de> serde::Deserialize<'de> for StarterResult {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = VariableSize::deserialize(de).map_err(serde::de::Error::custom)?;

        match val {
            VariableSize::Single([tag]) => match tag.as_str() {
                "Upcoming" => Ok(Self::Upcoming),
                "NoShow" => Ok(Self::NoShow),
                "Withdrawn" => Ok(Self::Withdrawn),
                "Retired" => Ok(Self::Retired),
                "Disqualified" => Ok(Self::Disqualified),
                _ => Err(serde::de::Error::custom(
                    "Did not match an acceptable pattern during deserialization",
                )),
            },
            VariableSize::String(tag, value) => match (tag.as_str(), value) {
                ("Eliminated", val) => Ok(Self::Eliminated(val)),
                (_, value) => {
                    eprintln!("Eliminated for {value} did not match an acceptable value");
                    Err(serde::de::Error::custom(
                        "Did not match an acceptable pattern during deserialization",
                    ))
                }
            },
            VariableSize::Double(tag, value) => match (tag.as_str(), value) {
                ("InProgress", num) => Ok(Self::InProgress(num)),
                ("Placed", num) => Ok(Self::Placed(num)),
                ("NotPlaced", num) => Ok(Self::NotPlaced(num)),
                _ => Err(serde::de::Error::custom(
                    "Did not match an acceptable pattern during deserialization",
                )),
            },
        }
    }
}
