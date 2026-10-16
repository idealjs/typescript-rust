use tsox_lsp::fourslash::{self, Session};


#[test]
fn generic_type_with_multiple_bases1_multi_file() {
    let content = r#"// @Filename: genericTypeWithMultipleBases_0.ts
interface iBaseScope {
    watch: () => void;
}
// @Filename: genericTypeWithMultipleBases_1.ts
interface iMover {
    moveUp: () => void;
}
// @Filename: genericTypeWithMultipleBases_2.ts
interface iScope<TModel> extends iBaseScope, iMover {
    family: TModel;
}
// @Filename: genericTypeWithMultipleBases_3.ts
var x: iScope<number>;
// @Filename: genericTypeWithMultipleBases_4.ts
x./**/"#;
    let mut s = Session::new_for_test("genericTypeWithMultipleBases1MultiFile", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
