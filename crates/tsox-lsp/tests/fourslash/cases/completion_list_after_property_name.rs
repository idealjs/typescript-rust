use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_property_name() {
    let content = r#"// @Filename: a.ts
class Test1 {
	public some /*afterPropertyName*/
}
// @Filename: b.ts
class Test2 {
	public some(/*inMethodParameter*/
}
// @Filename: c.ts
class Test3 {
	public some(a/*atMethodParameter*/
}
// @Filename: d.ts
class Test4 {
	public some(a /*afterMethodParameter*/
}
// @Filename: e.ts
class Test5 {
	public some(a /*afterMethodParameterBeforeComma*/,
}
// @Filename: f.ts
class Test6 {
	public some(a, /*afterMethodParameterComma*/
}
// @Filename: g.ts
class Test7 {
	constructor(/*inConstructorParameter*/
}
// @Filename: h.ts
class Test8 {
	constructor(public /*inConstructorParameterAfterModifier*/
}
// @Filename: i.ts
class Test9 {
	constructor(a/*atConstructorParameter*/
}
// @Filename: j.ts
class Test10 {
	constructor(public/*atConstructorParameterModifier*/
}
// @Filename: k.ts
class Test11 {
	constructor(public a/*atConstructorParameterAfterModifier*/
}
// @Filename: l.ts
class Test12 {
	constructor(a /*afterConstructorParameter*/
}
// @Filename: m.ts
class Test13 {
	constructor(a /*afterConstructorParameterBeforeComma*/,
}
// @Filename: n.ts
class Test14 {
	constructor(public a, /*afterConstructorParameterComma*/
}"#;
    let mut s = Session::new_for_test("completionListAfterPropertyName", content);
    // TODO: f.VerifyCompletions(t, []string{"afterPropertyName", "inMethodParameter", "atMethodParameter", "afte
    // TODO: f.VerifyCompletions(t, []string{"inConstructorParameter", "inConstructorParameterAfterModifier", "at
}
