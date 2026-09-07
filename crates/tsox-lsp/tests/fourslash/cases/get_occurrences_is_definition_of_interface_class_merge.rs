use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn get_occurrences_is_definition_of_interface_class_merge() {
    let content = r#"/*1*/interface /*2*/Numbers {
    p: number;
}
/*3*/interface /*4*/Numbers {
    m: number;
}
/*5*/class /*6*/Numbers {
    f(n: number) {
        return this.p + this.m + n;
    }
}
let i: /*7*/Numbers = new /*8*/Numbers();
let x = i.f(i.p + i.m);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7", "8")
}
