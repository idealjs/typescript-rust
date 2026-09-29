#![allow(unused_imports)]

use crate::checker::typenode_references::*;
use tsox_frontend::ast::NodeData;

impl Checker {
    /// 根标识符可解析、其变量注解型为构造期 error 预置(在途环回落)判定
    pub(crate) fn type_query_root_in_flight(&self, expr_name: &Arc<Node>) -> bool {
        let mut leftmost = expr_name;
        loop {
            match &leftmost.data {
                NodeData::QualifiedName(q) => leftmost = &q.left,
                NodeData::PropertyAccessExpression(pa) => leftmost = &pa.expression,
                _ => break,
            }
        }
        if leftmost.kind != SyntaxKind::Identifier {
            return false;
        }
        let Some(sym) = self.resolve_identifier(leftmost) else {
            return false;
        };
        let Some(decl) = sym.value_declaration.as_ref() else {
            return false;
        };
        let NodeData::VariableDeclaration(d) = &decl.data else {
            return false;
        };
        d.type_node.as_ref().is_some_and(|tn| {
            self.get_cached_type(tn)
                .is_some_and(|t| crate::checker::utilities::is_type_error(&t))
        })
    }

    fn accessor_sibling(
        member: &Arc<Node>,
        kind: SyntaxKind,
        name: &str,
    ) -> Option<Arc<Node>> {
        let parent = member.parent()?;
        let members = match &parent.data {
            NodeData::TypeLiteralNode(d) => &d.members,
            _ => return None,
        };
        members
            .iter()
            .find(|m| {
                m.kind == kind
                    && m.name()
                        .is_some_and(|n| n.text() == name)
            })
            .cloned()
    }

    /// Go getTypeOfAccessors 的解析序(getter 注解 → setter 参数注解 → any)在
    /// 类型字面量成员上的等价物;同对存在带注解 getter 时 setter 参数注解不
    /// 求值(checker.go:18843)
    pub(crate) fn type_literal_accessor_member_type(
        &mut self,
        member: &Arc<Node>,
        name: &str,
    ) -> Arc<Type> {
        let getter = Self::accessor_sibling(member, SyntaxKind::GetAccessor, name);
        let setter = Self::accessor_sibling(member, SyntaxKind::SetAccessor, name);
        if let Some(g) = getter
            && let NodeData::GetAccessorDeclaration(gd) = &g.data
            && let Some(tn) = &gd.type_node
        {
            return self.guarded_type_literal_accessor_annotation(tn, &g, name);
        }
        if let Some(s) = setter
            && let NodeData::SetAccessorDeclaration(sd) = &s.data
            && let Some(param) = sd.parameters.iter().next()
            && let NodeData::ParameterDeclaration(pd) = &param.data
            && let Some(tn) = &pd.type_node
        {
            return self.guarded_type_literal_accessor_annotation(tn, &s, name);
        }
        self.get_any_type()
    }

    /// Go getTypeOfAccessors 的 popTypeResolution false 分支:注解经容器在途
    /// 环回落后为 error 时报 TS2502 于 accessor 名;裸标识符 typeof(Go 惰性
    /// 成员型可取得容器真型)不视为环,静默 any
    fn guarded_type_literal_accessor_annotation(
        &mut self,
        annotation: &Arc<Node>,
        accessor: &Arc<Node>,
        name: &str,
    ) -> Arc<Type> {
        let t = self.get_type_from_type_node(annotation);
        let cyclic = match &annotation.data {
            NodeData::TypeQueryNode(d) => {
                d.expr_name.kind != SyntaxKind::Identifier
                    && crate::checker::utilities::is_type_error(&t)
                    && self.type_query_root_in_flight(&d.expr_name)
            }
            NodeData::IndexedAccessTypeNode(ad) => {
                self.indexed_access_root_in_flight(&ad.object_type)
            }
            _ => false,
        };
        if cyclic {
            if let Some(n) = accessor.name() {
                let file = self
                    .get_source_file_of_node(accessor)
                    .or_else(|| self.current_file.clone());
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    n.loc,
                    tsox_core::diagnostics::messages_generated::
                        X_0_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ITS_OWN_TYPE_ANNOTATION,
                    vec![name.to_string()],
                ));
            }
            return self.get_any_type();
        }
        if crate::checker::utilities::is_type_error(&t) {
            return self.get_any_type();
        }
        t
    }

    /// 索引访问对象位根 TypeReference 指向在途构造中的别名(声明型解析在途或
    /// type_node 缓存为 error 预置)判定
    fn indexed_access_root_in_flight(&self, object_type: &Arc<Node>) -> bool {
        let NodeData::TypeReferenceNode(tr) = &object_type.data else {
            return false;
        };
        if tr.type_name.kind != SyntaxKind::Identifier {
            return false;
        }
        let Some(sym) = self.resolve_identifier(&tr.type_name) else {
            return false;
        };
        if sym
            .declarations
            .iter()
            .any(|d| matches!(d.data, NodeData::TypeAliasDeclaration(_)))
        {
            let key = Arc::as_ptr(&sym) as *const tsox_frontend::ast::Symbol;
            if self.is_resolving(
                key,
                crate::checker::checker::TypeResolutionProperty::DeclaredType,
            ) {
                return true;
            }
        }
        let Some(decl) = sym.value_declaration.as_ref() else {
            return false;
        };
        let NodeData::TypeAliasDeclaration(d) = &decl.data else {
            return false;
        };
        self.get_cached_type(&d.type_node)
            .is_some_and(|t| crate::checker::utilities::is_type_error(&t))
    }
}
