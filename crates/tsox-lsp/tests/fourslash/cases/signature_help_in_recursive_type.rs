use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_in_recursive_type() {
    let content = r#"type Tail<T extends any[]> =
	((...args: T) => any) extends ((head: any, ...tail: infer R) => any) ? R : never;

type Reverse<List extends any[]> = _Reverse<List, []>;

type _Reverse<Source extends any[], Result extends any[] = []> = {
	1: Result,
	0: _Reverse<Tail<Source>, 0>,
}[Source extends [] ? 1 : 0];

type Foo = Reverse<[0,/**/]>;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "Reverse<List extends any[]>"})
}
