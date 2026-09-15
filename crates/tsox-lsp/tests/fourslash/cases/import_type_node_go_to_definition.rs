use tsox_lsp::fourslash::Session;


#[test]
fn import_type_node_go_to_definition() {
    let content = r#"// @Filename: /ns.ts
/*refFile*/export namespace /*refFoo*/Foo {
    export namespace /*refBar*/Bar {
        export class /*refBaz*/Baz {}
    }
}
// @Filename: /usage.ts
type A = typeof import([|/*1*/"./ns"|]).[|/*2*/Foo|].[|/*3*/Bar|];
type B = import([|/*4*/"./ns"|]).[|/*5*/Foo|].[|/*6*/Bar|].[|/*7*/Baz|];"#;
    let _s = Session::new_for_test("importTypeNodeGoToDefinition", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1", "2", "3", "4", "5", "6", "7")
}
