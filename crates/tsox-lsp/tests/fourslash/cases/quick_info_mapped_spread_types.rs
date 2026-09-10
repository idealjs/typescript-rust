use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyQuickInfoIs"]
#[test]
fn quick_info_mapped_spread_types() {
    let content = r#"interface Foo {
    /** Doc */
    bar: number;
}

const f: Foo = { bar: 0 };
f./*f*/bar;

const f2: { [TKey in keyof Foo]: string } = { bar: "0" };
f2./*f2*/bar;

const f3 = { ...f };
f3./*f3*/bar;

const f4 = { ...f2 };
f4./*f4*/bar;"#;
    let mut s = Session::new_for_test("quickInfoMappedSpreadTypes", content);
    fourslash::go_to_marker(&mut s, "f");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(property) Foo.bar: number", "Doc")
    fourslash::go_to_marker(&mut s, "f2");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(property) bar: string", "Doc")
    fourslash::go_to_marker(&mut s, "f3");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(property) Foo.bar: number", "Doc")
    fourslash::go_to_marker(&mut s, "f4");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(property) bar: string", "Doc")
}
