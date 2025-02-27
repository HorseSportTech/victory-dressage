use chrono::Utc;
use ulid::Ulid;

use crate::{
    domain::{
        penalties::BroadcastPenaltyVariety, position::Position, scoresheet::ScoredMark, SurrealId,
    },
    state::{application_page::ApplicationPage, battery::VirtualDeviceBattery},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AppSocketMessage {
    // Show(ShowDTO),
    Competition(CompetitionMessage),
    ApplicationState {
        id: ulid::Ulid,
        judge_id: SurrealId,
        show_id: Option<SurrealId>,
        competition_id: Option<SurrealId>,
        location: ApplicationPage,
        // #[serde(rename = "state")]
        state: VirtualDeviceBattery,
        #[serde(skip_serializing_if = "Option::is_none")]
        competitor_name: Option<String>,
    },
    // Auth(String),
    // Ack(ulid::Ulid),
    // Error(String),
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompetitionMessage {
    Unsubscribe,
    Subscribe {
        id: Ulid,
    },
    Mark {
        #[serde(rename = "sid")]
        sheet_id: Ulid,
        #[serde(rename = "n")]
        number: u32,
        #[serde(rename = "m")]
        mark: Option<f64>,
        #[serde(rename = "r")]
        remark: Option<String>,
        //#[serde(rename = "d", skip_serializing_if = "MarkModifier::is_default")]
        //pub modifier: MarkModifier,
    },
    Summary {
        #[serde(rename = "sid")]
        sheet_id: Ulid,
        #[serde(rename = "s")]
        summary: Option<String>,
        #[serde(rename = "n")]
        notes: Option<String>,
    },
    Trend {
        #[serde(rename = "sid")]
        sheet_id: Ulid,
        #[serde(rename = "rk")]
        rank: u16,
        #[serde(rename = "sc")]
        score: f64,
    },
    Reset {
        #[serde(rename = "sid")]
        sheet_id: Ulid,
        #[serde(rename = "ts")]
        timestamp: chrono::DateTime<Utc>,
    },
    Penalty {
        #[serde(rename = "sid")]
        sheet_id: Ulid,
        #[serde(rename = "v")]
        variety: BroadcastPenaltyVariety,
        #[serde(rename = "q")]
        quantity: u8,
    },
    //Notification{
    //starter_id: Ulid,
    //position: Position,
    //warn: NotificationCategory,
    //},
    //Signal{
    //sheet_id: Ulid,
    //varient: JudgesSignallingVarient,
    //}
    Status {
        sid: Ulid,
    },
    Lock {
        sheet_id: Ulid,
        locked: bool,
        scores: Option<Vec<ScoredMark>>,
    },
}

