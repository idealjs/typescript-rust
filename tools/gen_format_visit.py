#!/usr/bin/env python3
"""生成 format 模块的 visit_items：按字段序交错枚举节点的标量子节点与子列表。

输入是 crates/tsox-frontend/src/ast/node_data_generated.rs 的 struct 定义，
与 for_each_child 同源同序；区别在于本生成器保留空列表（Go 的
VisitEachChild 经 VisitNodes 钩子会访问空 NodeList，格式化引擎依赖
这一点消费列表的开闭 token）。

用法：python3 tools/gen_format_visit.py
输出：crates/tsox-frontend/src/format/visit_generated.rs
"""

import re
import sys

SRC = "crates/tsox-frontend/src/ast/node_data_generated.rs"
DST = "crates/tsox-frontend/src/format/visit_generated.rs"

struct_re = re.compile(r"pub struct (\w+Data) \{(.*?)\n\}", re.S)
field_re = re.compile(r"pub (\w+): (.+?),\n")
enum_re = re.compile(r"pub enum NodeData \{(.*?)\n\}", re.S)
variant_re = re.compile(r"^\s*(\w+)(?:\((\w+)\))?,", re.M)


def classify(ty):
    ty = ty.strip()
    if ty in ("Arc<Node>",):
        return "node"
    if ty in ("Option<Arc<Node>>",):
        return "opt_node"
    if ty in ("Arc<NodeList>",):
        return "list"
    if ty in ("Option<Arc<NodeList>>",):
        return "opt_list"
    if ty in ("Arc<ModifierList>", "Option<Arc<ModifierList>>"):
        return "modifiers"
    if ty in ("Vec<Arc<Node>>",):
        return "vec"
    return None


def main():
    text = open(SRC).read()

    structs = {}
    for m in struct_re.finditer(text):
        name, body = m.group(1), m.group(2)
        fields = []
        for fm in field_re.finditer(body + "\n"):
            kind = classify(fm.group(2))
            if kind:
                fields.append((fm.group(1), kind))
        structs[name] = fields

    em = enum_re.search(text)
    variants = [(v.group(1), v.group(2)) for v in variant_re.finditer(em.group(1))]

    out = []
    out.append("//! 由 tools/gen_format_visit.py 生成，勿手改。")
    out.append("//! 按字段序枚举标量子节点与子列表（保留空列表）。")
    out.append("#![allow(clippy::too_many_lines)]")
    out.append("")
    out.append("use std::sync::Arc;")
    out.append("")
    out.append("use crate::ast::node::{ModifierList, Node, NodeList};")
    out.append("use crate::ast::node_data_generated::NodeData;")
    out.append("")
    out.append("#[derive(Debug, Clone, Copy)]")
    out.append("pub(crate) enum VisitItem<'a> {")
    out.append("    Node(&'a Arc<Node>),")
    out.append("    List(&'a Arc<NodeList>),")
    out.append("    Modifiers(&'a Arc<ModifierList>),")
    out.append("    Slice(&'a [Arc<Node>]),")
    out.append("}")
    out.append("")
    out.append("pub(crate) fn visit_items<'a>(")
    out.append("    node: &'a Node,")
    out.append("    f: &mut dyn FnMut(VisitItem<'a>),")
    out.append(") {")
    out.append("    match &node.data {")

    def emit(arg, fields, indent):
        lines = []
        pad = " " * indent
        for fname, kind in fields:
            if kind == "node":
                lines.append(f"{pad}f(VisitItem::Node(&{arg}.{fname}));")
            elif kind == "opt_node":
                lines.append(f"{pad}if Some(x) = &{arg}.{fname} {{ f(VisitItem::Node(x)); }}".replace("if Some(x)", "if let Some(x)"))
            elif kind == "list":
                lines.append(f"{pad}f(VisitItem::List(&{arg}.{fname}));")
            elif kind == "opt_list":
                lines.append(f"{pad}if let Some(x) = &{arg}.{fname} {{ f(VisitItem::List(x)); }}")
            elif kind == "modifiers":
                lines.append(f"{pad}if let Some(x) = &{arg}.{fname} {{ f(VisitItem::Modifiers(x)); }}")
            elif kind == "vec":
                lines.append(f"{pad}f(VisitItem::Slice(&{arg}.{fname}));")
        return lines

    for variant, data_ty in variants:
        if data_ty is None:
            continue
        fields = structs.get(data_ty, [])
        body = emit("data", fields, 12)
        if not body:
            out.append(f"        NodeData::{variant}(_) => {{}}")
            continue
        out.append(f"        NodeData::{variant}(data) => {{")
        out.extend(body)
        out.append("        }")

    out.append("        _ => {}")
    out.append("    }")
    out.append("}")
    out.append("")

    open(DST, "w").write("\n".join(out))
    n_fields = sum(len(structs.get(t, [])) for _, t in variants if t)
    print(f"生成 {DST}：{len(variants)} 变体，{n_fields} 个子项字段", file=sys.stderr)


if __name__ == "__main__":
    main()
