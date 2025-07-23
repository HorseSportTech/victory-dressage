use crate::commands::warnings::manager::Warnings;

use super::dressage_test::DressageTest;
use super::penalties::PenaltyType;
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
    #[serde(default)]
    pub test: Option<DressageTest>,
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
impl Scoresheet {
    pub fn deductions(&self, test: &DressageTest) -> f64 {
        let total_marks = test.total_marks();
        let mut points_deduction = 0f64;
        let mut percent_deduction = 0f64;
        for i in 0..self.errors {
            let pen =
                &test.errors_of_course[usize::min(i as usize, test.errors_of_course.len() - 1)];
            match pen.ty {
                PenaltyType::Percentage(num) => percent_deduction += num as f64,
                PenaltyType::Points(num) => points_deduction += num as f64,
                PenaltyType::Elimination => (),
            }
        }
        for i in 0..self.tech_penalties {
            let pen = &test.technical_penalties
                [usize::min(i as usize, test.technical_penalties.len() - 1)];
            match pen.ty {
                PenaltyType::Percentage(num) => percent_deduction += num as f64,
                PenaltyType::Points(num) => points_deduction += num as f64,
                PenaltyType::Elimination => (),
            }
        }
        for i in 0..self.art_penalties {
            let pen =
                &test.artistic_penalties[usize::min(i as usize, test.artistic_penalties.len() - 1)];
            match pen.ty {
                PenaltyType::Percentage(num) => percent_deduction += num as f64,
                PenaltyType::Points(num) => points_deduction += num as f64,
                PenaltyType::Elimination => (),
            }
        }
        (points_deduction * 100.0 / total_marks) + percent_deduction
    }

    pub fn trend(&self, testsheet: &DressageTest) -> f64 {
        let mut total = 0.0;
        let mut max_total = 0.0;
        for movement in testsheet.movements.iter() {
            let Some(exercise) = self.scores.iter().find(|x| x.nr == movement.number as u16) else {
                continue;
            };
            total += exercise.mk.unwrap_or(0.0) * movement.coefficient as f64;
            max_total += exercise.mk.map_or(0.0, |_| movement.max) * movement.coefficient;
        }
        return total / max_total as f64 * 100.0 - self.deductions(testsheet);
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
