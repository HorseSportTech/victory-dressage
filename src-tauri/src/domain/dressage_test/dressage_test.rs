use crate::domain::{penalties::Penalties, SurrealId};

use super::Exercise;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DressageTest {
    pub id: SurrealId,
    pub name: String,
    pub movements: Vec<Exercise>,
    pub errors_of_course: Penalties,
    pub technical_penalties: Penalties,
    pub artistic_penalties: Penalties,
    pub test_type: TestSheetType,
    #[serde(default = "default_countdowns")]
    pub countdowns: [u8; 2],
    #[serde(default = "default_test_length")]
    pub length_in_seconds: u16,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestSheetType {
    Normal,
    Freestyle,
    Quality,
}

impl crate::traits::Entity for DressageTest {
    fn key(&self) -> String {
        format!("{}:{}", self.id.tb, self.id.id())
    }
    fn get_id(&self) -> String {
        self.id.id()
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Penalty {
    pub index: u8,
    pub r#type: PenaltyType,
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PenaltyType {
    Elimination,
    Points(f32),
    Percentage(f32),
}

impl Default for DressageTest {
    fn default() -> Self {
        Self {
            id: SurrealId::make("testSheet", "default"),
            name: String::from("Default Test Sheet"),
            movements: vec![],
            errors_of_course: Penalties(vec![]),
            technical_penalties: Penalties(vec![]),
            artistic_penalties: Penalties(vec![]),
            test_type: TestSheetType::Normal,
            countdowns: default_countdowns(),
            length_in_seconds: default_test_length(),
        }
    }
}

fn default_countdowns() -> [u8; 2] {
    [45, 0]
}
fn default_test_length() -> u16 {
    300
}
