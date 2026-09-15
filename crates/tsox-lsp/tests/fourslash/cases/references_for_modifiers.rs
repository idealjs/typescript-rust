use tsox_lsp::fourslash::Session;


#[test]
fn references_for_modifiers() {
    let content = r#"// @lib: es5
[|/*declareModifier*/declare /*abstractModifier*/abstract class C1 {
    [|/*staticModifier*/static a;|]
    [|/*readonlyModifier*/readonly b;|]
    [|/*publicModifier*/public c;|]
    [|/*protectedModifier*/protected d;|]
    [|/*privateModifier*/private e;|]
}|]
[|/*constModifier*/const enum E {
}|]
[|/*asyncModifier*/async function fn() {}|]
[|/*exportModifier*/export /*defaultModifier*/default class C2 {}|]"#;
    let _s = Session::new_for_test("referencesForModifiers", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "declareModifier", "abstractModifier", "staticModifier", "reado
}
