use super::{competitor::Competitor, scoresheet::Scoresheet, SurrealId};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct Starter {
	pub id: SurrealId,
	pub competitor: Competitor,
	pub score: Option<f64>,
	pub status: StarterResult,
	pub start_time: chrono::DateTime<chrono::Utc>,
	pub number: u16,
	pub index: u16,
	pub scoresheets: Vec<Scoresheet>,
}

impl crate::traits::Storable for Starter{}
impl crate::traits::Entity for Starter {
	fn key(&self) -> String {format!("{}:{}", self.id.tb, self.id.id())}
	fn id(&self) -> String {self.id.id()}
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
			Self::InProgress(r) |Self::Placed(r) | Self::NotPlaced(r) => Some(*r),
			_ => None,
		}
	}

	pub fn abbreviate(&self) -> String {
		match self {
			StarterResult::Upcoming => String::new(),
			StarterResult::InProgress(r) => format!("({})", r),
			StarterResult::Placed(r) => r.to_string(),
			StarterResult::NotPlaced(r) => r.to_string(),
			StarterResult::Eliminated(_) => "EL".to_string(),
			StarterResult::Withdrawn => "WD".to_string(),
			StarterResult::NoShow => "NS".to_string(),
			StarterResult::Retired => "RT".to_string(),
			StarterResult::Disqualified => "DQ".to_string(),
		}
	}

	pub fn is_finished(&self) -> bool {
		match self {
			StarterResult::Upcoming | StarterResult::InProgress(_) => true,
			StarterResult::Placed(_) | StarterResult::NotPlaced(_) | StarterResult::Eliminated(_) |
			StarterResult::Withdrawn | StarterResult::NoShow | StarterResult::Retired |
			StarterResult::Disqualified => true
		}
	}

    pub fn to_string(&self) -> String {
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
        }.to_string()
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
        let val = VariableSize::deserialize(de).map_err(|e| serde::de::Error::custom(e))?;

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
                    eprintln!("Eliminated for {} did not match an acceptable value", value);
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