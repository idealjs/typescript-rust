use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_getter_and_setter() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class Test {
    constructor() {
        this.value;
    }

    /** Getter text */
    get val/*1*/ue() {
        return this.value;
    }

    /** Setter text */
    set val/*2*/ue(value) {
        this.value = value;
    }
}"#;
    let mut s = Session::new_for_test("quickInfoForGetterAndSetter", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyQuickInfoIs(t, "(getter) Test.value: any", "Getter text")
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyQuickInfoIs(t, "(setter) Test.value: any", "Setter text")
}
