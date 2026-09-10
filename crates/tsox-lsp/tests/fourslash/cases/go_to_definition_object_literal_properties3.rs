use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_object_literal_properties3() {
    let content = r#"type A = {
  foo: unknown;
};

type B = {
  foo?: unknown;
  bar: unknown;
};

function test1(arg: A | B) {}

test1({
  foo/*1*/: 1,
});

function test2<T extends A>(arg: T | B) {}

test2({
  foo/*2*/: 2,
});"#;
    let mut s = Session::new_for_test("goToDefinitionObjectLiteralProperties3", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1", "2")
}
