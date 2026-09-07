use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_object_literal_properties1() {
    let content = r#"interface PropsBag {
   /*first*/propx: number
}
function foo(arg: PropsBag) {}
foo({
   [|pr/*p1*/opx|]: 10
})
function bar(firstarg: boolean, secondarg: PropsBag) {}
bar(true, {
   [|pr/*p2*/opx|]: 10
})"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "p1", "p2")
}
