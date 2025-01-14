use std::sync::Arc;

use crate::{
    cycle::MAX_ITERATIONS, zalsa::ZalsaDatabase, Database, DatabaseKeyIndex, Event, EventKind,
};

use super::{memo::Memo, Configuration, IngredientImpl};

impl<C> IngredientImpl<C>
where
    C: Configuration,
{
    /// Executes the query function for the given `active_query`. Creates and stores
    /// a new memo with the result, backdated if possible. Once this completes,
    /// the query will have been popped off the active query stack.
    ///
    /// # Parameters
    ///
    /// * `db`, the database.
    /// * `active_query`, the active stack frame for the query to execute.
    /// * `opt_old_memo`, the older memo, if any existed. Used for backdated.
    pub(super) fn execute<'db>(
        &'db self,
        db: &'db C::DbView,
        database_key_index: DatabaseKeyIndex,
        opt_old_memo: Option<Arc<Memo<C::Output<'_>>>>,
    ) -> &'db Memo<C::Output<'db>> {
        let (zalsa, zalsa_local) = db.zalsas();
        let revision_now = zalsa.current_revision();
        let id = database_key_index.key_index;

        tracing::info!("{:?}: executing query", database_key_index);

        db.salsa_event(&|| {
            Event::new(EventKind::WillExecute {
                database_key: database_key_index,
            })
        });

        let mut iteration_count: u32 = 0;

        // Our provisional value from the previous iteration, when doing fixpoint iteration.
        // Initially it's set to None, because the initial provisional value is created lazily,
        // only when a cycle is actually encountered.
        let mut opt_last_provisional: Option<&Memo<<C as Configuration>::Output<'db>>> = None;

        loop {
            let active_query = zalsa_local.push_query(database_key_index);

            // If we already executed this query once, then use the tracked-struct ids from the
            // previous execution as the starting point for the new one.
            if let Some(old_memo) = &opt_old_memo {
                active_query.seed_tracked_struct_ids(&old_memo.revisions.tracked_struct_ids);
            }

            // Query was not previously executed, or value is potentially
            // stale, or value is absent. Let's execute!
            let mut new_value = C::execute(db, C::id_to_input(db, id));
            let mut revisions = active_query.pop();

            // Did the new result we got depend on our own provisional value, in a cycle?
            if revisions.cycle_heads.contains(&database_key_index) {
                let opt_owned_last_provisional;
                let last_provisional_value = if let Some(last_provisional) = opt_last_provisional {
                    // We have a last provisional value from our previous time around the loop.
                    last_provisional
                        .value
                        .as_ref()
                        .expect("provisional value should not be evicted by LRU")
                } else {
                    // This is our first time around the loop; a provisional value must have been
                    // inserted into the memo table when the cycle was hit, so let's pull our
                    // initial provisional value from there.
                    opt_owned_last_provisional = self.get_memo_from_table_for(zalsa, id);
                    opt_owned_last_provisional
                        .as_deref()
                        .expect(
                            "{database_key_index:#?} is a cycle head, \
                            but no provisional memo found",
                        )
                        .value
                        .as_ref()
                        .expect("provisional value should not be evicted by LRU")
                };
                tracing::debug!(
                    "{database_key_index:?}: execute: \
                    I am a cycle head, comparing last provisional value with new value"
                );
                dbg!(&new_value);
                dbg!(last_provisional_value);
                // If the new result is equal to the last provisional result, the cycle has
                // converged and we are done.
                if !C::values_equal(&new_value, last_provisional_value) {
                    // We are in a cycle that hasn't converged; ask the user's
                    // cycle-recovery function what to do:
                    // TODO do we need explicit prevention of people calling queries inside
                    // cycle-recovery functions (some no-queries-allowed state on Runtime?)
                    // or is this just an "if it hurts, don't do it" scenario?
                    match C::recover_from_cycle(
                        db,
                        &new_value,
                        iteration_count,
                        C::id_to_input(db, id),
                    ) {
                        crate::CycleRecoveryAction::Iterate => {
                            tracing::debug!("{database_key_index:?}: execute: iterate again");
                        }
                        crate::CycleRecoveryAction::Fallback(fallback_value) => {
                            tracing::debug!(
                                "{database_key_index:?}: execute: user cycle_fn says to fall back"
                            );
                            new_value = fallback_value;
                            // We have to insert the fallback value for this query and then iterate
                            // one more time to fill in correct values for everything else in the
                            // cycle based on it; then we'll re-insert it as final value.
                        }
                    }
                    iteration_count = iteration_count.checked_add(1).expect(
                        "fixpoint iteration of {database_key_index:#?} should \
                                converge before u32::MAX iterations",
                    );
                    if iteration_count > MAX_ITERATIONS {
                        panic!("{database_key_index:?}: execute: too many cycle iterations");
                    }
                    opt_last_provisional = Some(self.insert_memo(
                        zalsa,
                        id,
                        Memo::new(Some(new_value), revision_now, revisions),
                    ));
                    continue;
                }
                tracing::debug!(
                    "{database_key_index:?}: execute: fixpoint iteration has a final value"
                );
                revisions.cycle_heads.remove(&database_key_index);
                dbg!(&revisions.cycle_heads);
            }

            tracing::debug!("{database_key_index:?}: execute: result.revisions = {revisions:#?}");
            dbg!(&new_value);

            // If the new value is equal to the old one, then it didn't
            // really change, even if some of its inputs have. So we can
            // "backdate" its `changed_at` revision to be the same as the
            // old value.
            if let Some(old_memo) = &opt_old_memo {
                self.backdate_if_appropriate(old_memo, &mut revisions, &new_value);
                self.diff_outputs(db, database_key_index, old_memo, &mut revisions);
            }

            return self.insert_memo(
                zalsa,
                id,
                Memo::new(Some(new_value), revision_now, revisions),
            );
        }
    }
}
