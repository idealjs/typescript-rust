use tsox_lsp::fourslash::Session;


#[test]
fn hover_nil_base_symbol_intersection() {
    let content = r#"
// @strict: true
// @filename: main.ts

class Base {}

declare const BaseFactory: new() => Base & { c: string };

class Derived extends BaseFactory {
  static /*1*/idField = "id" as const;
}
"#;
    let _s = Session::new_for_test("hoverNilBaseSymbolIntersection", content);
    // TODO: // We only care that hover/quickinfo does not crash (panic) when baseType.Symbol() is nil.
    // TODO: // Pre-fix (#2763), hovering on the static property could panic in getJSDocOrTag.
    // TODO: f.VerifyBaselineHover(t)
}
