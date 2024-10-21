//!
//!
//! ```no_run
//! #[salsa::query_group(DefDatabaseStorage)]
//! pub trait DefDatabase: InternDatabase + ExpandDatabase + Upcast<dyn ExpandDatabase> {
//!     fn crate_supports_no_std(&self, crate_id: CrateId) -> bool;
//!     
//!     #[salsa::input]
//!     fn expand_proc_attr_macros(&self) -> bool;
//!
//!     #[salsa::invoke(DefMap::crate_def_map_query)]
//!     fn crate_def_map(&self, krate: CrateId) -> Arc<DefMap>;
//! }
//!
//! fn crate_supports_no_std(db: &dyn DefDatabase, crate_id: CrateId) -> bool {
//!     let file = db.crate_graph()[crate_id].root_file_id();
//!     // todo...
//! }
//! ```

use salsa::{Durability, Setter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrateId(u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefMap;

impl DefMap {
    pub fn crate_def_map_query(db: &dyn salsa::Database, crate_id: CrateId) -> DefMap {
        todo!()
    }
}

#[salsa::db]
pub trait DefDatabase: salsa::Database {
    // this single getter is what the user "wrote", but also...
    fn expand_proc_attr_macros(&self) -> bool;
    // this setter is generated under the hood
    fn set_expand_proc_attr_macros(&mut self, value: bool);

    // ...as is this one.
    fn set_expand_proc_attr_macros_with_durability(&mut self, value: bool, durability: Durability);

    // this was marked as an input. `CrateId`, however, is a salsa struct!
    // it is just
    fn crate_supports_no_std(&self, crate_id: CrateId) -> bool;

    // #[salsa::invoke(DefMap::crate_def_map_query)]
    fn crate_def_map(&self, krate: CrateId) -> DefMap;
}

#[salsa::db]
#[derive(Default)]
pub struct DefDb {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for DefDb {
    fn salsa_event(&self, _event: &dyn Fn() -> salsa::Event) {}
}

const _: () = {
    // used as a placeholder for structs.
    #[salsa::input]
    struct DefDatabaseData {
        expand_proc_attr_macros: Option<bool>,
    }

    #[salsa::tracked]
    fn create_data(db: &dyn DefDatabase) -> DefDatabaseData {
        DefDatabaseData::new(db, None)
    }

    #[salsa::db]
    impl<DB> DefDatabase for DB
    where
        DB: salsa::Database,
    {
        fn expand_proc_attr_macros(&self) -> bool {
            let data = create_data(self);
            data.expand_proc_attr_macros(self).unwrap()
        }

        fn set_expand_proc_attr_macros(&mut self, value: bool) {
            let data = create_data(self);
            data.set_expand_proc_attr_macros(self).to(Some(value));
        }

        fn set_expand_proc_attr_macros_with_durability(
            &mut self,
            value: bool,
            durability: Durability,
        ) {
            let data = create_data(self);
            data.set_expand_proc_attr_macros(self)
                .with_durability(durability)
                .to(Some(value));
        }

        fn crate_supports_no_std(&self, crate_id: CrateId) -> bool {
            #[salsa::tracked]
            fn __definition__(
                db: &dyn DefDatabase,
                _input: DefDatabaseData,
                crate_id: CrateId,
            ) -> bool {
                crate_supports_no_std(db, crate_id)
            }
            __definition__(self, create_data(self), crate_id)
        }

        fn crate_def_map(&self, crate_id: CrateId) -> DefMap {
            #[salsa::tracked]
            fn __definition__(
                db: &dyn DefDatabase,
                _input: DefDatabaseData,
                crate_id: CrateId,
            ) -> DefMap {
                DefMap::crate_def_map_query(db, crate_id)
            }
            __definition__(self, create_data(self), crate_id)
        }
    }
};

fn crate_supports_no_std(db: &dyn DefDatabase, _crate_id: CrateId) -> bool {
    false
    // let file = db.crate_graph()[crate_id].root_file_id();
    // todo...
}

fn main() {}
