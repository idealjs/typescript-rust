use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity_intersection1() {
    let content = r#"{
    type Foo = { a: "a" | "c" };
    type Bar = { a: "a" | "b" };
    const obj/*o1*/: Foo & Bar = { a: "a" };
}
{
    type Foo = { a: "c" };
    type Bar = { a: "b" };
    const obj/*o2*/: Foo & Bar = { a: "" };
}
{
    type Foo = { a: "c" };
    type Bar = { a: "b" };
    type Never = Foo & Bar;
    const obj/*o3*/: Never = { a: "" };
}"#;
    let _s = Session::new_for_test("quickinfoVerbosityIntersection1", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"o1": {0, 1}, "o2": {0}, "o3": {0}})
}
