//! Test a specific cycle scenario:
//!
//! ```text
//! Thread T1          Thread T2
//! ---------          ---------
//!    |                  |
//!    v                  |
//! query_a()             |
//!  ^  |                 v
//!  |  +------------> query_b()
//!  |                    |
//!  +--------------------+
//! ```

use salsa::CycleRecoveryAction;

use crate::setup::{Knobs, KnobsDatabase};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct CycleValue(u32);

const MIN: CycleValue = CycleValue(0);
const MAX: CycleValue = CycleValue(3);

// Signal 1: T1 has entered `query_a`
// Signal 2: T2 has entered `query_b`

#[salsa::tracked(cycle_fn=cycle_fn, cycle_initial=initial)]
fn query_a(db: &dyn KnobsDatabase) -> CycleValue {
    db.signal(1);

    // Wait for Thread T2 to enter `query_b` before we continue.
    db.wait_for(2);

    query_b(db)
}

#[salsa::tracked(cycle_fn=cycle_fn, cycle_initial=initial)]
fn query_b(db: &dyn KnobsDatabase) -> CycleValue {
    // Wait for Thread T1 to enter `query_a` before we continue.
    db.wait_for(1);

    db.signal(2);

    let a_value = query_a(db);
    CycleValue(a_value.0 + 1).min(MAX)
}

fn cycle_fn(
    _db: &dyn KnobsDatabase,
    _value: &CycleValue,
    _count: u32,
) -> CycleRecoveryAction<CycleValue> {
    CycleRecoveryAction::Iterate
}

fn initial(_db: &dyn KnobsDatabase) -> CycleValue {
    MIN
}

#[cfg(feature = "loom")]
#[test]
fn the_test() {
    loom::model(|| {
        let db_t1 = Knobs::default();
        let db_t2 = db_t1.clone();

        // Thread 1:
        loom::thread::spawn(move || {
            let r = query_a(&db_t1);
            assert_eq!(r, MAX);
        });

        // Thread 2:
        loom::thread::spawn(move || {
            let r = query_b(&db_t2);
            assert_eq!(r, MAX);
        });
    });
}
