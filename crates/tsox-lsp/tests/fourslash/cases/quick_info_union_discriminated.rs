use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "uKind", "(property) A.kind: \"a\"", "Kind A");
    fourslash::verify_quick_info_at(&mut s, "uProp", "(property) A.prop: number", "Prop A");
    fourslash::verify_quick_info_at(&mut s, "u2Kind", "(property) kind: \"bogus\"", "");
    fourslash::verify_quick_info_at(&mut s, "u2Prop", "(property) prop: number", "");
}
