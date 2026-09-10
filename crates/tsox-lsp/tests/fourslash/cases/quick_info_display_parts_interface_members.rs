use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_display_parts_interface_members() {
    let content = r#"interface I {
    /*1*/property: string;
    /*2*/method(): string;
    (): string;
    new (): I;
}
var iInstance: I;
/*3*/iInstance./*4*/property = /*5*/iInstance./*6*/method();
/*7*/iInstance();
var /*8*/anotherInstance = new /*9*/iInstance();"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsInterfaceMembers", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
