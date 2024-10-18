//! A port of og-salsa to new Salsa.
//!
//! The desugaring is taking from [this document](https://hackmd.io/wIy3aYNaQiuQBI4Oqw7E9g).
//!
//! ```no_run
//! #[salsa::query_group(HelloWorldStorage)]
//! trait HelloWorldDatabase: salsa::Database {
//!     #[salsa::input]
//!     fn input_string(&self, key: ()) -> Arc<String>;
//!
//!     fn length(&self, key: ()) -> usize;
//! }
//!
//! fn length(db: &dyn HelloWorldDatabase, (): ()) -> usize {
//!     // Read the input string:
//!     let input_string = db.input_string(());
//!
//!     // Return its length:
//!     input_string.len()
//! }
//! ```

use salsa::Setter;
use std::sync::Arc;

#[salsa::db]
pub trait HelloWorldDatabase {
    fn input_string(&self, key: ()) -> Arc<String>;

    fn set_input_string(&mut self, key: (), value__: Arc<String>);

    fn set_input_string_with_durability(
        &mut self,
        key: (),
        value__: Arc<String>,
        durability__: salsa::Durability,
    );

    fn length(&self, key: ()) -> usize;
}

const _: () = {
    #[salsa::input]
    struct HelloWorldDatabaseData {
        input_string: Option<Arc<String>>,
    }

    #[salsa::tracked]
    fn create_data(db: &dyn salsa::Database) -> HelloWorldDatabaseData {
        HelloWorldDatabaseData::new(db, None)
    }

    #[salsa::db]
    impl<DB> HelloWorldDatabase for DB
    where
        DB: salsa::Database,
    {
        fn length(&self, key: ()) -> usize {
            #[salsa::tracked]
            fn __definition__(db: &dyn HelloWorldDatabase, key: ()) -> usize {
                length(db, key)
            }
            __definition__(self, key)
        }

        fn input_string(&self, key: ()) -> Arc<String> {
            // todo: what to do about the key?
            let data = create_data(self);
            data.input_string(self).unwrap()
        }

        fn set_input_string(&mut self, key: (), value__: Arc<String>) {
            let data = create_data(self);
            data.set_input_string(self).to(Some(value__));
        }

        fn set_input_string_with_durability(
            &mut self,
            key: (),
            value__: Arc<String>,
            durability__: salsa::Durability,
        ) {
            // todo: what to do with the key?
            let data = create_data(self);
            data.set_input_string(self)
                .with_durability(durability__)
                .to(Some(value__));
        }
    }
};

fn length(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    db.length(())
}

fn main() {
    let db = salsa::DatabaseImpl::new();
    db.set_input_string((), Arc::new(format!("Hello, world")));
}
