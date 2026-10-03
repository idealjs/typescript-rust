#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::node::NodeList;

#[path = "r36k34_defs.rs"]
pub mod r36k34_defs;

use crate::printer::NodeFactory;
use std::sync::OnceLock;
use tsox_frontend::format::mig::m4o_2::{EmitHelper, Priority};


fn create_binding_helper_arc() -> &'static Arc<EmitHelper> { ::tsox_core::fntrace::enter("create_binding_helper_arc"); 
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| {
        Arc::new(EmitHelper {
            name: "typescript:commonjscreatebinding".to_string(),
            scoped: false,
            text: r#"var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));"#
                .to_string(),
            text_callback: None,
            priority: Some(Priority { value: 1 }),
            dependencies: Vec::new(),
            import_name: "__createBinding".to_string(),
        })
    })
}

pub fn create_binding_helper() -> &'static EmitHelper { ::tsox_core::fntrace::enter("create_binding_helper"); 
    create_binding_helper_arc()
}

pub fn es_decorate_helper() -> &'static Arc<EmitHelper> { ::tsox_core::fntrace::enter("es_decorate_helper"); 
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:esDecorate".to_string(),
        scoped: false,
        text: r#"var __esDecorate = (this && this.__esDecorate) || function (ctor, descriptorIn, decorators, contextIn, initializers, extraInitializers) {
    function accept(f) { if (f !== void 0 && typeof f !== "function") throw new TypeError("Function expected"); return f; }
    var kind = contextIn.kind, key = kind === "getter" ? "get" : kind === "setter" ? "set" : "value";
    var target = !descriptorIn && ctor ? contextIn["static"] ? ctor : ctor.prototype : null;
    var descriptor = descriptorIn || (target ? Object.getOwnPropertyDescriptor(target, contextIn.name) : {});
    var _, done = false;
    for (var i = decorators.length - 1; i >= 0; i--) {
        var context = {};
        for (var p in contextIn) context[p] = p === "access" ? {} : contextIn[p];
        for (var p in contextIn.access) context.access[p] = contextIn.access[p];
        context.addInitializer = function (f) { if (done) throw new TypeError("Cannot add initializers after decoration has completed"); extraInitializers.push(accept(f || null)); };
        var result = (0, decorators[i])(kind === "accessor" ? { get: descriptor.get, set: descriptor.set } : descriptor[key], context);
        if (kind === "accessor") {
            if (result === void 0) continue;
            if (result === null || typeof result !== "object") throw new TypeError("Object expected");
            if (_ = accept(result.get)) descriptor.get = _;
            if (_ = accept(result.set)) descriptor.set = _;
            if (_ = accept(result.init)) initializers.unshift(_);
        }
        else if (_ = accept(result)) {
            if (kind === "field") initializers.unshift(_);
            else descriptor[key] = _;
        }
    }
    if (target) Object.defineProperty(target, contextIn.name, descriptor);
    done = true;
};"#
        .to_string(),
        text_callback: None,
        priority: Some(Priority { value: 2 }),
        dependencies: Vec::new(),
        import_name: "__esDecorate".to_string(),
    }))
}

pub fn export_star_helper() -> &'static Arc<EmitHelper> { ::tsox_core::fntrace::enter("export_star_helper"); 
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(EmitHelper {
        name: "typescript:export-star".to_string(),
        scoped: false,
        text: r#"var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};"#
        .to_string(),
        text_callback: None,
        priority: Some(Priority { value: 2 }),
        dependencies: vec![create_binding_helper_arc().clone()],
        import_name: "__exportStar".to_string(),
    }))
}

impl<'a> NodeFactory<'a> {
    pub fn new_es_decorate_helper(
        &self,
        ctor: &Arc<Node>,
        descriptor_in: &Arc<Node>,
        decorators: &Arc<Node>,
        context_in: &Arc<Node>,
        initializers: &Arc<Node>,
        extra_initializers: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_es_decorate_helper"); 
        self.emit_context_mut()
            .request_emit_helper(es_decorate_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__esDecorate"),
            None,
            None,
            self.new_node_list(vec![
                Arc::clone(ctor),
                Arc::clone(descriptor_in),
                Arc::clone(decorators),
                Arc::clone(context_in),
                Arc::clone(initializers),
                Arc::clone(extra_initializers),
            ]),
            NodeFlags::empty(),
        )
    }

    pub fn new_export_default(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_export_default"); 
        self.new_export_assignment(None, false, None, expression)
    }

    pub fn new_export_star_helper(
        &self,
        module_expression: &Arc<Node>,
        exports_expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_export_star_helper"); 
        self.emit_context_mut()
            .request_emit_helper(export_star_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__exportStar"),
            None,
            None,
            self.new_node_list(vec![
                Arc::clone(module_expression),
                Arc::clone(exports_expression),
            ]),
            NodeFlags::empty(),
        )
    }

    pub fn new_external_module_export(&self, name: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_external_module_export"); 
        let specifier = self.new_export_specifier(false, None, name);
        let named_exports = self.new_named_exports(&self.new_node_list(vec![specifier]));
        self.new_export_declaration(None, false, &named_exports, None, None)
    }

    pub fn new_function_bind_call(
        &self,
        target: &Arc<Node>,
        this_arg: &Arc<Node>,
        arguments_list: &[Arc<Node>],
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_function_bind_call"); 
        let mut args = Vec::with_capacity(1 + arguments_list.len());
        args.push(Arc::clone(this_arg));
        args.extend(arguments_list.iter().cloned());
        self.new_method_call(target, &self.new_identifier("bind"), args)
    }

    pub fn new_assignment_target_wrapper(
        &self,
        param_name: &Arc<Node>,
        expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_assignment_target_wrapper"); 
        let set_accessor = self.new_set_accessor_declaration(
            None,
            &self.new_identifier("value"),
            None,
            self.new_node_list(vec![self.new_parameter_declaration(
                None,
                None,
                param_name,
                None,
                None,
                None,
            )]),
            None,
            None,
            self.new_block(
                &self.new_node_list(vec![self.new_expression_statement(expression)]),
                false,
            ),
        );
        let obj_literal =
            self.new_object_literal_expression(&self.new_node_list(vec![set_accessor]), false);
        self.new_property_access_expression(
            &self.new_parenthesized_expression(&obj_literal),
            None,
            &self.new_identifier("value"),
            NodeFlags::empty(),
        )
    }
}
