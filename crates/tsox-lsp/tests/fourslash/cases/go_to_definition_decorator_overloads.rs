use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "useDecString", "useDecSymbol")
}
