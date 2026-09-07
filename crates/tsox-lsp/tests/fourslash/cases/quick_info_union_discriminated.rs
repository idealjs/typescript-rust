use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_union_discriminated() {
    let content = r#"// @Filename: quickInfoJsDocTags.ts
type U = A | B;

interface A {
    /** Kind A */
    kind: "a";
    /** Prop A */
    prop: number;
}

interface B {
    /** Kind B */
    kind: "b";
    /** Prop B */
    prop: string;
}

const u: U = {
    /*uKind*/kind: "a",
    /*uProp*/prop: 0,
}
const u2: U = {
    /*u2Kind*/kind: "bogus",
    /*u2Prop*/prop: 1,
};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "uKind", "(property) A.kind: \"a\"", "Kind A")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "uProp", "(property) A.prop: number", "Prop A")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "u2Kind", "(property) kind: \"bogus\"", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "u2Prop", "(property) prop: number", "")
}
