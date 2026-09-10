use tsox_lsp::fourslash::{self, Session};


#[test]
fn derived_type_indexer_with_generic_constraints() {
    let content = r#"// @strict: false
class CollectionItem {
    x: number;
}
class Entity extends CollectionItem {
    y: number;
}
class BaseCollection<TItem extends CollectionItem>  {
    _itemsByKey: { [key: string]: TItem; };
}
class DbSet<TEntity extends Entity> extends BaseCollection<TEntity> { // error
    _itemsByKey: { [key: string]: TEntity; } = {};
}
var a: BaseCollection<CollectionItem>;
var /**/r = a._itemsByKey['x']; // should just say CollectionItem not TItem extends CollectionItem
var result = r.x;
a = new DbSet<Entity>();
var r2 = a._itemsByKey['x'];
var result2 = r2.x;"#;
    let mut s = Session::new_for_test("derivedTypeIndexerWithGenericConstraints", content);
    fourslash::verify_quick_info_at(&mut s, "", "var r: CollectionItem", "");
    fourslash::verify_no_errors(&mut s, );
}
