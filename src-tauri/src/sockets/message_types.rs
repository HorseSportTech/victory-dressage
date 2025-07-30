pub mod common {
    use ulid::Ulid;

    use crate::commands::alert_manager::AlertType;
    use crate::domain::scoresheet::ScoredMark;
    use crate::domain::starter::StarterResult;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub(in crate::sockets) struct Signal {
        #[serde(rename = "sid")]
        pub(in crate::sockets) sheet_id: ulid::Ulid,
        pub(in crate::sockets) signal: AlertType,
    }
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub(in crate::sockets) struct Status {
        #[serde(rename = "sid")]
        pub(in crate::sockets) sheet_id: Ulid,
        pub(in crate::sockets) status: StarterResult,
    }
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub(crate) struct Lock {
        #[serde(rename = "sid")]
        pub sheet_id: Ulid,
        pub locked: bool,
        pub scores: Option<Vec<ScoredMark>>,
    }
}
pub mod application {
    use decimal::Decimal;
    use ulid::Ulid;

    use crate::domain::penalties::BroadcastPenaltyVariety;
    use crate::domain::SurrealId;
    use crate::state::application_page::ApplicationPage;
    use crate::state::battery::VirtualDeviceBattery;

    use super::common::{Lock, Signal, Status};

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum Payload {
        // Show(ShowDTO),
        Competition(CompetitionMessage),
        #[serde(rename_all = "camelCase")]
        ApplicationState {
            id: ulid::Ulid,
            judge_id: SurrealId,
            show_id: Option<SurrealId>,
            competition_id: Option<SurrealId>,
            location: ApplicationPage,
            state: VirtualDeviceBattery,
        },
        NoOp,
        Ack(ulid::Ulid),
    }
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum CompetitionMessage {
        Unsubscribe,
        Mark(Mark),
        Summary(Summary),
        Penalty(Penalty),
        Signal(Signal),
        Status(Status),
        Lock(Lock),
    }
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(in crate::sockets) struct Mark {
        #[serde(rename = "sid")]
        pub sheet_id: Ulid,
        #[serde(rename = "n")]
        pub number: u32,
        #[serde(rename = "m")]
        pub mark: Option<Decimal>,
        #[serde(rename = "r")]
        pub remark: Option<String>,
        //#[serde(rename = "d", skip_serializing_if = "MarkModifier::is_default")]
        //pub modifier: MarkModifier,
    }
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(in crate::sockets) struct Summary {
        #[serde(rename = "sid")]
        sheet_id: Ulid,
        #[serde(rename = "s")]
        summary: Option<String>,
        #[serde(rename = "n")]
        notes: Option<String>,
    }
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(in crate::sockets) struct Penalty {
        #[serde(rename = "sid")]
        sheet_id: Ulid,
        #[serde(rename = "v")]
        variety: BroadcastPenaltyVariety,
        #[serde(rename = "q")]
        quantity: u8,
    }
}
pub mod server {
    use decimal::Decimal;
    use ulid::Ulid;

    use crate::domain::starter::Starter;
    use crate::domain::SurrealId;
    use crate::state::application_page::ApplicationPage;
    use crate::state::battery::VirtualDeviceBattery;

    use super::application::Penalty;
    use super::common::{Lock, Signal, Status};

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub enum Payload {
        // Show(ShowDTO),
        Competition(CompetitionMessage),
        ApplicationState {
            id: ulid::Ulid,
            judge_id: SurrealId,
            show_id: Option<SurrealId>,
            competition_id: Option<SurrealId>,
            location: ApplicationPage,
            state: VirtualDeviceBattery,
            #[serde(default)]
            competitor_name: Option<String>,
        },
        Ack(ulid::Ulid),
    }
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum CompetitionMessage {
        Unsubscribe,
        Trend(Trend),
        Reset(Reset),
        // Penalty(Penalty),
        Signal(Signal),
        /// If a new starter is recieved, then first check for an
        /// existing starter with the same ID. If no exsisting
        /// starter is found, then add the starter as a new
        /// starter.
        /// This is a list of starters so that multiple
        /// can be changed at the same time (such as
        /// when a time is updated).
        /// TODO: Consider if other functions
        /// should be rolled into this
        /// function as well.
        AlterStarter(AlterStarter),
        Status(Status),
        Lock(Lock),
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(in crate::sockets) struct Trend {
        #[serde(rename = "sid")]
        pub(in crate::sockets) sheet_id: Ulid,
        #[serde(rename = "rk")]
        pub(in crate::sockets) rank: u16,
        #[serde(rename = "sc")]
        pub(in crate::sockets) score: Decimal,
    }
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub(in crate::sockets) struct AlterStarter {
        starter: Starter,
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(in crate::sockets) struct Reset {
        #[serde(rename = "sid")]
        pub(in crate::sockets) sheet_id: Ulid,
        #[serde(rename = "ts")]
        pub(in crate::sockets) timestamp: chrono::DateTime<chrono::Utc>,
    }
}
