use tsox_lsp::fourslash::Session;


#[test]
fn member_list_inside_object_literals() {
    let content = r#"namespace ObjectLiterals {
    interface MyPoint {
        x1: number;
        y1: number;
    }

    var p1: MyPoint = {
        /*1*/
    };

    var p2: MyPoint = {
        x1: 5,
        /*2*/
    };

    var p3: MyPoint = {
        x1/*3*/:
    };

    var p4: MyPoint = {
        /*4*/y1
    };
}"#;
    let _s = Session::new_for_test("memberListInsideObjectLiterals", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "3", "4"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
}
