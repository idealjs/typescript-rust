use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::{
    ModifierFlags, Node, NodeList, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::diagnostics::messages_generated::*;

use crate::checker::types::*;






use super::*;


impl Checker {
    pub(crate) fn check_class_member(&mut self, node: &Arc<Node>) {

        self.check_grammar_modifiers(node);

        if node.kind == SyntaxKind::Constructor {
            self.check_multiple_constructor_implementations(node);
        }

        {
            let class_node = node.parent.clone();
            if let Some(cls) = &class_node
                && matches!(cls.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression)
                && let crate::ast::NodeData::ClassDeclaration(cd) = &cls.data
            {
                let (my_name, my_loc) = match &node.data {
                    crate::ast::NodeData::PropertyDeclaration(d) => {
                        (d.name.text().to_string(), d.name.loc)
                    }
                    crate::ast::NodeData::MethodDeclaration(d) => {
                        (d.name.text().to_string(), d.name.loc)
                    }
                    _ => (String::new(), node.loc),
                };
                if !my_name.is_empty() && my_name.starts_with('#') {

                    let i_am_static = node.has_syntactic_modifier(ModifierFlags::Static);
                    let conflict = cd.members.iter().any(|m| {
                        if m.loc.pos() >= node.loc.pos() {
                            return false;
                        }
                        let Some(mn) = m.name() else { return false };
                        mn.kind == SyntaxKind::PrivateIdentifier
                            && mn.text() == my_name
                            && m.has_syntactic_modifier(ModifierFlags::Static) != i_am_static
                    });
                    if conflict {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            my_loc,
                            crate::diagnostics::messages_generated::
                                DUPLICATE_IDENTIFIER_0_STATIC_AND_INSTANCE_ELEMENTS_CANNOT_SHARE_THE_SAME_PRIVATE_NAME,
                            vec![my_name.clone()],
                        ));
                    }
                }
                if !my_name.is_empty() && !my_name.starts_with('#') {

                    let kind_of = |m: &Arc<Node>| match &m.data {
                        crate::ast::NodeData::PropertyDeclaration(_) => "prop",
                        crate::ast::NodeData::MethodDeclaration(d) => {
                            if d.body.is_some() { "method-body" } else { "method-sig" }
                        }
                        crate::ast::NodeData::GetAccessorDeclaration(_) => "get",
                        crate::ast::NodeData::SetAccessorDeclaration(_) => "set",
                        _ => "",
                    };
                    let mine = kind_of(node);
                    let mut theirs_all_prop = true;
                    let dup = cd.members.iter().any(|m| {
                        if Arc::ptr_eq(m, node) || m.loc.pos() >= node.loc.pos() {
                            return false;
                        }
                        let name_match = match &m.data {
                            crate::ast::NodeData::PropertyDeclaration(d) => {
                                d.name.text() == my_name
                            }
                            crate::ast::NodeData::MethodDeclaration(d) => {
                                d.name.text() == my_name
                            }
                            _ => false,
                        };
                        if !name_match {
                            return false;
                        }
                        let theirs = kind_of(m);
                        if theirs != "prop" {
                            theirs_all_prop = false;
                        }
                        match (mine, theirs) {
                            ("prop", "prop") => true,
                            ("prop", "method-body") | ("prop", "method-sig") => true,
                            ("method-body", "prop") | ("method-sig", "prop") => true,

                            ("method-body", "method-body") => true,
                            _ => false,
                        }
                    });

                    let earlier_has_prop = cd.members.iter().any(|m| {
                        m.loc.pos() < node.loc.pos()
                            && matches!(&m.data, crate::ast::NodeData::PropertyDeclaration(d) if d.name.text() == my_name)
                    });

                    let earlier_has_method = cd.members.iter().any(|m| {
                        m.loc.pos() < node.loc.pos()
                            && matches!(&m.data, crate::ast::NodeData::MethodDeclaration(d) if d.name.text() == my_name)
                    });
                    let report_here = match mine {
                        "prop" => true,
                        "method-body" | "method-sig" => earlier_has_prop,
                        _ => false,
                    };
                    let _ = earlier_has_method;
                    if dup && report_here {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            my_loc,
                            crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                            vec![my_name.clone()],
                        ));
                    }

                    if dup && mine == "prop" && !report_here {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            my_loc,
                            crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                            vec![my_name.clone()],
                        ));
                    }

                    if dup && matches!(mine, "method-body" | "method-sig") && theirs_all_prop {
                        if let Some(earlier) = cd.members.iter().find(|m| {
                            m.loc.pos() < node.loc.pos()
                                && matches!(&m.data, crate::ast::NodeData::PropertyDeclaration(d) if d.name.text() == my_name)
                        }) {
                            let earlier_loc = earlier
                                .name()
                                .map(|n| n.loc)
                                .unwrap_or(earlier.loc);
                            let already = self
                                .diagnostics
                                .get_all()
                                .iter()
                                .any(|d| d.code == 2300 && d.loc == earlier_loc);
                            if !already {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    earlier_loc,
                                    crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                                    vec![my_name.clone()],
                                ));
                            }
                        }
                    }

                    if dup && mine == "prop" {
                        let first = cd.members.iter().find(|m| {
                            m.loc.pos() < node.loc.pos()
                                && match &m.data {
                                    crate::ast::NodeData::PropertyDeclaration(d) => {
                                        d.name.text() == my_name
                                    }
                                    crate::ast::NodeData::MethodDeclaration(d) => {
                                        d.name.text() == my_name
                                    }
                                    _ => false,
                                }
                        });
                        if let Some(first) = first {
                            let first_type = match &first.data {
                                crate::ast::NodeData::PropertyDeclaration(d) => {
                                    let tn = d.type_node.clone();
                                    tn.map(|tn| {
                                        let t = self.get_type_from_type_node(&tn);
                                        self.type_to_string(&t)
                                    })
                                }
                                _ => None,
                            };
                            let first_sig = match &first.data {
                                crate::ast::NodeData::MethodDeclaration(d) => {
                                    let ret = d
                                        .type_node
                                        .as_ref()
                                        .map(|tn| self.get_type_from_type_node(tn))
                                        .unwrap_or_else(|| self.any_type());
                                    let sig = self
                                        .build_signature_from_function_like_type_node(
                                            &d.parameters,
                                            ret,
                                            false,
                                            None,
                                            Some(Arc::clone(first)),
                                        );
                                    Some(self.type_to_string(
                                        &self.create_function_or_constructor_type(
                                            vec![sig],
                                            false,
                                        ),
                                    ))
                                }
                                _ => None,
                            };
                            let later_type = match &node.data {
                                crate::ast::NodeData::PropertyDeclaration(d) => {
                                    let tn = d.type_node.clone();
                                    tn.map(|tn| {
                                        let t = self.get_type_from_type_node(&tn);
                                        self.type_to_string(&t)
                                    })
                                }
                                _ => None,
                            };
                            if let (Some(f), Some(l)) = (first_type.or(first_sig), later_type) {
                                if f != l {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        self.current_file.clone(),
                                        my_loc,
                                        crate::diagnostics::messages_generated::
                                            SUBSEQUENT_PROPERTY_DECLARATIONS_MUST_HAVE_THE_SAME_TYPE_PROPERTY_0_MUST_BE_OF_TYPE_1_BUT_HERE_HAS_TYPE_2,
                                        vec![my_name.clone(), f, l],
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        match node.kind {
            SyntaxKind::PropertyDeclaration => {

                if let crate::ast::NodeData::PropertyDeclaration(data) = &node.data {

                    self.check_computed_property_name(&data.name);

                    if node.has_syntactic_modifier(ModifierFlags::Abstract)
                        && data.initializer.is_some()
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            data.name.loc,
                            crate::diagnostics::messages_generated::
                                PROPERTY_0_CANNOT_HAVE_AN_INITIALIZER_BECAUSE_IT_IS_MARKED_ABSTRACT,
                            vec![data.name.text().to_string()],
                        ));
                    }

                    if node.has_syntactic_modifier(ModifierFlags::Static) {
                        if let Some(type_node) = &data.type_node {
                            let prev = self.in_static_member_type;
                            self.in_static_member_type = true;
                            let _ = self.get_type_from_type_node(type_node);
                            self.in_static_member_type = prev;
                        }
                    }
                    if let Some(init) = &data.initializer {

                        let is_static = node.has_syntactic_modifier(ModifierFlags::Static);
                        self.this_container_stack.push(if is_static {
                            ThisContainerKind::StaticMember
                        } else {
                            ThisContainerKind::InstanceMember
                        });
                        self.check_expression(init);
                        self.this_container_stack.pop();

                        if let Some(tn) = &data.type_node {
                            let target = self.get_type_from_type_node(tn);
                            let anchor = data.name.loc;
                            self.check_contextual_elements(init, &target, anchor);
                        }
                    }
                }
            }
            SyntaxKind::PropertySignature => {

                if let crate::ast::NodeData::PropertySignatureDeclaration(data) = &node.data {
                    self.check_computed_property_name(&data.name);
                }
            }
            SyntaxKind::ClassStaticBlockDeclaration => {
                if let crate::ast::NodeData::ClassStaticBlockDeclaration(data) = &node.data {

                    self.this_container_stack
                        .push(ThisContainerKind::StaticMember);
                    self.check_statement(&data.body);
                    self.this_container_stack.pop();
                }
            }
            SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {

                if node.kind != SyntaxKind::Constructor
                    && let Some(name) = Self::member_name_node(node)
                {
                    self.check_computed_property_name(&name);
                }

                let (body, type_node, parameters): (
                    Option<Arc<Node>>,
                    Option<Arc<Node>>,
                    Option<Arc<NodeList>>,
                ) = match &node.data {
                    crate::ast::NodeData::MethodDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone(), Some(Arc::clone(&d.parameters)))
                    }
                    crate::ast::NodeData::ConstructorDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone(), Some(Arc::clone(&d.parameters)))
                    }
                    crate::ast::NodeData::GetAccessorDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone(), Some(Arc::clone(&d.parameters)))
                    }
                    crate::ast::NodeData::SetAccessorDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone(), Some(Arc::clone(&d.parameters)))
                    }
                    _ => (None, None, None),
                };

                if body.is_some()
                    && (self
                        .enclosing_class_stack
                        .last()
                        .is_some_and(|c| c.has_syntactic_modifier(ModifierFlags::Ambient))
                        || self.ambient_context_depth > 0
                        || self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file))
                    && let Some(body) = &body
                {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        crate::core::text::TextRange::new(body.loc.pos(), body.loc.pos() + 1),
                        crate::diagnostics::messages_generated::
                            AN_IMPLEMENTATION_CANNOT_BE_DECLARED_IN_AMBIENT_CONTEXTS,
                        vec![],
                    ));
                }

                if matches!(node.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor) {
                    let ambient = self
                        .enclosing_class_stack
                        .last()
                        .is_some_and(|c| c.has_syntactic_modifier(ModifierFlags::Ambient))
                        || self.ambient_context_depth > 0
                        || self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file);
                    let is_abstract = node.has_syntactic_modifier(ModifierFlags::Abstract);
                    if node.kind == SyntaxKind::SetAccessor
                        && let Some(params) = &parameters
                        && let Some(first) = params.iter().next()
                        && let crate::ast::NodeData::ParameterDeclaration(pd) = &first.data
                    {
                        if let Some(rest) = &pd.dot_dot_dot_token {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                rest.loc,
                                crate::diagnostics::messages_generated::
                                    A_SET_ACCESSOR_CANNOT_HAVE_REST_PARAMETER,
                                vec![],
                            ));
                        }
                        if let Some(question) = &pd.question_token {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                question.loc,
                                crate::diagnostics::messages_generated::
                                    A_SET_ACCESSOR_CANNOT_HAVE_AN_OPTIONAL_PARAMETER,
                                vec![],
                            ));
                        }
                        if pd.initializer.is_some() {
                            let name_loc = Self::class_member_name_node(node)
                                .map(|n| n.loc)
                                .unwrap_or(node.loc);
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                crate::diagnostics::messages_generated::
                                    A_SET_ACCESSOR_PARAMETER_CANNOT_HAVE_AN_INITIALIZER,
                                vec![],
                            ));
                        }
                    }
                    if body.is_none() && !ambient && !is_abstract && node.loc.end() > 0 {

                        let file = self.current_file.clone();
                        let mut p = node.loc.end();
                        if let Some(f) = file.as_ref() {
                            while p > node.loc.pos()
                                && matches!(
                                    f.text.as_bytes()[p - 1],
                                    b'\r' | b'\n' | b' ' | b'\t'
                                )
                            {
                                p -= 1;
                            }
                        }
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            crate::core::text::TextRange::new(p - 1, p),
                            crate::diagnostics::messages_generated::X_0_EXPECTED,
                            vec!["{".to_string()],
                        ));
                    }

                    if body.is_some() && is_abstract {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            node.loc,
                            crate::diagnostics::messages_generated::
                                AN_ABSTRACT_ACCESSOR_CANNOT_HAVE_AN_IMPLEMENTATION,
                            vec![],
                        ));
                    }

                    if node.kind == SyntaxKind::GetAccessor
                        && let Some(class) = self.enclosing_class_stack.last().cloned()
                        && let crate::ast::NodeData::GetAccessorDeclaration(gd) = &node.data
                        && gd.name.kind == SyntaxKind::Identifier
                    {
                        let setter = Self::class_members_of(&class).iter().find_map(|m| {
                            if let crate::ast::NodeData::SetAccessorDeclaration(sd) = &m.data
                                && sd.name.kind == SyntaxKind::Identifier
                                && sd.name.text() == gd.name.text()
                            {
                                Some((Arc::clone(m), sd.name.loc))
                            } else {
                                None
                            }
                        });
                        if let Some((setter_node, setter_name_loc)) = setter {
                            let getter_abstract = is_abstract;
                            let setter_abstract =
                                setter_node.has_syntactic_modifier(ModifierFlags::Abstract);
                            if getter_abstract != setter_abstract {
                                let file = self.current_file.clone();
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file.clone(),
                                    gd.name.loc,
                                    crate::diagnostics::messages_generated::
                                        ACCESSORS_MUST_BOTH_BE_ABSTRACT_OR_NON_ABSTRACT,
                                    vec![],
                                ));
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    setter_name_loc,
                                    crate::diagnostics::messages_generated::
                                        ACCESSORS_MUST_BOTH_BE_ABSTRACT_OR_NON_ABSTRACT,
                                    vec![],
                                ));
                            }

                            let setter_param_type_node =
                                if let crate::ast::NodeData::SetAccessorDeclaration(sd) =
                                    &setter_node.data
                                {
                                    sd.parameters.iter().next().and_then(|p| {
                                        if let crate::ast::NodeData::ParameterDeclaration(pd) =
                                            &p.data
                                        {
                                            pd.type_node.clone()
                                        } else {
                                            None
                                        }
                                    })
                                } else {
                                    None
                                };
                            if gd.type_node.is_none() && let Some(setter_tn) = setter_param_type_node
                            {
                                self.accessor_pair_return_hint =
                                    Some(self.get_type_from_type_node(&setter_tn));
                            }
                        }
                    }

                        if node.kind == SyntaxKind::SetAccessor
                            && let Some(class) = self.enclosing_class_stack.last().cloned()
                            && let crate::ast::NodeData::SetAccessorDeclaration(sd) = &node.data
                            && sd.name.kind == SyntaxKind::Identifier
                            && let Some(param) = sd.parameters.iter().next()
                            && let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data
                            && pd.type_node.is_none()
                            && let Some(param_name) = (if pd.name.kind == SyntaxKind::Identifier {
                                Some(pd.name.text().to_string())
                            } else {
                                None
                            })
                        {
                            let getter_type = Self::class_members_of(&class)
                                .iter()
                                .find_map(|m| {
                                    if let crate::ast::NodeData::GetAccessorDeclaration(gd) =
                                        &m.data
                                        && gd.name.kind == SyntaxKind::Identifier
                                        && gd.name.text() == sd.name.text()
                                        && let Some(tn) = &gd.type_node
                                    {
                                        Some(self.get_type_from_type_node(tn))
                                    } else {
                                        None
                                    }
                                });
                            if let (Some(expected), Some(body)) = (getter_type, &sd.body) {
                                for (lhs_loc, rhs) in
                                    Self::assignments_to_name(body, &param_name)
                                {
                                    let actual = self.get_type_of_node(&rhs);
                                    if !actual.flags.contains(TypeFlags::Any)
                                        && !self.is_type_assignable_to(&actual, &expected)
                                    {
                                        let display_type =
                                            if crate::checker::is_literal_type(&actual) {
                                                self.get_base_type_of_literal_type(&actual)
                                            } else {
                                                actual.clone()
                                            };
                                        let actual_str = self.type_to_string(&display_type);
                                        let expected_str = self.type_to_string(&expected);
                                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                            self.current_file.clone(),
                                            lhs_loc,
                                            TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                            vec![actual_str, expected_str],
                                        ));
                                    }
                                }
                            }
                        }

                    if !(body.is_none() && !is_abstract && !ambient) {
                        let name_loc = Self::class_member_name_node(node)
                            .map(|n| n.loc)
                            .unwrap_or(node.loc);
                        fn first_param_is_this(params: &Arc<NodeList>) -> bool {
                            params.iter().next().is_some_and(|p| {
                                matches!(
                                    &p.data,
                                    crate::ast::NodeData::ParameterDeclaration(pd)
                                if pd.name.kind == SyntaxKind::Identifier
                    && pd.name.text() == "this")
                            })
                        }
                        let (has_type_params, params, set_has_return) = match &node.data {
                            crate::ast::NodeData::GetAccessorDeclaration(d) => (
                                d.type_parameters.is_some(),
                                Some(&d.parameters),
                                false,
                            ),
                            crate::ast::NodeData::SetAccessorDeclaration(d) => (
                                d.type_parameters.is_some(),
                                Some(&d.parameters),
                                d.type_node.is_some(),
                            ),
                            _ => (false, None, false),
                        };
                        let param_count =
                            params.map_or(0, |p| p.iter().count());
                        let first_is_this =
                            params.is_some_and(first_param_is_this);
                        let expected = if node.kind == SyntaxKind::GetAccessor {
                            0
                        } else {
                            1
                        };
                        let count_correct = param_count == expected
                            || (first_is_this && param_count == expected + 1);
                        let file = self.current_file.clone();
                        if has_type_params {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                crate::diagnostics::messages_generated::
                                    AN_ACCESSOR_CANNOT_HAVE_TYPE_PARAMETERS,
                                vec![],
                            ));
                        } else if !count_correct {
                            let message = if node.kind == SyntaxKind::GetAccessor {
                                crate::diagnostics::messages_generated::
                                    A_GET_ACCESSOR_CANNOT_HAVE_PARAMETERS
                            } else {
                                crate::diagnostics::messages_generated::
                                    A_SET_ACCESSOR_MUST_HAVE_EXACTLY_ONE_PARAMETER
                            };
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                message,
                                vec![],
                            ));
                        } else if node.kind == SyntaxKind::SetAccessor && set_has_return {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                crate::diagnostics::messages_generated::
                                    A_SET_ACCESSOR_CANNOT_HAVE_A_RETURN_TYPE_ANNOTATION,
                                vec![],
                            ));
                        }

                        if node.kind == SyntaxKind::GetAccessor
                            && !ambient
                            && let Some(body_node) = &body
                            && !self.function_body_definitely_returns(body_node)
                            && !Self::function_body_has_explicit_return(body_node)
                        {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                crate::diagnostics::messages_generated::
                                    A_GET_ACCESSOR_MUST_RETURN_A_VALUE,
                                vec![],
                            ));
                        }
                    }
                }

                if let Some(params) = &parameters {
                    let is_ctor_impl =
                        matches!(node.kind, SyntaxKind::Constructor) && body.is_some();
                    self.check_parameter_property_modifiers(params, is_ctor_impl);

                    if matches!(node.kind, SyntaxKind::MethodDeclaration | SyntaxKind::Constructor)
                    {
                        self.check_parameter_implicit_any(node, params, 0);
                    }
                    for p in params.iter() {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);

                            if matches!(
                                node.kind,
                                SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                            ) {
                                let _ = self.get_type_from_type_node(pt);
                            }
                        }
                    }
                }
                if let Some(tn) = &type_node {
                    self.check_type_annotation(tn);
                }

                if self.no_implicit_any
                    && matches!(node.kind, SyntaxKind::MethodDeclaration)
                    && type_node.is_none()
                    && body.is_none()
                {
                    if let Some(name) = Self::class_member_name_node(node) {
                        if name.kind == SyntaxKind::Identifier {
                            let file = self.current_file.clone();
                            let diagnostic = crate::ast::Diagnostic::new(
                                file,
                                name.loc,
                                crate::diagnostics::messages_generated::
                                    X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                                vec![name.text().to_string(), "any".to_string()],
                            );
                            self.diagnostics.add(diagnostic);
                        }
                    }
                }
                if let Some(body) = body {

                    if node.kind == SyntaxKind::Constructor
                        && self
                            .enclosing_class_stack
                            .last()
                            .is_some_and(|c| self.extends_base_of(c).is_some())
                    {
                        self.check_super_before_this(&body);
                    }

                    let is_static = node.has_syntactic_modifier(ModifierFlags::Static);
                    self.this_container_stack.push(if is_static {
                        ThisContainerKind::StaticMember
                    } else {
                        ThisContainerKind::InstanceMember
                    });
                    self.push_function_scope(node);

                    self.in_ctor_body_stack
                        .push(node.kind == SyntaxKind::Constructor);

                    let declared_return = if node.kind == SyntaxKind::GetAccessor
                        && type_node.is_none()
                        && let Some(hint) = self.accessor_pair_return_hint.take()
                    {
                        Some(hint)
                    } else {
                        let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
                        type_node
                            .as_ref()
                            .map(|tn| self.get_type_from_type_node(tn))
                            .map(|t| self.unwrap_async_return_type(t, is_async))
                    };
                    self.return_type_stack.push(declared_return.clone());
                    match body.kind {
                        SyntaxKind::Block => self.check_statement(&body),
                        _ => self.check_expression(&body),
                    }
                    self.return_type_stack.pop();
                    self.in_ctor_body_stack.pop();
                    self.pop_function_scope();
                    self.this_container_stack.pop();

                    if let Some(ret_type) = &declared_return
                        && !ret_type.flags.contains(TypeFlags::Void)
                        && !ret_type.flags.contains(TypeFlags::Undefined)
                        && !ret_type.flags.contains(TypeFlags::Any)
                        && body.kind == SyntaxKind::Block
                        && !self.function_body_definitely_returns(&body)
                    {
                        let loc = type_node
                            .as_ref()
                            .map_or(node.loc, |tn| tn.loc);
                        if matches!(node.kind, SyntaxKind::MethodDeclaration) {
                            if Self::function_body_has_explicit_return(&body) {

                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    loc,
                                    FUNCTION_LACKS_ENDING_RETURN_STATEMENT_AND_RETURN_TYPE_DOES_NOT_INCLUDE_UNDEFINED,
                                    vec![],
                                ));
                            } else {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    loc,
                                    A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
                                    vec![],
                                ));
                            }
                        } else if node.kind == SyntaxKind::GetAccessor {

                            let tgt = self.type_to_string(ret_type);
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                loc,
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                vec!["undefined".to_string(), tgt],
                            ));
                        }
                    }
                }
            }
            _ => {

            }
        }
    }
    pub(crate) fn resolve_base_class_constructor_type(&mut self) -> Option<Arc<Type>> {
        let (base_node, symbol) = self.base_class_node_of_enclosing_class()?;

        let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
        if !self.resolving_type_aliases.insert(key) {
            return None;
        }
        let ctor_type = self.get_type_of_class_declaration(&base_node);
        self.resolving_type_aliases.remove(&key);
        Some(ctor_type)
    }

    fn base_class_node_of_enclosing_class(&self) -> Option<(Arc<Node>, Arc<Symbol>)> {
        let class_node = self.enclosing_class_stack.last().cloned()?;
        self.extends_base_of(&class_node)
    }

    pub(crate) fn resolve_base_class_instance_type(&mut self, type_ref: &Arc<Node>) -> Arc<Type> {

        if let crate::ast::NodeData::ExpressionWithTypeArguments(data) = &type_ref.data {
            if data.expression.kind == SyntaxKind::Identifier {
                if let Some(symbol) = self.resolve_identifier(&data.expression) {
                    if symbol.flags.contains(SymbolFlags::Class) {

                        if self.type_resolution_stack.len() >= 200 {
                            return self.get_any_type();
                        }

                        if let Some(class_node) = symbol
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                            .cloned()
                        {

                            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
                            if !self.push_type_resolution(
                                key,
                                TypeResolutionProperty::ResolvedBaseTypes,
                            ) {
                                return self.get_any_type();
                            }

                            let heritage_args = data.type_arguments.clone();
                            let base_tps: Vec<Arc<crate::ast::Symbol>> = match &class_node.data {
                                crate::ast::NodeData::ClassDeclaration(cd) => {
                                    match &cd.type_parameters {
                                        Some(tps) => tps
                                            .iter()
                                            .filter_map(|tp| {
                                                self.program
                                                    .symbol_map()
                                                    .symbol_of(tp)
                                                    .map(Arc::clone)
                                            })
                                            .collect(),
                                        None => Vec::new(),
                                    }
                                }
                                _ => Vec::new(),
                            };
                            let pushed = if let Some(args) = &heritage_args
                                && !base_tps.is_empty()
                            {
                                let arg_types: Vec<Arc<Type>> = args
                                    .iter()
                                    .map(|a| self.get_type_from_type_node(a))
                                    .collect();
                                let mut mapping = HashMap::new();
                                let mut name_frame: Vec<(Arc<Symbol>, Arc<Type>)> = Vec::new();
                                for (i, tp_sym) in base_tps.iter().enumerate() {
                                    if i < arg_types.len() {
                                        mapping.insert(
                                            Arc::as_ptr(tp_sym) as *const crate::ast::Symbol,
                                            Arc::clone(&arg_types[i]),
                                        );
                                        name_frame
                                            .push((Arc::clone(tp_sym), Arc::clone(&arg_types[i])));
                                    }
                                }
                                self.type_argument_stack.push(mapping);
                                self.type_argument_name_frames.push(name_frame);
                                true
                            } else {
                                false
                            };
                            let instance = {

                                self.push_scope(&class_node);
                                let i = self.build_class_instance_type_with_base(&class_node);
                                self.pop_scope();
                                i
                            };
                            if pushed {
                                self.type_argument_stack.pop();
                                self.type_argument_name_frames.pop();
                            }
                            self.pop_type_resolution();
                            return instance;
                        }
                    }
                }
            }
        }

        let t = self.get_type_from_type_node(type_ref);
        if t.flags.contains(TypeFlags::Any) {
            return self.get_any_type();
        }

        if t.flags.contains(TypeFlags::Object) {
            return t;
        }
        self.get_any_type()
    }

    pub(crate) fn merge_instance_types(&mut self, derived: &Arc<Type>, base: &Arc<Type>) -> Arc<Type> {
        if base.flags.contains(TypeFlags::Any) {
            return Arc::clone(derived);
        }
        let derived_data = match &derived.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };
        let base_data = match &base.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };

        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();

        for prop in &derived_data.properties {
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }

        for prop in &base_data.properties {
            if symbol_table.get(&prop.name).is_some() {
                continue;
            }
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }

        let mut index_infos = derived_data.index_infos.clone();
        index_infos.extend(base_data.index_infos.iter().cloned());

        let mut call_signatures: Vec<Arc<Signature>> =
            derived_data.call_signatures().to_vec();
        let derived_call_count = call_signatures.len();
        call_signatures.extend(base_data.call_signatures().iter().cloned());
        let mut signatures = call_signatures;
        signatures.extend(derived_data.construct_signatures().iter().cloned());
        signatures.extend(base_data.construct_signatures().iter().cloned());
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,

            symbol: derived.symbol.clone(),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count: derived_call_count
                        + base_data.call_signatures().len(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    fn get_type_from_heritage_type_reference(&mut self, type_ref: &Arc<Node>) -> Arc<Type> {
        self.get_type_from_type_node(type_ref)
    }

    pub(crate) fn check_property_initialization(&mut self, class_node: &Arc<Node>) {
        if !self.strict_null_checks || !self.strict_property_initialization {
            return;
        }

        if class_node.has_syntactic_modifier(ModifierFlags::Ambient)
            || self.ambient_context_depth > 0
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file)
        {
            return;
        }
        let members = match &class_node.data {
            crate::ast::NodeData::ClassDeclaration(d) => &d.members,
            _ => return,
        };

        let constructor = members.iter().find(|m| m.kind == SyntaxKind::Constructor);
        for member in members.iter() {
            if member.kind != SyntaxKind::PropertyDeclaration {
                continue;
            }

            let mods = self.get_combined_modifier_flags(member);
            if mods.contains(ModifierFlags::Ambient) || mods.contains(ModifierFlags::Static) {
                continue;
            }

            if mods.contains(ModifierFlags::Abstract) {
                continue;
            }
            let crate::ast::NodeData::PropertyDeclaration(pd) = &member.data else {
                continue;
            };

            if pd.initializer.is_some() || pd.postfix_token.is_some() {
                continue;
            }

            let name_node = &pd.name;
            if !matches!(
                name_node.kind,
                SyntaxKind::Identifier
                    | SyntaxKind::PrivateIdentifier
                    | SyntaxKind::ComputedPropertyName
            ) {
                continue;
            }

            let Some(type_node) = &pd.type_node else {
                continue;
            };
            let prop_type = self.get_type_from_type_node(type_node);
            if prop_type
                .flags
                .intersects(TYPE_FLAGS_ANY_OR_UNKNOWN | TypeFlags::Undefined)
                || type_contains_undefined(&prop_type)
            {
                continue;
            }

            if let Some(ctor) = constructor {
                if self.is_property_assigned_in_constructor(name_node, ctor) {
                    continue;
                }
            }

            let name_text = self.node_text(name_node);
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name_node.loc,
                PROPERTY_0_HAS_NO_INITIALIZER_AND_IS_NOT_DEFINITELY_ASSIGNED_IN_THE_CONSTRUCTOR,
                vec![name_text],
            ));
        }
    }

    fn node_text(&self, node: &Arc<Node>) -> String {
        match &node.data {
            crate::ast::NodeData::Identifier(d) => d.text.clone(),
            crate::ast::NodeData::PrivateIdentifier(d) => d.text.clone(),
            crate::ast::NodeData::ComputedPropertyName(_) => {

                let Some(file) = &self.current_file else {
                    return String::new();
                };
                let pos = node.loc.pos();
                let end = node.loc.end();
                if pos < end && end <= file.text.len() {
                    file.text[pos..end].to_string()
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        }
    }

    #[allow(dead_code)]
    fn resolve_property_name(
        &mut self,
        _member: &Arc<Node>,
        name: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {

        self.resolve_identifier(name)
    }

    fn is_property_assigned_in_constructor(&self, name_node: &Arc<Node>, ctor: &Arc<Node>) -> bool {
        let name_text = match &name_node.data {
            crate::ast::NodeData::Identifier(d) => d.text.as_str(),
            _ => return false,
        };

        let body = match &ctor.data {
            crate::ast::NodeData::ConstructorDeclaration(d) => &d.body,
            _ => return false,
        };
        let Some(body) = body else {
            return false;
        };
        Self::node_contains_this_assignment(body, name_text)
    }

    fn node_contains_this_assignment(node: &Arc<Node>, name: &str) -> bool {

        if let crate::ast::NodeData::BinaryExpression(data) = &node.data {
            if data.operator_token.kind == SyntaxKind::EqualsToken {
                if Self::is_this_property_access(&data.left, name) {
                    return true;
                }
            }
        }

        let mut found = false;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            if Self::node_contains_this_assignment(child, name) {
                found = true;
                return true;
            }
            false
        });
        found
    }

    fn is_this_property_access(node: &Arc<Node>, name: &str) -> bool {
        match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => {
                if data.expression.kind == SyntaxKind::ThisKeyword {
                    if let crate::ast::NodeData::Identifier(id) = &data.name.data {
                        return id.text == name;
                    }
                }
                false
            }
            crate::ast::NodeData::ElementAccessExpression(data) => {
                if data.expression.kind == SyntaxKind::ThisKeyword {
                    if let crate::ast::NodeData::StringLiteral(sl) = &data.argument_expression.data
                    {
                        return sl.text == name;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn class_member_name_node(node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            crate::ast::NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::GetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::SetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            _ => None,
        }
    }

    fn class_member_name_text(node: &Arc<Node>) -> Option<String> {
        if matches!(node.kind, SyntaxKind::Constructor) {
            return Some("constructor".to_string());
        }
        let name = Self::class_member_name_node(node)?;
        match name.kind {

            SyntaxKind::Identifier | SyntaxKind::NumericLiteral => {
                let text = name.text().to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
            SyntaxKind::StringLiteral => Some(format!("\"{}\"", name.text())),
            _ => None,
        }
    }

    fn class_member_has_body(node: &Arc<Node>) -> bool {
        matches!(
            &node.data,
            crate::ast::NodeData::MethodDeclaration(d) if d.body.is_some()
        ) || matches!(
            &node.data,
            crate::ast::NodeData::ConstructorDeclaration(d) if d.body.is_some()
        )
    }

    fn function_like_params_and_return(
        node: &Arc<Node>,
    ) -> Option<(&Arc<NodeList>, Option<&Arc<Node>>)> {
        match &node.data {
            crate::ast::NodeData::FunctionDeclaration(d) => {
                Some((&d.parameters, d.type_node.as_ref()))
            }
            crate::ast::NodeData::MethodDeclaration(d) => {
                Some((&d.parameters, d.type_node.as_ref()))
            }
            crate::ast::NodeData::ConstructorDeclaration(d) => Some((&d.parameters, None)),
            _ => None,
        }
    }

    pub(crate) fn overload_signature_compatible_with_implementation(
        &mut self,
        overload: &Arc<Node>,
        implementation: &Arc<Node>,
    ) -> bool {
        let Some((ov_params, ov_return)) = Self::function_like_params_and_return(overload)
            .map(|(p, r)| (Arc::clone(p), r.cloned()))
        else {
            return true;
        };
        let Some((im_params, im_return)) = Self::function_like_params_and_return(implementation)
            .map(|(p, r)| (Arc::clone(p), r.cloned()))
        else {
            return true;
        };

        let return_ok = match (ov_return, im_return) {
            (Some(ovn), Some(imn)) => {
                let ov_t = self.get_type_from_type_node(&ovn);
                let im_t = self.get_type_from_type_node(&imn);
                ov_t.flags.contains(TypeFlags::Void)
                    || self.is_type_assignable_to(&ov_t, &im_t)
                    || self.is_type_assignable_to(&im_t, &ov_t)
            }
            _ => true,
        };
        if !return_ok {
            return false;
        }

        let n = ov_params.len().min(im_params.len());
        for i in 0..n {
            let ov_tn = match &ov_params.nodes[i].data {
                crate::ast::NodeData::ParameterDeclaration(p) => p.type_node.as_ref(),
                _ => None,
            };
            let im_tn = match &im_params.nodes[i].data {
                crate::ast::NodeData::ParameterDeclaration(p) => p.type_node.as_ref(),
                _ => None,
            };
            let (Some(o), Some(m)) = (ov_tn, im_tn) else {
                continue;
            };
            let ov_t = self.get_type_from_type_node(&o);
            let im_t = self.get_type_from_type_node(&m);
            if !self.is_type_assignable_to(&ov_t, &im_t)
                && !self.is_type_assignable_to(&im_t, &ov_t)
            {
                return false;
            }
        }
        true
    }

    pub(crate) fn check_class_member_overloads(&mut self, members: &NodeList) {

        let mut groups: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (idx, m) in members.iter().enumerate() {
            if !matches!(m.kind, SyntaxKind::Constructor | SyntaxKind::MethodDeclaration) {
                continue;
            }
            if let Some(name) = Self::class_member_name_text(m) {
                groups.entry(name).or_default().push(idx);
            }
        }
        for (_, idxs) in groups {

            let mut prev: Option<usize> = None;
            let mut has_body = false;
            for &idx in &idxs {
                let node = &members.nodes[idx];
                if !Self::class_member_has_body(node) {
                    if let Some(p) = prev {
                        if p + 1 != idx {
                            self.report_implementation_expected_error(members, p);
                        }
                    }
                } else {
                    has_body = true;
                }
                prev = Some(idx);
            }
            let last = idxs[idxs.len() - 1];
            if !has_body {
                let node = &members.nodes[last];
                let exempt = node.has_syntactic_modifier(ModifierFlags::Abstract)
                    || matches!(
                        &node.data,
                        crate::ast::NodeData::MethodDeclaration(d) if d.postfix_token.is_some()
                    );
                if !exempt {
                    self.report_implementation_expected_error(members, last);
                }
            } else {

                let impl_idx = idxs
                    .iter()
                    .copied()
                    .find(|&i| Self::class_member_has_body(&members.nodes[i]))
                    .unwrap_or(last);
                let impl_node = Arc::clone(&members.nodes[impl_idx]);
                for &i in &idxs {
                    if i == impl_idx {
                        continue;
                    }
                    let overload = Arc::clone(&members.nodes[i]);
                    if !self.overload_signature_compatible_with_implementation(&overload, &impl_node)
                        && let Some(name_node) = crate::ast::utilities::get_name_of_declaration(
                            &overload,
                        )
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            name_node.loc,
                            crate::diagnostics::messages_generated::
                                THIS_OVERLOAD_SIGNATURE_IS_NOT_COMPATIBLE_WITH_ITS_IMPLEMENTATION_SIGNATURE,
                            Vec::new(),
                        ));
                    }
                }
            }
        }
    }

    fn report_implementation_expected_error(&mut self, members: &NodeList, idx: usize) {
        let node = Arc::clone(&members.nodes[idx]);
        let name_text = Self::class_member_name_text(&node);
        if let Some(sib) = members.nodes.get(idx + 1) {
            if sib.kind == node.kind {
                let sib_name = Self::class_member_name_text(sib);
                let same_name = match (&name_text, &sib_name) {
                    (Some(a), Some(b)) => a == b,
                    _ => false,
                };

                if same_name {
                    return;
                }
                if Self::class_member_has_body(sib) {

                    let file = self.current_file.clone();
                    let loc = Self::class_member_name_node(sib)
                        .map(|n| n.loc)
                        .unwrap_or(sib.loc);
                    let display_name = name_text.unwrap_or_default();
                    let diagnostic = crate::ast::Diagnostic::new(
                        file,
                        loc,
                        crate::diagnostics::messages_generated::
                            FUNCTION_IMPLEMENTATION_NAME_MUST_BE_0,
                        vec![display_name],
                    );
                    self.diagnostics.add(diagnostic);
                    return;
                }
            }
        }

        let file = self.current_file.clone();
        let (loc, message): (crate::core::text::TextRange, crate::diagnostics::Message) =
            if matches!(node.kind, SyntaxKind::Constructor) {
                (
                    node.loc,
                    crate::diagnostics::messages_generated::CONSTRUCTOR_IMPLEMENTATION_IS_MISSING,
                )
            } else {
                (
                    Self::class_member_name_node(&node)
                        .map(|n| n.loc)
                        .unwrap_or(node.loc),
                    crate::diagnostics::messages_generated::
                        FUNCTION_IMPLEMENTATION_IS_MISSING_OR_NOT_IMMEDIATELY_FOLLOWING_THE_DECLARATION,
                )
            };
        let diagnostic = crate::ast::Diagnostic::new(file, loc, message, Vec::new());
        self.diagnostics.add(diagnostic);
    }

    pub(crate) fn check_parameter_property_modifiers(&mut self, params: &NodeList, is_ctor_impl: bool) {
        for param in params.iter() {
            let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };

            if pd.modifiers.is_some() {
                self.check_grammar_modifiers(param);
            }
            let Some(modifiers) = &pd.modifiers else { continue };
            if is_ctor_impl {
                continue;
            }
            if modifiers.modifier_flags.intersects(
                ModifierFlags::Public
                    | ModifierFlags::Private
                    | ModifierFlags::Protected
                    | ModifierFlags::Readonly,
            ) {
                let file = self.current_file.clone();
                let diagnostic = crate::ast::Diagnostic::new(
                    file,
                    param.loc,
                    crate::diagnostics::messages_generated::
                        A_PARAMETER_PROPERTY_IS_ONLY_ALLOWED_IN_A_CONSTRUCTOR_IMPLEMENTATION,
                    Vec::new(),
                );
                self.diagnostics.add(diagnostic);
            }
        }
    }

    pub(crate) fn check_parameter_implicit_any(
        &mut self,
        node: &Arc<Node>,
        params: &NodeList,
        contextual_param_count: usize,
    ) {
        if !self.no_implicit_any {
            return;
        }
        for (i, param) in params.iter().enumerate() {
            let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            if pd.type_node.is_some() || pd.initializer.is_some() {
                continue;
            }
            let name = &pd.name;
            if name.kind != SyntaxKind::Identifier || name.text() == "this" {
                continue;
            }

            if i < contextual_param_count {
                continue;
            }

            if self.param_has_typed_jsdoc_tag(node, name.text()) {
                continue;
            }
            let file = self.current_file.clone();
            let name_text = name.text().to_string();
            let diagnostic = if pd.dot_dot_dot_token.is_some() {
                crate::ast::Diagnostic::new(
                    file,
                    param.loc,
                    crate::diagnostics::messages_generated::
                        REST_PARAMETER_0_IMPLICITLY_HAS_AN_ANY_TYPE,
                    vec![name_text],
                )
            } else {
                crate::ast::Diagnostic::new(
                    file,
                    param.loc,
                    crate::diagnostics::messages_generated::PARAMETER_0_IMPLICITLY_HAS_AN_1_TYPE,
                    vec![name_text, "any".to_string()],
                )
            };
            self.diagnostics.add(diagnostic);
        }
    }

    fn param_has_typed_jsdoc_tag(&self, node: &Arc<Node>, param_name: &str) -> bool {
        let Some(file) = &self.current_file else {
            return false;
        };
        for jsdoc in file.resolve_jsdoc(node) {
            let crate::ast::NodeData::JSDoc(d) = &jsdoc.data else {
                continue;
            };
            let Some(tags) = &d.tags else { continue };
            for tag in tags.iter() {
                if let crate::ast::NodeData::JSDocParameterOrPropertyTag(td) = &tag.data
                    && td.name.kind == SyntaxKind::Identifier
                    && td.name.text() == param_name
                    && td.type_expression.is_some()
                {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn check_type_annotation(&mut self, tn: &Arc<Node>) {

        self.with_declaring_file_context(tn, |c| c.check_type_annotation_inner(tn));
    }

    fn check_type_annotation_inner(&mut self, tn: &Arc<Node>) {
        match tn.kind {
            SyntaxKind::FunctionType | SyntaxKind::ConstructorType => {
                let (params, return_type): (&NodeList, Option<&Arc<Node>>) = match &tn.data {
                    crate::ast::NodeData::FunctionTypeNode(d) => {
                        (&d.parameters, d.type_node.as_ref())
                    }
                    crate::ast::NodeData::ConstructorTypeNode(d) => {
                        (&d.parameters, d.type_node.as_ref())
                    }
                    _ => return,
                };
                self.check_parameter_property_modifiers(params, false);
                self.check_parameter_implicit_any(tn, params, 0);
                for p in params.iter() {
                    if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                        && let Some(pt) = &pd.type_node
                    {
                        self.check_type_annotation(pt);
                    }
                }
                if let Some(rt) = return_type {
                    self.check_type_annotation(rt);
                }
            }
            SyntaxKind::TypeReference => {
                if let crate::ast::NodeData::TypeReferenceNode(d) = &tn.data
                    && let Some(args) = &d.type_arguments
                {
                    for a in args.iter() {
                        self.check_type_annotation(a);
                    }
                }
            }
            SyntaxKind::UnionType | SyntaxKind::IntersectionType => {
                if let crate::ast::NodeData::UnionTypeNode(d) = &tn.data {
                    for t in d.types.iter() {
                        self.check_type_annotation(t);
                    }
                }
                if let crate::ast::NodeData::IntersectionTypeNode(d) = &tn.data {
                    for t in d.types.iter() {
                        self.check_type_annotation(t);
                    }
                }
            }
            SyntaxKind::ParenthesizedType => {
                if let crate::ast::NodeData::ParenthesizedTypeNode(d) = &tn.data {
                    self.check_type_annotation(&d.type_node);
                }
            }
            SyntaxKind::ArrayType | SyntaxKind::TypeOperator => {
                if let crate::ast::NodeData::ArrayTypeNode(d) = &tn.data {
                    self.check_type_annotation(&d.element_type);
                }
                if let crate::ast::NodeData::TypeOperatorNode(d) = &tn.data {
                    self.check_type_annotation(&d.type_node);
                }
            }
            SyntaxKind::TupleType => {
                if let crate::ast::NodeData::TupleTypeNode(d) = &tn.data {
                    for t in d.elements.iter() {
                        self.check_type_annotation(t);
                    }
                }
            }
            SyntaxKind::IndexedAccessType => {
                if let crate::ast::NodeData::IndexedAccessTypeNode(d) = &tn.data {
                    self.check_type_annotation(&d.object_type);
                    self.check_type_annotation(&d.index_type);

                    self.check_indexed_access_index_type(tn);
                }
            }
            SyntaxKind::TypeLiteral => {

                if let crate::ast::NodeData::TypeLiteralNode(d) = &tn.data {
                    for member in d.members.iter() {
                        if matches!(
                            member.kind,
                            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                        ) {
                            self.check_accessor_in_type_context(member);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_indexed_access_index_type(&mut self, node: &Arc<Node>) {
        use crate::checker::types::{TypeData, TypeFlags};
        let t = self.get_type_from_type_node(node);

        if !self.type_argument_stack.is_empty() {
            return;
        }

        if self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"))
        {
            return;
        }
        let (object_type, index_type) = match &t.data {
            TypeData::IndexedAccess(d) => match (&d.object_type, &d.index_type) {
                (Some(o), Some(i)) => (Arc::clone(o), Arc::clone(i)),
                _ => return,
            },
            _ => return,
        };

        if object_type
            .flags
            .intersects(TypeFlags::Any | TypeFlags::Unknown)
        {
            return;
        }

        if self.type_flags_is_generic_object_type(&object_type) {
            return;
        }

        let object_index_type = self.get_index_type(&object_type);
        let has_number_index_info = self
            .get_index_info_of_type(&object_type, &self.number_type())
            .is_some();

        let constituents: Vec<Arc<Type>> = if index_type.flags.contains(TypeFlags::Union) {
            match &index_type.data {
                TypeData::Union(u) => u.union_or_intersection.types.clone(),
                _ => vec![Arc::clone(&index_type)],
            }
        } else {
            vec![Arc::clone(&index_type)]
        };
        for c in &constituents {
            let mut ok = self.is_type_assignable_to(c, &object_index_type);
            if !ok && has_number_index_info {

                ok = self.is_type_assignable_to(c, &self.number_type());
            }
            if ok {
                continue;
            }
            if object_type.object_flags.intersects(
                crate::checker::types::ObjectFlags::IsGenericObjectType,
            ) {

                if let Some(name) = self.property_name_from_index(c) {
                    if let Some(sym) = self.get_constituent_property(&object_type, &name) {
                        let non_public = sym
                            .value_declaration
                            .as_ref()
                            .map(|d| {
                                self.get_combined_modifier_flags(d).intersects(
                                    crate::ast::ModifierFlags::NonPublicAccessibilityModifier,
                                )
                            })
                            .unwrap_or(false);
                        if non_public {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                crate::diagnostics::messages_generated::
                                    PRIVATE_OR_PROTECTED_MEMBER_0_CANNOT_BE_ACCESSED_ON_A_TYPE_PARAMETER,
                                vec![name],
                            ));
                            return;
                        }
                    }
                }
            }
            let index_display = self.type_to_string(&index_type);
            let object_display = self.type_to_string(&object_type);
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::TYPE_0_CANNOT_BE_USED_TO_INDEX_TYPE_1,
                vec![index_display, object_display],
            ));
            return;
        }
    }

    fn property_name_from_index(&mut self, t: &Arc<Type>) -> Option<String> {
        use crate::checker::types::{TypeData, TypeFlags};
        if t.flags.intersects(TypeFlags::StringLiteral | TypeFlags::NumberLiteral) {
            if let TypeData::Literal(l) = &t.data {
                return match &l.value {
                    crate::checker::types::LiteralValue::String(s) => Some(s.clone()),
                    crate::checker::types::LiteralValue::Number(n) => Some(n.to_string()),
                    _ => None,
                };
            }
        }
        None
    }
    pub(crate) fn check_heritage_clause(&mut self, node: &Arc<Node>) {

        let data = match &node.data {
            crate::ast::NodeData::HeritageClause(d) => d,
            _ => return,
        };
        if data.token == SyntaxKind::ExtendsKeyword {

            if data.types.len() > 1 {
                for type_ref in data.types.iter().skip(1) {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        type_ref.loc,
                        crate::diagnostics::messages_generated::
                            CLASSES_CAN_ONLY_EXTEND_A_SINGLE_CLASS,
                        Vec::new(),
                    ));
                }
            }

            for type_ref in data.types.iter() {
                if let crate::ast::NodeData::ExpressionWithTypeArguments(ewa) = &type_ref.data {

                    if ewa.expression.kind == SyntaxKind::Identifier {
                        if let Some(sym) = self.resolve_identifier(&ewa.expression)
                            && sym.flags == SymbolFlags::Interface
                        {
                            let name = ewa.expression.text().to_string();
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                ewa.expression.loc,
                                crate::diagnostics::messages_generated::
                                    CANNOT_EXTEND_AN_INTERFACE_0_DID_YOU_MEAN_IMPLEMENTS,
                                vec![name],
                            ));
                        }
                    }

                    self.push_ts2304_suppression();
                    let _ = self.get_type_from_type_node(&ewa.expression);
                    self.pop_ts2304_suppression();
                }
            }
            return;
        }
        if data.token != SyntaxKind::ImplementsKeyword {
            return;
        }

        let class_node = match node.parent.as_ref() {
            Some(p) => p,
            None => return,
        };
        let class_data = match &class_node.data {
            crate::ast::NodeData::ClassDeclaration(d) => d,
            _ => return,
        };
        let class_name = class_data
            .name
            .as_ref()
            .map(|n| n.text().to_string())
            .unwrap_or_default();

        let instance_type = self.build_class_instance_type_with_base(class_node);

        for type_ref in data.types.iter() {
            let interface_type = self.get_type_from_heritage_type_reference(type_ref);
            if interface_type.flags.contains(TypeFlags::Any) {

                continue;
            }
            if !self.is_type_assignable_to(&instance_type, &interface_type) {

                let mut issued_member_error = false;
                for member in class_data.members.iter() {
                    if member.has_syntactic_modifier(ModifierFlags::Static) {
                        continue;
                    }
                    let name_node = match &member.data {
                        crate::ast::NodeData::PropertyDeclaration(d) => &d.name,
                        crate::ast::NodeData::MethodDeclaration(d) => &d.name,
                        crate::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                        crate::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
                        _ => continue,
                    };
                    let prop_name = name_node.text().to_string();
                    if prop_name.is_empty() {
                        continue;
                    }
                    let Some(prop) = self.get_property_of_type(&instance_type, &prop_name)
                    else {
                        continue;
                    };
                    let Some(base_prop) = self.get_property_of_type(&interface_type, &prop_name)
                    else {
                        continue;
                    };
                    let prop_type = self.get_type_of_symbol(&prop);
                    let base_type = self.get_type_of_symbol(&base_prop);
                    if !self.is_type_assignable_to(&prop_type, &base_type) {
                        let class_str = self.type_to_string(&instance_type);
                        let iface_str = self.type_to_string(&interface_type);
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            name_node.loc,
                            crate::diagnostics::messages_generated::
                                PROPERTY_0_IN_TYPE_1_IS_NOT_ASSIGNABLE_TO_THE_SAME_PROPERTY_IN_BASE_TYPE_2,
                            vec![prop_name, class_str, iface_str],
                        ));
                        issued_member_error = true;
                        break;
                    }
                }
                if !issued_member_error {
                    let iface_name = self.type_to_string(&interface_type);
                    self.grammar_error_on_node_with_args(
                        class_node,
                        &crate::diagnostics::messages_generated::CLASS_0_INCORRECTLY_IMPLEMENTS_INTERFACE_1,
                        &[class_name.clone(), iface_name],
                    );
                }
            }
        }
    }

    #[allow(dead_code)]
    fn build_class_instance_type(&mut self, members: &Arc<NodeList>) -> Arc<Type> {
        self.build_interface_type_from_members(members)
    }
}
