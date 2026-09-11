use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_filtered_triggers03() {
    let content = r#"declare class ViewJayEss {
    constructor(obj: object);
}
new ViewJayEss({
    methods: {
        sayHello/**/
    }
});"#;
    let mut s = Session::new_for_test("signatureHelpFilteredTriggers03", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.Insert(t, "(")
}
