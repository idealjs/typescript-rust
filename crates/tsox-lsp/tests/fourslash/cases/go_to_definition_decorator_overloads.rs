use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_decorator_overloads() {
    let content = r#"// @Target: ES6
// @experimentaldecorators: true
async function f() {}

function /*defDecString*/dec(target: any, propertyKey: string): void;
function /*defDecSymbol*/dec(target: any, propertyKey: symbol): void;
function dec(target: any, propertyKey: string | symbol) {}

declare const s: symbol;
class C {
    @[|/*useDecString*/dec|] f() {}
    @[|/*useDecSymbol*/dec|] [s]() {}
}"#;
    let _s = Session::new_for_test("goToDefinitionDecoratorOverloads", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "useDecString", "useDecSymbol")
}
