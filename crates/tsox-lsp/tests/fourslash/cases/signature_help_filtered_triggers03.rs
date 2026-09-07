use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Insert"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("Insert"); // f.Insert(t, "(")
}
