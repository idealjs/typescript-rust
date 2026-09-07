use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToTypeDefinition"]
#[test]
fn go_to_type_definition_promise_type() {
    let content = r#"// @lib: es5,es2015.promise
type User = { name: string };
async function /*reference*/getUser() { return { name: "Bob" } satisfies User as User }

const /*reference2*/promisedBob = getUser() 

export {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "reference", "reference2")
}
