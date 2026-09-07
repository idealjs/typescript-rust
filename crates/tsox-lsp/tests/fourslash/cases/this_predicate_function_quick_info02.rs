use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn this_predicate_function_quick_info02() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface Sundries {
    broken: boolean;
}

interface Supplies {
    spoiled: boolean;
}

interface Crate<T> {
    contents: T;
    /*1*/isSundries(): this is Crate<Sundries>;
    /*2*/isSupplies(): this is Crate<Supplies>;
    /*3*/isPackedTight(): this is (this & {extraContents: T});
}
const crate: Crate<any>;
if (crate.isPackedTight/*4*/()) {
    crate.;
}
if (crate.isSundries/*5*/()) {
    crate.contents.;
    if (crate.isPackedTight/*6*/()) {
       crate.;
    }
}
if (crate.isSupplies/*7*/()) {
    crate.contents.;
    if (crate.isPackedTight/*8*/()) {
       crate.;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(method) Crate<T>.isSundries(): this is Crate<Sundries>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(method) Crate<T>.isSupplies(): this is Crate<Supplies>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(method) Crate<T>.isPackedTight(): this is (this & {\n    extraContents
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(method) Crate<any>.isPackedTight(): this is (Crate<any> & {\n    extra
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(method) Crate<any>.isSundries(): this is Crate<Sundries>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "(method) Crate<Sundries>.isPackedTight(): this is (Crate<Sundries> & {\
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "7", "(method) Crate<any>.isSupplies(): this is Crate<Supplies>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "8", "(method) Crate<Supplies>.isPackedTight(): this is (Crate<Supplies> & {\
}
