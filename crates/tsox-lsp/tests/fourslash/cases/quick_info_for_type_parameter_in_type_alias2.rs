use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_for_type_parameter_in_type_alias2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"type Call<AA> = { (): A/*1*/A };
type Index<AA> = {[foo: string]: A/*2*/A};
type GenericMethod<AA> = { method<BB>(): A/*3*/A & B/*4*/B }
type Nesting<TT> = { method<UU>(): new <WW>() => T/*5*/T & U/*6*/U & W/*7*/W };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(type parameter) AA in type Call<AA>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(type parameter) AA in type Index<AA>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(type parameter) AA in type GenericMethod<AA>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(type parameter) BB in method<BB>(): AA & BB", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(type parameter) TT in type Nesting<TT>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "(type parameter) UU in method<UU>(): new <WW>() => TT & UU & WW", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "7", "(type parameter) WW in <WW>(): TT & UU & WW", "")
}
