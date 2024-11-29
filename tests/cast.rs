use expect_test::expect;
use test_log::test;

#[salsa::interned]
struct FancyId<'db> {
    wrapper: Wrapper<'db>,
}

#[derive(salsa::Update, Clone, PartialEq, Eq, Hash, Debug)]
enum Wrapper<'db> {
    A(A<'db>),
    B(B<'db>),
}

#[salsa::interned]
struct A<'db> {
    data: String,
}

#[salsa::interned]
struct B<'db> {
    data: String,
}

impl<'db> FancyId<'db> {
    // Convenient downcast methods for each of the options
    fn a(self, db: &'db dyn salsa::Database) -> Option<A<'db>> {
        self.0.downcast(db)
    }

    fn b(self, db: &'db dyn salsa::Database) -> Option<B<'db>> {
        self.0.downcast(db)
    }
}

#[salsa::tracked]
fn intern_a(db: &dyn salsa::Database) -> FancyId<'_> {
    FancyId::new(db, Wrapper::A(A::new(db, String::from("Hello, world!"))))
}

#[salsa::tracked]
fn intern_b(db: &dyn salsa::Database) -> FancyId<'_> {
    FancyId::new(db, Wrapper::B(B::new(db, String::from("Hello, world!"))))
}

#[test]
fn test_casting() {
    let db = salsa::DatabaseImpl::new();

    let a = intern_a(&db).a(&db).unwrap();
    let b = intern_b(&db).b(&db).unwrap();
}
