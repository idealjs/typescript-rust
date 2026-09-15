use tsox_lsp::fourslash::{self, Session};

#[test]
fn probe_switch_case_string() {
    let content = r#"function /*a*/ ;
function* /*b*/ ;
interface I {
    abstract baseMethod(): Iterable<number>;
}
class C implements I {
    */*c*/ ;
    public */*d*/
}
const o: I = {
    */*e*/
};
1 * /*f*/"#;
    let mut s = Session::new_for_test("dbg", content);
    let m = s.marker("e").clone();
    let service = s.service.as_ref().unwrap();
    let list = service.debug_completion_labels(&m.file_name, m.position);
    eprintln!("labels={list:?}");
}

#[test]
fn probe_string_ctx() {
    let content = r#"
type ActorRef<TEvent extends { type: string }> = {
  send: (ev: TEvent) => void
}

type Action<TContext> = {
  (ctx: TContext): void
}

type Config<TContext> = {
  entry: Action<TContext>
}

declare function createMachine<TContext>(config: Config<TContext>): void

type EventFrom<T> = T extends ActorRef<infer TEvent> ? TEvent : never

declare function sendTo<
  TContext,
  TActor extends ActorRef<any>
>(
  actor: ((ctx: TContext) => TActor),
  event: EventFrom<TActor>
): Action<TContext>

createMachine<{
  child: ActorRef<{ type: "EVENT" }>;
}>({
  entry: sendTo((ctx) => ctx.child, { type: "/*2*/" }),
});"#;
    let mut s = Session::new_for_test("completionsForStringDependingOnContexSensitiveSignature", content);
    let m = s.marker("2").clone();
    let service = s.service.as_ref().unwrap();
    let file = m.file_name.clone();
    eprintln!("labels={:?}", service.debug_completion_labels(&file, m.position));
    eprintln!("ctx none={:?}", service.debug_contextual_type(&file, m.position, false));
    eprintln!("ctx ign={:?}", service.debug_contextual_type(&file, m.position, true));
}

#[test]
fn probe_recursive_generic_members() {
    let content = r#"export class TestBase<T>
{
    public publicMethod(p: any): void {}
    private privateMethod(p: any): void {}
    protected protectedMethod(p: any): void {}
    public test(t: T): void
    {
        t./**/
    }
}"#;
    let mut s = Session::new_for_test("completionsForRecursiveGenericTypesMember", content);
    let m = s.marker("").clone();
    let service = s.service.as_ref().unwrap();
    eprintln!("labels-plain={:?}", service.debug_completion_labels(&m.file_name, m.position));
    let content2 = r#"export class TestBase<T extends TestBase<T>>
{
    public publicMethod(p: any): void {}
    private privateMethod(p: any): void {}
    protected protectedMethod(p: any): void {}
    public test(t: T): void
    {
        t./**/
    }
}"#;
    let mut s2 = Session::new_for_test("completionsForRecursiveGenericTypesMember2", content2);
    let m2 = s2.marker("").clone();
    let service2 = s2.service.as_ref().unwrap();
    eprintln!("labels-rec={:?}", service2.debug_completion_labels(&m2.file_name, m2.position));
}

#[test]
fn probe_spread_arg_parse() {
    let text = "const [] = [Math.min(.)]";
    let file = tsox_frontend::parser::Parser::parse_source_file_text("probe.ts", text.to_string());
    fn walk(n: &tsox_frontend::ast::Node, text: &str, depth: usize) {
        let seg: String = text[n.pos().min(text.len())..n.end().min(text.len())].chars().take(18).collect();
        eprintln!("{}{:?} [{}..{}] `{}`", "  ".repeat(depth), n.kind, n.pos(), n.end(), seg);
        tsox_frontend::ast::node_data_generated::for_each_child(n, |c| { walk(c, text, depth + 1); true });
    }
    walk(&file.node, text, 0);
}

#[test]
fn probe_mapped_parse() {
    let text = "type Wrap<T> = { [K in Extract<keyof T, string> as `${K}Wrapped`]: T[K]; };";
    let (file, diags) = tsox_frontend::parser::Parser::parse_source_file_text_with_diagnostics("probe.ts", text.to_string());
    for d in &diags {
        eprintln!("diag [{}..{}] msg={}", d.range.pos(), d.range.end(), d.message);
    }
    let _ = &file.node;
    let content = "type Wrap<T> = { [K in Extract<keyof T, string> as `${K}Wrapped`]: T[K]; };\nlet x: Wrap<{ a: 1 }>;\n";
    let mut s = Session::new_for_test("mappedMin", content);
    let service = s.service.as_ref().unwrap();
    eprintln!("labels={:?}", service.debug_completion_labels("mappedMin.ts", content.len() - 1));
}

#[test]
fn probe_namespace_merged_value() {
    let content = r#"namespace N {
    export type T = number;
}
const N = { m() {} };
let x: N./*type*/;
N./*value*/;"#;
    let mut s = Session::new_for_test("completionsNamespaceMergedWithObject", content);
    let m = s.marker("value").clone();
    let service = s.service.as_ref().unwrap();
    eprintln!("labels={:?}", service.debug_completion_labels(&m.file_name, m.position));
    let contents_pos = content.find("crate.contents.").unwrap() + 6;
    eprintln!("contents-qic={:?}", service.debug_quick_info(&m.file_name, contents_pos));
    let crate_pos = content.find("crate.contents.").unwrap();
    eprintln!("crate-qic={:?}", service.debug_quick_info(&m.file_name, crate_pos + 1));
}

#[test]
fn probe_wrapped_class() {
    let content = r#"class Client {
    private close() { }
    public open() { }
}
type Wrap<T> = T &
{
    [K in Extract<keyof T, string> as `${K}Wrapped`]: T[K];
};
class Service {
    method() {
        let service = undefined as unknown as Wrap<Client>;
        const { /*a*/ } = service;
    }
}"#;
    let mut s = Session::new_for_test("completionsWrappedClass", content);
    let m = s.marker("a").clone();
    let service = s.service.as_ref().unwrap();
    eprintln!("labels={:?}", service.debug_completion_labels(&m.file_name, m.position));
    let contents_pos = content.find("crate.contents.").unwrap() + 6;
    eprintln!("contents-qic={:?}", service.debug_quick_info(&m.file_name, contents_pos));
    let crate_pos = content.find("crate.contents.").unwrap();
    eprintln!("crate-qic={:?}", service.debug_quick_info(&m.file_name, crate_pos + 1));
}

#[test]
fn probe_mixin_ctor() {
    let content = r#"type MixinCtor<A, B> = new () => A & B & { constructor: MixinCtor<A, B> };
function merge<A, B>(a: { prototype: A }, b: { prototype: B }): MixinCtor<A, B> {
  return null;
}

class TreeNode {
  value: any;
}

abstract class LeftSideNode extends TreeNode {
  abstract right(): TreeNode;
  left(): TreeNode {
    return null;
  }
}

abstract class RightSideNode extends TreeNode {
  abstract left(): TreeNode;
  right(): TreeNode {
    return null;
  };
}

var obj = new (merge(LeftSideNode, RightSideNode))();
obj./**/"#;
    let mut s = Session::new_for_test("mixinCtorProbe", content);
    let service = s.service.as_ref().unwrap();
    let m = s.marker("").clone();
    eprintln!("labels={:?}", service.debug_completion_labels(&m.file_name, m.position));
    let contents_pos = content.find("crate.contents.").unwrap() + 6;
    eprintln!("contents-qic={:?}", service.debug_quick_info(&m.file_name, contents_pos));
    let crate_pos = content.find("crate.contents.").unwrap();
    eprintln!("crate-qic={:?}", service.debug_quick_info(&m.file_name, crate_pos + 1));
    eprintln!("ctx={:?}", service.debug_contextual_type(&m.file_name, m.position, false));
    let obj_pos = content.find("obj.").unwrap();
    eprintln!("obj-qic={:?}", service.debug_quick_info(&m.file_name, obj_pos));
    let merge_pos = content.find("merge(LeftSideNode").unwrap();
    eprintln!("merge-qic={:?}", service.debug_quick_info(&m.file_name, merge_pos));
    let new_pos = content.find("new (merge").unwrap();
    eprintln!("new-qic={:?}", service.debug_quick_info(&m.file_name, new_pos + 1));
    let alias_ref_pos = content.find("): MixinCtor<A, B> {").unwrap() + 3;
    eprintln!("aliasref-qic={:?}", service.debug_quick_info(&m.file_name, alias_ref_pos));
    let alias_decl_pos = content.find("type MixinCtor").unwrap() + 5;
    eprintln!("aliasdecl-qic={:?}", service.debug_quick_info(&m.file_name, alias_decl_pos));
    let content2 = content.replace("{ constructor: MixinCtor<A, B> }", "{ }");
    let mut s2 = Session::new_for_test("mixinCtorProbe2", &content2);
    let service2 = s2.service.as_ref().unwrap();
    let merge_pos2 = content2.find("merge(LeftSideNode").unwrap();
    eprintln!("merge2-qic={:?}", service2.debug_quick_info("mixinCtorProbe2.ts", merge_pos2));
}

#[test]
fn probe_this_predicate() {
    let content = r#"interface Sundries {
    broken: boolean;
}
interface Crate<T> {
    contents: T;
    isSundries(): this is Crate<Sundries>;
    isPackedTight(): this is (this & {extraContents: T});
}
const crate: Crate<any>;
if (crate.isPackedTight()) {
    crate.contents;
}
if (crate.isSundries()) {
    crate.contents./**/;
}"#;
    let mut s = Session::new_for_test("thisPred", content);
    let service = s.service.as_ref().unwrap();
    let text = "const crate: Crate<any>;\nif (crate.isSundries()) {\n    crate.contents.;\n}";
    let file = tsox_frontend::parser::Parser::parse_source_file_text("probe.ts", text.to_string());    fn walk(n: &tsox_frontend::ast::Node, text: &str, depth: usize) {
        let seg: String = text[n.pos().min(text.len())..n.end().min(text.len())].chars().take(18).collect();
        eprintln!("{}{:?} [{}..{}] `{}`", "  ".repeat(depth), n.kind, n.pos(), n.end(), seg);
        tsox_frontend::ast::node_data_generated::for_each_child(n, |c| { walk(c, text, depth + 1); true });
    }
    walk(&file.node, text, 0);
    let m = s.marker("").clone();
    eprintln!("labels={:?}", service.debug_completion_labels(&m.file_name, m.position));
    let contents_pos = content.find("crate.contents.").unwrap() + 6;
    eprintln!("contents-qic={:?}", service.debug_quick_info(&m.file_name, contents_pos));
    let crate_pos = content.find("crate.contents.").unwrap();
    eprintln!("crate-qic={:?}", service.debug_quick_info(&m.file_name, crate_pos + 1));
    let m2 = s.marker("2").clone();
    eprintln!("labels2={:?}", service.debug_completion_labels(&m2.file_name, m2.position));
}

#[test]
fn probe_partial_constraint() {
    let content = r#"type Colors = {
    rgb: { r: number, g: number, b: number };
    hsl: { h: number, s: number, l: number }
};

function createColor<T extends keyof Colors>(kind: T, values: Colors[T]) { }

createColor('rgb', {
  /*3*/
});

function f4<T extends 'a' | 'b'>(p: { kind: T } & X[T]) { }
type X = { a: { a }, b: { b } }
f4({
    kind: "a",
    /*6*/
})"#;
    let mut s = Session::new_for_test("completionsObjectLiteralWithPartialConstraint", content);
    for name in ["3", "6"] {
        let m = s.marker(name).clone();
        let service = s.service.as_ref().unwrap();
        eprintln!("m{name} labels={:?}", service.debug_completion_labels(&m.file_name, m.position));
        eprintln!("m{name} ctx={:?}", service.debug_contextual_type(&m.file_name, m.position, false));
    }
}

#[test]
fn probe_js_module_exports() {
    let content = r#"// @allowJs: true
// @module: commonjs
// @Filename: mod.js
function foo() { return {a: "hello, world"}; }
module.exports = foo();
// @Filename: mod2.js
var x = {name: 'test'};
(function createExport(obj){
    module.exports = {
        "default": x,
        "sausages": {eggs: 2}
    };
})();
// @Filename: app.js
import {/*a*/a} from "./mod"
import def, {sausages} from "./mod2"
const d2 = /*d*/def;
def./**/"#;
    let mut s = Session::new_for_test("jsModProbe", content);
    let service = s.service.as_ref().unwrap();
    let m = s.marker("").clone();
    eprintln!("labels={:?}", service.debug_completion_labels(&m.file_name, m.position));
    let ma = s.marker("a").clone();
    eprintln!("import-a-qic={:?}", service.debug_quick_info(&ma.file_name, ma.position));
    let md = s.marker("d").clone();
    eprintln!("d-qic={:?}", service.debug_quick_info(&md.file_name, md.position));
}

#[test]
fn probe_typeof_module_display() {
    let content = r#"// @Filename: /node_modules/@types/three/index.d.ts
export class Vector3 {}
export as namespace THREE;
// @Filename: /global.d.ts
import * as _THREE from 'three';
declare global {
  const THREE: typeof _THREE;
}
// @Filename: /index.ts
let v = new /*1*/THREE.Vector3();
const w = /*2*/_THREE;"#;
    let mut s = Session::new_for_test("typeofModProbe", content);
    let service = s.service.as_ref().unwrap();
    let m1 = s.marker("1").clone();
    eprintln!("three-qic={:?}", service.debug_quick_info(&m1.file_name, m1.position));
    let m2 = s.marker("2").clone();
    eprintln!("m-qic={:?}", service.debug_quick_info(&m2.file_name, m2.position));
}

#[test]
fn probe_json_kind() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @strict: true
// @jsx: preserve
// @resolveJsonModule: true
// @Filename: /index.js
export const x = 0;
// @Filename: /jsx.jsx
export const y = 0;
// @Filename: /j.jonah.json
{ "j": 0 }
// @Filename: /a.js
import { x as x0 } from ".";
import { x as x1 } from "./index";
import { x as x2 } from "./index.js";
import { y } from "./jsx.jsx";
import { j } from "./j.jonah.json";"#;
    let mut s = Session::new_for_test("jsonKindProbe", content);
    let service = s.service.as_ref().unwrap();
    fourslash::debug_dump_diagnostics(&s, "/a.js");
    let program = service.get_program();
    for f in program.source_files() {
        if f.file_name.ends_with(".json") {
            eprintln!("[json-kind] {} script_kind={:?}", f.file_name, f.script_kind);
        }
    }
}

#[test]
fn probe_fmt_scanner_jsx() {
    let text = "(\n    <input\n        value=\"x\n        x\"\n    />\n);\n";
    let (file, diags) = tsox_frontend::parser::Parser::parse_source_file_text_with_diagnostics("p.tsx", text.to_string());
    eprintln!("diags={}", diags.len());
    let file = std::sync::Arc::new(file);
    let service_opts = tsox_lsp::ls::lsutil::FormatCodeSettings::default();
    let ctx_opts = {
        // 与 ls::format::to_engine_settings 同构的最小默认
        tsox_frontend::format::FormatCodeSettings::default_or(service_opts)
    };
    let ctx = tsox_frontend::format::with_format_code_settings(ctx_opts, "\n");
    let changes = tsox_frontend::format::format_document(&ctx, &file);
    for c in &changes {
        let seg: String = text[c.pos.min(text.len())..c.end.min(text.len())].chars().take(20).collect();
        eprintln!("change [{}..{}] `{}` -> `{:?}`", c.pos, c.end, seg, c.new_text);
    }
}
