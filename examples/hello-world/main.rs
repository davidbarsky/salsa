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

use salsa::{Setter, Storage};

#[salsa::db]
pub trait HelloWorldDatabase: salsa::Database {
    fn input_string(&self, key: ()) -> String;

    fn set_input_string(&mut self, key: (), value__: String);

    fn set_input_string_with_durability(
        &mut self,
        key: (),
        value__: String,
        durability__: salsa::Durability,
    );

    fn length(&self, key: ()) -> usize;
}

#[salsa::input]
struct HelloWorldDatabaseData {
    input_string: Option<String>,
}

#[salsa::db]
#[derive(Default)]
pub struct HelloWorldDb {
    storage: Storage<Self>,
}

#[salsa::db]
impl salsa::Database for HelloWorldDb {
    fn salsa_event(&self, _event: &dyn Fn() -> salsa::Event) {}
}

const _: () = {
    #[salsa::tracked]
    fn create_data(db: &dyn salsa::Database) -> HelloWorldDatabaseData {
        HelloWorldDatabaseData::new(db, None)
    }

    #[salsa::db]
    impl<DB> HelloWorldDatabase for DB
    where
        DB: salsa::Database,
    {
        fn input_string(&self, _key: ()) -> String {
            let data = create_data(self);
            data.input_string(self).unwrap()
        }

        fn set_input_string(&mut self, _key: (), value__: String) {
            let data = create_data(self);
            data.set_input_string(self).to(Some(value__));
        }

        fn set_input_string_with_durability(
            &mut self,
            _key: (),
            value__: String,
            durability__: salsa::Durability,
        ) {
            let data = create_data(self);
            data.set_input_string(self)
                .with_durability(durability__)
                .to(Some(value__));
        }

        fn length(&self, key: ()) -> usize {
            #[salsa::tracked]
            fn __length_definition__(
                db: &dyn HelloWorldDatabase,
                _input: HelloWorldDatabaseData,
                key: (),
            ) -> usize {
                length(db, key)
            }

            __length_definition__(self, create_data(self), key)
        }
    }
};

fn length(db: &dyn HelloWorldDatabase, key: ()) -> usize {
    let string = db.input_string(key);
    string.len()
}

fn main() {
    let mut db = HelloWorldDb::default();
    db.set_input_string((), format!("Hello, world"));

    dbg!(db.length(()));
}
