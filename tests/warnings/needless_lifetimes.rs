use salsa::Update;

#[salsa::db]
pub trait Db: salsa::Database {}

#[derive(Debug, PartialEq, Eq, Hash, Update)]
pub struct Item {}

#[salsa::tracked]
pub struct SourceTree<'db> {}

#[salsa::tracked]
impl<'db> SourceTree<'db> {
    #[salsa::tracked(return_ref)]
    pub fn all_items(self, _db: &'db dyn Db) -> Vec<Item> {
        todo!()
    }
}

#[salsa::tracked(return_ref)]
fn use_tree<'db>(_db: &'db dyn Db, _tree: SourceTree<'db>) {}

#[allow(unused)]
fn use_it(db: &dyn Db, tree: SourceTree) {
    tree.all_items(db);
    use_tree(db, tree);
}
