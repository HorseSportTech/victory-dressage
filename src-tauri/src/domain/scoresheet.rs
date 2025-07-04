use crate::commands::warnings::manager::Warnings;

use super::SurrealId;
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Scoresheet {
    pub id: SurrealId,
    pub score: Option<f64>,
    pub rank: Option<u16>,
    pub errors: u8,
    pub tech_penalties: u8,
    pub art_penalties: u8,
    pub scores: Vec<ScoredMark>,
    pub summary: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub warning_manager: Warnings,
    #[serde(default)]
    pub locked: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScoredMark {
    pub nr: u16,
    pub mk: Option<f64>,
    pub rk: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub at: Vec<f64>,
}

impl ScoredMark {
    pub fn new(index: u16) -> Self {
        ScoredMark {
            nr: index,
            mk: None,
            rk: None,
            at: vec![],
        }
    }
}

impl crate::traits::Storable for Scoresheet {}
impl crate::traits::Entity for Scoresheet {
    fn key(&self) -> String {
        format!("{}:{}", self.id.tb, self.id.id())
    }
    fn get_id(&self) -> String {
        self.id.id()
    }
}
