use crate::{plumbing::JarAux, Database, Id, IngredientIndex};

pub trait SalsaStructInDb<'db> {
    fn new<DB>(db: &DB, id: Id) -> Self
    where
        DB: ?Sized + Database;

    fn ingredient_index<DB>(db: &DB) -> IngredientIndex
    where
        DB: ?Sized + Database;

    fn lookup_ingredient_index(aux: &dyn JarAux) -> Option<IngredientIndex>;
}
