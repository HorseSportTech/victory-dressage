use crate::domain::starter::Starter;

impl Starter {
    // --- Impose Handlers --- //
    // The following handlers are to reduce the duplication
    // of logic across both the state and memory.
    // They should provide an exact update of the state
    // in both cases
    pub(in crate::sockets) fn impose_lock(&mut self, lock: &super::message_types::server::Lock) {
        if let Some(scoresheet) = self.scoresheets.first_mut() {
            scoresheet.rank = lock.rank;
            scoresheet.locked = lock.locked;

            // NOTE: the errors and penalties are none IF THEY
            // ARE NOT BEING UPDATED, not merely if there
            // are no penalties, so use that to determine
            // whether to overwrite the current value
            // or not
            if let Some(e) = lock.errors_of_course {
                scoresheet.errors = e;
            }
            if let Some(tp) = lock.technical_penalties {
                scoresheet.tech_penalties = tp;
            }
            if let Some(ap) = lock.artistic_penalties {
                scoresheet.art_penalties = ap;
            }

            // TODO: Add check over all marks
        }
    }
    pub(in crate::sockets) fn impose_trend(&mut self, trend: &super::message_types::server::Trend) {
        if let Some(scoresheet) = self.scoresheets.first_mut() {
            if scoresheet.id.ulid() == trend.sheet_id {
                scoresheet.score = Some(trend.score);
                scoresheet.rank = Some(trend.rank);
            }
        }
    }
}
