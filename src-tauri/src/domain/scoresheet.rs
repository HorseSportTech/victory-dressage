use crate::commands::warnings::manager::Warnings;
use decimal::{dec, Decimal};

use super::dressage_test::DressageTest;
use super::penalties::PenaltyType;
use super::SurrealId;
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Scoresheet {
    pub id: SurrealId,
    #[serde(
        deserialize_with = "decimal::parsing::deserialize_opt_from_f64",
        serialize_with = "decimal::parsing::serialize_opt_as_f64"
    )]
    pub score: Option<Decimal>,
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
    #[serde(default)]
    pub test: Option<DressageTest>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ScoredMark {
    #[serde(rename = "nr")]
    pub number: u16,
    #[serde(
        rename = "mk",
        deserialize_with = "decimal::parsing::deserialize_opt_from_f64",
        serialize_with = "decimal::parsing::serialize_opt_as_f64"
    )]
    pub mark: Option<Decimal>,
    #[serde(rename = "rk")]
    pub rank: Option<String>,
    #[serde(rename = "at", default, skip_serializing_if = "Vec::is_empty")]
    pub attempts: Vec<Decimal>,
}

impl ScoredMark {
    pub fn new(index: u16) -> Self {
        ScoredMark {
            number: index,
            mark: None,
            rank: None,
            attempts: vec![],
        }
    }
}
impl Scoresheet {
    pub fn deductions(&self, test: &DressageTest) -> Decimal {
        let total_marks = test.total_marks();
        let mut points_deduction = dec!(0.0);
        let mut percent_deduction = dec!(0.000);
        for i in 0..self.errors {
            let pen =
                &test.errors_of_course[usize::min(i as usize, test.errors_of_course.len() - 1)];
            match pen.ty {
                PenaltyType::Percentage(num) => percent_deduction += num,
                PenaltyType::Points(num) => points_deduction += num,
                PenaltyType::Elimination => (),
            }
        }
        for i in 0..self.tech_penalties {
            let pen = &test.technical_penalties
                [usize::min(i as usize, test.technical_penalties.len() - 1)];
            match pen.ty {
                PenaltyType::Percentage(num) => percent_deduction += num,
                PenaltyType::Points(num) => points_deduction += num,
                PenaltyType::Elimination => (),
            }
        }
        for i in 0..self.art_penalties {
            let pen =
                &test.artistic_penalties[usize::min(i as usize, test.artistic_penalties.len() - 1)];
            match pen.ty {
                PenaltyType::Percentage(num) => percent_deduction += num,
                PenaltyType::Points(num) => points_deduction += num,
                PenaltyType::Elimination => (),
            }
        }
        (points_deduction * dec!(100.000) / total_marks) + percent_deduction
    }

    pub fn trend(&self, testsheet: &DressageTest) -> Decimal {
        let mut total = dec!(0.0);
        let mut max_total = dec!(0.0);
        for movement in testsheet.movements.iter() {
            let Some(exercise) = self
                .scores
                .iter()
                .find(|x| x.number == movement.number as u16)
            else {
                continue;
            };
            total += exercise.mark.unwrap_or_default() * movement.coefficient;
            max_total += exercise.mark.map_or(dec!(0.0), |_| movement.max) * movement.coefficient;
        }
        total * dec!(100.000) / max_total - self.deductions(testsheet)
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
