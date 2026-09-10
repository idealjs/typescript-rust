use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // Using separate files for each example to avoid unclosed J"]
#[test]
fn auto_close_tags_with_trivia_and_complex_names() {
    // TODO: // Using separate files for each example to avoid unclosed JSX tags affecting other tests.
    let content = r#"// @noLib: true

// @Filename: /0.tsx
// JSDoc
const x = <
	/** hello world! */
	div /** hello world! */
	>/*0*/

// @Filename: /1.tsx
// Single-line comments
const x =
	<
	// hello world!
	div // hello world!
	>/*1*/

// @Filename: /2.tsx
// Namespaced tag
const x =
	<ns:sometag>/*2*/

// @Filename: /3.tsx
// Namespace with single-line comments
const x = <
	// pre-ns	
	ns
	// pre-colon
	:
	// post-colon
	sometag
	// post-id
	>/*3*/

// @Filename: /4.tsx
// UppercaseComponent-named tag
const x = <SomeComponent>/*4*/

// @Filename: /5.tsx
// propertyAccess.Component-named tag
const x = <
	someModule
	.
	SomeComponent
>/*5*/

// @Filename: /6.tsx
// propertyAccess.Component-named tag with single-line comments
const x =
	<
	// pre-object
	someModule
	// pre-dot
	.
	// post-dot
	SomeComponent
	// post-id
	>/*6*/;

// @Filename: /7.tsx
// Generic propertyAccess.Component-named tag
const x =
	<
	someModule.SomeComponent<string>
	prop="stringValue"
	>/*7*/;

// @Filename: /8.tsx
// Namespaced tag with hyphens
const x =
	<my-namespace:my-tag>/*8*/

// @Filename: /9.tsx
// Generic tag with no attributes
const x = <SomeComponent<number>>/*9*/

// @Filename: /10.tsx
// Tag name containing $ (must be snippet-escaped)
const x = <$Foo>/*10*/
"#;
    let mut s = Session::new_for_test("autoCloseTagsWithTriviaAndComplexNames", content);
    fourslash::unsupported("VerifyBaselineClosingTags"); // f.VerifyBaselineClosingTags(t)
}
