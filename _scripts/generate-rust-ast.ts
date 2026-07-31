/**
 * Rust AST code generator: reads _scripts/ast.json and produces:
 *   - src/ast/syntax_kind_generated.rs
 *   - src/ast/node_data_generated.rs
 *
 * Usage: node --experimental-strip-types _scripts/generate-rust-ast.ts
 *
 * Generates:
 *   - SyntaxKind enum
 *   - NodeData structs and enum
 *   - for_each_child dispatch
 *   - Node accessor methods (node_text, node_expression, node_name, node_type)
 *   - Type guard functions (is_token, is_identifier, etc.)
 *   - Kind alias guard functions (is_trivia_kind, is_literal_kind, etc.)
 */

import * as fs from "node:fs";
import * as path from "node:path";
import type {
    AliasType,
    KindType,
    ListType,
    MemberInfo,
    NodeType,
    PrimitiveType,
    Type,
    TypeParameterType,
    UnionType,
} from "./schema.ts";
import {
    api,
    kindGuardName,
} from "./schema.ts";

// ────────────────────────────────────────────────────────────────────────────
// Constants
// ────────────────────────────────────────────────────────────────────────────

const ROOT = path.resolve(import.meta.dirname!, "..");

// ────────────────────────────────────────────────────────────────────────────
// String helpers
// ────────────────────────────────────────────────────────────────────────────

/** Convert PascalCase/camelCase to snake_case, with JSDoc and Rust keyword handling. */
function toSnakeCase(name: string): string {
    // Normalise "JSDoc" → "Jsdoc" so it is treated as a single word anywhere
    // in the name (not just at the start).
    name = name.replace(/JSDoc/g, "Jsdoc");
    let result = "";
    for (let i = 0; i < name.length; i++) {
        const c = name[i];
        if (c >= "A" && c <= "Z" && i > 0) {
            result += "_";
        }
        result += c.toLowerCase();
    }
    // Rust keyword avoidance
    if (result === "type") return "type_node";
    return result;
}

// ────────────────────────────────────────────────────────────────────────────
// Type mapping helpers
// ────────────────────────────────────────────────────────────────────────────

const PRIMITIVE_RUST_MAP: Record<string, string> = {
    "string": "String",
    "bool": "bool",
    "boolean": "bool",
    "int": "i32",
    "NodeFlags": "NodeFlags",
    "ModifierFlags": "ModifierFlags",
    "TokenFlags": "TokenFlags",
};

/** Resolve a type to its underlying primitive type name, if any. */
function resolvePrimitiveName(type: Type): string | undefined {
    let t = type;
    while (true) {
        if (t.kind === "primitive") return (t as PrimitiveType).name;
        if (t.kind === "alias") {
            t = (t as AliasType).resolved;
            continue;
        }
        if (t.kind === "typeParameter") {
            t = (t as TypeParameterType).constraint;
            continue;
        }
        return undefined;
    }
}

/** Map a schema type to its Rust type string (without Option wrapping). */
function rustBaseType(type: Type): string {
    if (type.kind === "list") {
        const listType = type as ListType;
        if (listType.listKind === "NodeList") return "Arc<NodeList>";
        if (listType.listKind === "ModifierList") return "Arc<ModifierList>";
        // raw list
        const elemBase = listType.elementType.baseKind();
        if (elemBase === "node") return "Vec<Arc<Node>>";
        const primName = resolvePrimitiveName(listType.elementType);
        if (primName) return `Vec<${PRIMITIVE_RUST_MAP[primName] || "String"}>`;
        return "Vec<Arc<Node>>"; // fallback
    }

    const baseKind = type.baseKind();
    if (baseKind === "node") return "Arc<Node>";
    if (baseKind === "kind") return "SyntaxKind";
    if (baseKind === "primitive") {
        const primName = resolvePrimitiveName(type);
        return primName ? (PRIMITIVE_RUST_MAP[primName] || "String") : "String";
    }
    if (baseKind === "list") {
        // Alias resolving to a list — unwrap
        if (type.kind === "alias") return rustBaseType((type as AliasType).resolved);
        return "Arc<NodeList>";
    }
    return "String";
}

/** Full Rust type for a member, including Option wrapping. */
function rustType(member: MemberInfo): string {
    const base = rustBaseType(member.type);
    // `any`-typed members are treated as optional since `any` subsumes `undefined`.
    const isOptional = member.optional || resolvePrimitiveName(member.type) === "any";
    return isOptional ? `Option<${base}>` : base;
}

/** Rust field name for a member (snake_case + keyword avoidance). */
function rustFieldName(member: MemberInfo): string {
    return toSnakeCase(member.name);
}

// ────────────────────────────────────────────────────────────────────────────
// Base hierarchy helpers
// ────────────────────────────────────────────────────────────────────────────

/** Check if a node extends (directly or indirectly) the given base. */
function extendsBase(node: NodeType, baseName: string): boolean {
    const visited = new Set<string>();
    function check(n: NodeType): boolean {
        if (visited.has(n.name)) return false;
        visited.add(n.name);
        for (const ext of n.extends) {
            if (ext.name === baseName) return true;
            if (check(ext)) return true;
        }
        return false;
    }
    return check(node);
}

// ────────────────────────────────────────────────────────────────────────────
// Member filtering
// ────────────────────────────────────────────────────────────────────────────

/** Members included in Rust structs (no goOnly, no noGo, no Kind param, no NodeFlags). */
function structMembers(node: NodeType): MemberInfo[] {
    return node.members.filter(m => {
        if (m.goOnly || m.noGo || m.isKindParam()) return false;
        // NodeFlags members are stored on the Node wrapper, not on NodeData.
        const primName = resolvePrimitiveName(m.type);
        if (primName === "NodeFlags") return false;
        return true;
    });
}

/** Child members visited in for_each_child (no goOnly, no noGo, no noFactory, must be child). */
function childMembers(node: NodeType): MemberInfo[] {
    return node.members.filter(m => !m.goOnly && !m.noGo && !m.noFactory && m.isChild());
}

// ────────────────────────────────────────────────────────────────────────────
// Code writer
// ────────────────────────────────────────────────────────────────────────────

class CodeWriter {
    private lines: string[] = [];
    private indent = 0;

    write(line: string = ""): void {
        if (line === "") {
            this.lines.push("");
        }
        else {
            this.lines.push("    ".repeat(this.indent) + line);
        }
    }

    /** Push a raw line without any indentation prefix. */
    writeRaw(line: string): void {
        this.lines.push(line);
    }

    push(): void {
        this.indent++;
    }

    pop(): void {
        this.indent--;
    }

    toString(): string {
        return this.lines.join("\n");
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Syntax kind generation
// ────────────────────────────────────────────────────────────────────────────

function generateSyntaxKind(): string {
    const w = new CodeWriter();
    w.write("// Code generated from _scripts/ast.json. DO NOT EDIT.");
    w.write("");
    w.write("/// TypeScript syntax kind, mirroring Go's `ast.Kind`.");
    w.write("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]");
    w.write("#[repr(i16)]");
    w.write("pub enum SyntaxKind {");

    const elements = api.kindElements();
    for (const el of elements) {
        if (!el.name) continue;
        if (el.name === "Unknown") {
            w.write("    #[default]");
        }
        w.write(`    ${el.name},`);
    }

    w.write("}");
    return w.toString() + "\n";
}

// ────────────────────────────────────────────────────────────────────────────
// Node data generation
// ────────────────────────────────────────────────────────────────────────────

function generateHeader(w: CodeWriter): void {
    w.write("// Code generated by _scripts/generate-rust-ast.ts. DO NOT EDIT.");
    w.write("");
    w.write("//! Generated AST node data types.");
    w.write("//!");
    w.write("//! This file is automatically generated from `_scripts/ast.json`.");
    w.write("//! Do not edit manually — edit ast.json and re-run the generator.");
    w.write("");
    w.write("#![allow(unused_imports)]");
    w.write("use super::SyntaxKind;");
    w.write("use super::node::{ModifierList, Node, NodeList};");
    w.write("use super::node_flags::{ModifierFlags, NodeFlags};");
    w.write("use crate::core::text::TextRange;");
    w.write("use std::sync::Arc;");
    w.write("");
    w.write("/// Token flags for lexical tokens.");
    w.write("pub type TokenFlags = u32;");
    w.write("");
}

// ── Node data structs ──────────────────────────────────────────────────────

function generateStructs(w: CodeWriter): void {
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("// Node data structs");
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("");

    for (const node of api.nodes()) {
        const members = structMembers(node);
        if (members.length === 0) continue;

        w.write("#[derive(Debug)]");
        w.write(`pub struct ${node.name}Data {`);
        w.push();
        for (const m of members) {
            w.write(`pub ${rustFieldName(m)}: ${rustType(m)},`);
        }
        w.pop();
        w.write("}");
        w.write("");
    }
}

// ── NodeData enum ──────────────────────────────────────────────────────────

function generateNodeDataEnum(w: CodeWriter): void {
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("// NodeData enum");
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("");
    w.write("/// Kind-specific data for an AST node.");
    w.write("///");
    w.write("/// In Go this is the `nodeData` interface with hundreds of implementations.");
    w.write("/// In Rust we use an enum so the compiler can verify exhaustiveness.");
    w.write("#[derive(Debug)]");
    w.write("pub enum NodeData {");

    for (const node of api.nodes()) {
        const members = structMembers(node);
        if (members.length === 0) {
            w.write(`    ${node.name},`);
        }
        else {
            w.write(`    ${node.name}(${node.name}Data),`);
        }
    }

    w.write("}");
    w.write("");
}

// ── for_each_child dispatch ─────────────────────────────────────────────────

function generateForEachChild(w: CodeWriter): void {
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("// for_each_child dispatch");
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("");
    w.write("/// Visit each child node of the given node.");
    w.write("///");
    w.write("/// Calls `visitor` for each child AST node. Returns true if the visitor");
    w.write("/// requested to stop (by returning true from the closure).");
    w.write("pub fn for_each_child<F>(node: &Node, mut visitor: F) -> bool");
    w.write("where");
    w.write("    F: FnMut(&Arc<Node>) -> bool,");
    w.write("{");
    w.push();
    w.write("match &node.data {");

    for (const node of api.nodes()) {
        const children = childMembers(node);
        if (children.length === 0) continue;

        w.push();
        w.write(`NodeData::${node.name}(data) => {`);
        w.push();

        for (const m of children) {
            const fieldName = rustFieldName(m);
            const baseKind = m.type.baseKind();

            if (baseKind === "node") {
                if (m.optional) {
                    w.write(`if let Some(child) = &data.${fieldName} {`);
                    w.push();
                    w.write("if visitor(child) {");
                    w.push();
                    w.write("return true;");
                    w.pop();
                    w.write("}");
                    w.pop();
                    w.write("}");
                }
                else {
                    w.write(`if visitor(&data.${fieldName}) {`);
                    w.push();
                    w.write("return true;");
                    w.pop();
                    w.write("}");
                }
            }
            else if (baseKind === "list") {
                if (m.optional) {
                    w.write(`if let Some(list) = &data.${fieldName} {`);
                    w.push();
                    w.write("for child in list.iter() {");
                    w.push();
                    w.write("if visitor(child) {");
                    w.push();
                    w.write("return true;");
                    w.pop();
                    w.write("}");
                    w.pop();
                    w.write("}");
                    w.pop();
                    w.write("}");
                }
                else {
                    w.write(`for child in data.${fieldName}.iter() {`);
                    w.push();
                    w.write("if visitor(child) {");
                    w.push();
                    w.write("return true;");
                    w.pop();
                    w.write("}");
                    w.pop();
                    w.write("}");
                }
            }
        }

        w.pop();
        w.write("}");
        w.pop();
    }

    w.write("    _ => {}");
    w.write("}");
    w.write("false");
    w.pop();
    w.write("}");
    w.write("");
}

// ── Node accessor methods ──────────────────────────────────────────────────

function generateAccessors(w: CodeWriter): void {
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("// Node accessor methods (generated)");
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("");

    // node_text: returns &str for nodes with a text: String field
    w.write("/// Get the text content of literal/identifier nodes.");
    w.write("pub fn node_text(node: &Node) -> &str {");
    w.push();
    w.write("match &node.data {");
    w.push();
    for (const node of api.nodes()) {
        const members = structMembers(node);
        const textMember = members.find(m => m.name.toLowerCase() === "text" && m.type.baseKind() === "primitive");
        if (textMember && resolvePrimitiveName(textMember.type) === "string") {
            w.write(`NodeData::${node.name}(d) => &d.${rustFieldName(textMember)},`);
        }
    }
    w.write(`_ => "",`);
    w.pop();
    w.write("}");
    w.pop();
    w.write("}");
    w.write("");

    // node_expression: returns Option<&Arc<Node>> for nodes with non-optional expression field
    w.write("/// Get the primary expression child of a node, if any.");
    w.write("pub fn node_expression(node: &Node) -> Option<&Arc<Node>> {");
    w.push();
    w.write("match &node.data {");
    w.push();
    for (const node of api.nodes()) {
        const members = structMembers(node);
        const exprMember = members.find(m =>
            m.name.toLowerCase() === "expression" &&
            m.type.baseKind() === "node" &&
            !m.optional
        );
        if (exprMember) {
            w.write(`NodeData::${node.name}(d) => Some(&d.${rustFieldName(exprMember)}),`);
        }
    }
    w.write("_ => None,");
    w.pop();
    w.write("}");
    w.pop();
    w.write("}");
    w.write("");

    // node_name: returns Option<&Arc<Node>> for nodes with a name field (node type)
    w.write("/// Get the name of a declaration, if any.");
    w.write("pub fn node_name(node: &Node) -> Option<&Arc<Node>> {");
    w.push();
    w.write("match &node.data {");
    w.push();
    for (const node of api.nodes()) {
        const members = structMembers(node);
        const nameMember = members.find(m => m.name.toLowerCase() === "name" && m.type.baseKind() === "node");
        if (nameMember) {
            if (nameMember.optional) {
                w.write(`NodeData::${node.name}(d) => d.${rustFieldName(nameMember)}.as_ref(),`);
            }
            else {
                w.write(`NodeData::${node.name}(d) => Some(&d.${rustFieldName(nameMember)}),`);
            }
        }
    }
    w.write("_ => None,");
    w.pop();
    w.write("}");
    w.pop();
    w.write("}");
    w.write("");

    // node_type: returns Option<&Arc<Node>> for type nodes and function-like declarations.
    // The node lists are curated to mirror Go's `Node.Type()` for type nodes and
    // function-like signature declarations (see ast.go `func (n *Node) Type()`).
    const TYPE_NODE_TYPES: string[] = [
        "ArrayTypeNode",
        "ParenthesizedTypeNode",
        "TypeOperatorNode",
        "OptionalTypeNode",
        "RestTypeNode",
        "NamedTupleMember",
        "FunctionTypeNode",
        "ConstructorTypeNode",
        "TypePredicateNode",
        "MappedTypeNode",
        "JSDocNonNullableType",
        "JSDocNullableType",
        "JSDocVariadicType",
        "JSDocOptionalType",
    ];
    const FUNCTION_LIKE_TYPES: string[] = [
        "FunctionDeclaration",
        "FunctionExpression",
        "ArrowFunction",
        "MethodDeclaration",
        "MethodSignatureDeclaration",
        "ConstructorDeclaration",
        "ConstructSignatureDeclaration",
        "CallSignatureDeclaration",
        "GetAccessorDeclaration",
        "SetAccessorDeclaration",
    ];

    w.write("/// Get the type child of a type node, if any.");
    w.write("///");
    w.write("/// Mirrors `Node.Type()` in Go — returns the `type` child node for type");
    w.write("/// nodes that have one (e.g. `ArrayType`, `ParenthesizedType`,");
    w.write("/// `TypeOperator`, `OptionalType`, `RestType`, `NamedTupleMember`,");
    w.write("/// `JSDocNullableType`, `JSDocNonNullableType`, `JSDocOptionalType`,");
    w.write("/// `JSDocVariadicType`, `FunctionType`, `ConstructorType`,");
    w.write("/// `TypePredicate`, `MappedType`).");
    w.write("pub fn node_type(node: &Node) -> Option<&Arc<Node>> {");
    w.push();
    w.write("match &node.data {");
    w.push();

    // Helper to emit a match arm for a node's Type/ElementType member.
    function emitTypeArm(nodeName: string): void {
        const node = api.getNode(nodeName);
        if (!node) return;
        const members = structMembers(node);
        const typeMember = members.find(m => m.name.toLowerCase() === "type" && m.type.baseKind() === "node");
        const elementTypeMember = members.find(m => m.name.toLowerCase() === "elementtype" && m.type.baseKind() === "node");
        const member = typeMember || elementTypeMember;
        if (!member) return;
        const fieldName = rustFieldName(member);
        if (member.optional) {
            w.write(`NodeData::${node.name}(d) => d.${fieldName}.as_ref(),`);
        }
        else {
            w.write(`NodeData::${node.name}(d) => Some(&d.${fieldName}),`);
        }
    }

    // First group: TypeNode-based nodes.
    for (const nodeName of TYPE_NODE_TYPES) {
        emitTypeArm(nodeName);
    }

    // Comment separating the TypeNode group from the FunctionLike group.
    w.write("// Function-like declarations: the `type_node` field holds the");
    w.write("// return-type annotation (e.g. `: x is string` in a type guard).");
    w.write("// Mirrors Go's `Node.Type()`, which returns the type annotation");
    w.write("// for signature declarations.");

    // Second group: FunctionLike declarations.
    for (const nodeName of FUNCTION_LIKE_TYPES) {
        emitTypeArm(nodeName);
    }

    w.write("_ => None,");
    w.pop();
    w.write("}");
    w.pop();
    w.write("}");
    w.write("");
}

// ── Type guard functions ───────────────────────────────────────────────────

function formatKindMatch(kinds: string[], indent: string): string[] {
    /** Format a list of kind names for a match arm. Inline if the resulting line
     *  width (indent + joined kinds + " => true,") is ≤ 100 (Rust's default
     *  max_width); otherwise one-per-line. */
    const prefixed = kinds.map(k => `SyntaxKind::${k}`);
    const inlineLine = `${indent}${prefixed.join(" | ")} => true,`;
    if (inlineLine.length <= 100) {
        return [inlineLine];
    }
    const lines: string[] = [];
    for (let i = 0; i < prefixed.length; i++) {
        const prefix = i === 0 ? "" : "| ";
        const suffix = i === prefixed.length - 1 ? " => true," : "";
        lines.push(`${indent}${prefix}${prefixed[i]}${suffix}`);
    }
    return lines;
}

function generateTypeGuards(w: CodeWriter): void {
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("// Type guard functions");
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("");

    for (const node of api.nodes()) {
        const kindTypes = node.kindTypes();
        const kindNames = kindTypes.map(k => k.name);

        w.write(`/// Check if a node is a ${node.name}.`);
        w.write(`pub fn is_${toSnakeCase(node.name)}(node: &Node) -> bool {`);

        if (kindNames.length <= 1) {
            const kind = kindNames[0] || node.syntaxKindName;
            w.write(`    node.kind == SyntaxKind::${kind}`);
        }
        else {
            w.write("    match node.kind {");
            for (const line of formatKindMatch(kindNames, "        ")) {
                w.writeRaw(line);
            }
            w.write("        _ => false,");
            w.write("    }");
        }

        w.write("}");
        w.write("");
    }
}

// ── Kind alias guard functions ──────────────────────────────────────────────

function generateKindAliasGuards(w: CodeWriter): void {
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("// Kind alias guard functions");
    w.write("// ──────────────────────────────────────────────────────────────────────");
    w.write("");

    for (const guard of api.kindGuards()) {
        const funcName = toSnakeCase(guard.guardName);

        if (guard.type === "range") {
            const firstKind = api.resolveKindMarkerValue(guard.first);
            const lastKind = api.resolveKindMarkerValue(guard.last);
            w.write(`pub fn ${funcName}(kind: SyntaxKind) -> bool {`);
            w.write(`    (kind as i16) >= (SyntaxKind::${firstKind} as i16)`);
            w.write(`        && (kind as i16) <= (SyntaxKind::${lastKind} as i16)`);
            w.write("}");
            w.write("");
        }
        else {
            const expanded = api.expandKindAliasMembers(guard.aliasName);
            const kindNames = expanded.map(k => k.name);
            w.write(`pub fn ${funcName}(kind: SyntaxKind) -> bool {`);
            w.write("    match kind {");
            for (const line of formatKindMatch(kindNames, "        ")) {
                w.writeRaw(line);
            }
            w.write("        _ => false,");
            w.write("    }");
            w.write("}");
            w.write("");
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Main generation
// ────────────────────────────────────────────────────────────────────────────

function generateNodeData(): string {
    const w = new CodeWriter();

    generateHeader(w);
    generateStructs(w);
    generateNodeDataEnum(w);
    generateForEachChild(w);
    generateAccessors(w);
    generateTypeGuards(w);
    generateKindAliasGuards(w);

    // The last section (generateKindAliasGuards) ends with w.write("") after each
    // guard, so w.toString() already ends with a single trailing newline.
    return w.toString();
}

// ────────────────────────────────────────────────────────────────────────────
// Write output
// ────────────────────────────────────────────────────────────────────────────

function writeFile(filePath: string, content: string): void {
    fs.writeFileSync(filePath, content);
    console.log(`Wrote ${filePath}`);
}

// ────────────────────────────────────────────────────────────────────────────
// Entry point
// ────────────────────────────────────────────────────────────────────────────

function main(): void {
    const syntaxKindPath = path.join(ROOT, "src", "ast", "syntax_kind_generated.rs");
    const nodeDataPath = path.join(ROOT, "src", "ast", "node_data_generated.rs");

    writeFile(syntaxKindPath, generateSyntaxKind());
    writeFile(nodeDataPath, generateNodeData());
}

main();
