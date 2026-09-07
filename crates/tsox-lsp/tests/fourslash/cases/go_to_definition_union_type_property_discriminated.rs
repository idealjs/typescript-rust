use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_union_type_property_discriminated() {
    let content = r#"type U = A | B;

interface A {
  /*aKind*/kind: "a";
  /*aProp*/prop: number;
};

interface B {
  /*bKind*/kind: "b";
  /*bProp*/prop: string;
}

const u: U = {
  [|/*kind*/kind|]: "a",
  [|/*prop*/prop|]: 0,
};
const u2: U = {
  [|/*kindBogus*/kind|]: "bogus",
  [|/*propBogus*/prop|]: 0,
};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "kind", "prop", "kindBogus", "propBogus")
}
