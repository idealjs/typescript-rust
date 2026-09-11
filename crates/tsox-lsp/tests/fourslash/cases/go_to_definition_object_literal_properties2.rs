use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_object_literal_properties2() {
    let content = r#"type C = {
  foo: string;
  bar: number;
};

declare function fn<T extends C>(arg: T): T;

fn({
  foo/*1*/: "",
  bar/*2*/: true,
});

const result = fn({
  foo/*3*/: "",
  bar/*4*/: 1,
});

// this one shouldn't go to the constraint type
result.foo/*5*/;"#;
    let mut s = Session::new_for_test("goToDefinitionObjectLiteralProperties2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1", "2", "3", "4", "5")
}
