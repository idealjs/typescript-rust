use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_parameter_property_with_conflicting_member() {
    let content = r#"
// @filename: c1.ts
class C1 {
  [|x|]() {}
  constructor(public [|x|]: number) {
    [|x|]++;
  }
}
new C1(1).[|x|];

// @filename: c2.ts
interface C2 {
  get [|x|](): void
}
class C2 {
  constructor(public [|x|]: number) {
    [|x|]++;
  }
}
new C2(1).[|x|];
"#;
    let mut s = Session::new_for_test("findAllRefsParameterPropertyWithConflictingMember", content);
    // TODO: f.VerifyBaselineFindAllReferences(t)
}
