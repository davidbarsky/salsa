use super::{memo::Memo, Configuration, IngredientImpl};
use crate::accumulator::accumulated_map::InputAccumulatedValues;
use crate::{
    runtime::StampedValue, zalsa::ZalsaDatabase, zalsa_local::QueryRevisions, AsDynDatabase as _,
    Id,
};

impl<C> IngredientImpl<C>
where
    C: Configuration,
{
    pub fn fetch<'db>(&'db self, db: &'db C::DbView, id: Id) -> &'db C::Output<'db> {
        let (zalsa, zalsa_local) = db.zalsas();
        zalsa_local.unwind_if_revision_cancelled(db.as_dyn_database());

        let memo = self.refresh_memo(db, id);
        let StampedValue {
            value,
            durability,
            changed_at,
        } = memo.revisions.stamped_value(memo.value.as_ref().unwrap());

        if let Some(evicted) = self.lru.record_use(id) {
            self.evict_value_from_memo_for(zalsa, evicted);
        }

        zalsa_local.report_tracked_read(
            self.database_key_index(id).into(),
            durability,
            changed_at,
            InputAccumulatedValues::from_map(&memo.revisions.accumulated),
            &memo.revisions.cycle_heads,
        );

        value
    }

    #[inline]
    pub(super) fn refresh_memo<'db>(
        &'db self,
        db: &'db C::DbView,
        id: Id,
    ) -> &'db Memo<C::Output<'db>> {
        loop {
            if let Some(memo) = self.fetch_hot(db, id).or_else(|| self.fetch_cold(db, id)) {
                return memo;
            }
        }
    }

    #[inline]
    fn fetch_hot<'db>(&'db self, db: &'db C::DbView, id: Id) -> Option<&'db Memo<C::Output<'db>>> {
        let zalsa = db.zalsa();
        let memo_guard = self.get_memo_from_table_for(zalsa, id);
        if let Some(memo) = &memo_guard {
            if memo.value.is_some()
                && self.shallow_verify_memo(db, zalsa, self.database_key_index(id), memo)
            {
                // Unsafety invariant: memo is present in memo_map
                unsafe {
                    return Some(self.extend_memo_lifetime(memo));
                }
            }
        }
        None
    }

    fn fetch_cold<'db>(&'db self, db: &'db C::DbView, id: Id) -> Option<&'db Memo<C::Output<'db>>> {
        let (zalsa, zalsa_local) = db.zalsas();
        let database_key_index = self.database_key_index(id);

        // Try to claim this query: if someone else has claimed it already, go back and start again.
        let _claim_guard = zalsa.sync_table_for(id).claim(
            db.as_dyn_database(),
            zalsa_local,
            database_key_index,
            self.memo_ingredient_index,
        )?;

        // Now that we've claimed the item, check again to see if there's a "hot" value.
        let opt_old_memo = self.get_memo_from_table_for(zalsa, id);
        if let Some(old_memo) = &opt_old_memo {
            if old_memo.value.is_some() {
                let active_query = zalsa_local.push_query(database_key_index);
                if self.deep_verify_memo(db, old_memo, &active_query) {
                    // Unsafety invariant: memo is present in memo_map.
                    unsafe {
                        return Some(self.extend_memo_lifetime(old_memo));
                    }
                }
            }
        }
        let revision_now = zalsa.current_revision();

        let mut opt_last_provisional = if let Some(initial_value) = self.initial_value(db) {
            Some(self.insert_memo(
                zalsa,
                id,
                Memo::new(
                    Some(initial_value),
                    revision_now,
                    QueryRevisions::fixpoint_initial(database_key_index),
                ),
            ))
        } else {
            None
        };
        let mut iteration_count = 0;

        loop {
            let active_query = zalsa_local.push_query(database_key_index);
            let mut result = self.execute(db, active_query, opt_old_memo.clone());

            if result.in_cycle(database_key_index) {
                if let Some(last_provisional) = opt_last_provisional {
                    match (&result.value, &last_provisional.value) {
                        (Some(result_value), Some(provisional_value))
                            if !C::values_equal(result_value, provisional_value) =>
                        {
                            match C::recover_from_cycle(db, result_value, iteration_count) {
                                crate::CycleRecoveryAction::Iterate => {
                                    iteration_count += 1;
                                    opt_last_provisional = Some(result);
                                    continue;
                                }
                                crate::CycleRecoveryAction::Fallback(value) => {
                                    result = self.insert_memo(
                                        zalsa,
                                        id,
                                        Memo::new(
                                            Some(value),
                                            revision_now,
                                            result.revisions.clone(),
                                        ),
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
                // This is no longer a provisional result, it's our real result, so remove ourselves
                // from the cycle heads.
            }
            return Some(result);
        }
    }
}
