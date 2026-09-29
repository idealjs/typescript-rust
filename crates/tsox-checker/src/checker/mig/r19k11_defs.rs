use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::{
    ConditionalExpressionData, IndexedAccessTypeNodeData, IntersectionTypeNodeData,
    PrefixUnaryExpressionData, SourceFileData, SyntheticExpressionData, TypeOperatorNodeData,
    TypeParameterDeclarationData, UnionTypeNodeData,
};
use tsox_frontend::ast::Node;

use crate::checker::checker::Checker;
use crate::checker::types_type_flags_instantiable_non_primitive::ObjectFlags;
use crate::checker::relater_relation::RelationKind;
use crate::checker::types::{Signature, SignatureFlags, Type};

pub trait R19K11NodeExt {
    fn as_type_operator_node(&self) -> &TypeOperatorNodeData;
    fn as_union_type_node(&self) -> &UnionTypeNodeData;
    fn as_intersection_type_node(&self) -> &IntersectionTypeNodeData;
    fn as_indexed_access_type_node(&self) -> &IndexedAccessTypeNodeData;
    fn as_source_file(&self) -> &SourceFileData;
    fn as_synthetic_expression(&self) -> &SyntheticExpressionData;
    fn as_type_parameter_declaration(&self) -> &TypeParameterDeclarationData;
    fn as_conditional_expression(&self) -> &ConditionalExpressionData;
    fn as_prefix_unary_expression(&self) -> &PrefixUnaryExpressionData;
}

macro_rules! as_data11 {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                tsox_frontend::ast::NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

impl R19K11NodeExt for Node {
    as_data11!(as_type_operator_node, TypeOperatorNode, TypeOperatorNodeData);
    as_data11!(as_union_type_node, UnionTypeNode, UnionTypeNodeData);
    as_data11!(
        as_intersection_type_node,
        IntersectionTypeNode,
        IntersectionTypeNodeData
    );
    as_data11!(
        as_indexed_access_type_node,
        IndexedAccessTypeNode,
        IndexedAccessTypeNodeData
    );
    as_data11!(as_source_file, SourceFile, SourceFileData);
    as_data11!(
        as_synthetic_expression,
        SyntheticExpression,
        SyntheticExpressionData
    );
    as_data11!(
        as_type_parameter_declaration,
        TypeParameterDeclaration,
        TypeParameterDeclarationData
    );
    as_data11!(
        as_conditional_expression,
        ConditionalExpression,
        ConditionalExpressionData
    );
    as_data11!(
        as_prefix_unary_expression,
        PrefixUnaryExpression,
        PrefixUnaryExpressionData
    );
}

pub trait R19K11CheckerExt {
    fn get_global_typed_property_descriptor_type(&mut self) -> Arc<Type>;
    fn get_global_class_decorator_context_type(&mut self) -> Arc<Type>;
    fn get_global_class_method_decorator_context_type(&mut self) -> Arc<Type>;
    fn get_global_class_getter_decorator_context_type(&mut self) -> Arc<Type>;
    fn get_global_class_setter_decorator_context_type(&mut self) -> Arc<Type>;
    fn get_global_class_accessor_decorator_context_type(&mut self) -> Arc<Type>;
    fn get_global_class_accessor_decorator_target_type(&mut self) -> Arc<Type>;
    fn get_global_class_accessor_decorator_result_type(&mut self) -> Arc<Type>;
    fn get_global_class_field_decorator_context_type(&mut self) -> Arc<Type>;
    fn new_call_signature(
        &mut self,
        type_parameters: &[Arc<Type>],
        this_parameter: Option<&Arc<tsox_frontend::ast::Symbol>>,
        parameters: &[Arc<tsox_frontend::ast::Symbol>],
        return_type: &Arc<Type>,
    ) -> Arc<Signature>;
    fn new_function_type(
        &mut self,
        type_parameters: &[Arc<Type>],
        this_parameter: Option<&Arc<tsox_frontend::ast::Symbol>>,
        parameters: &[Arc<tsox_frontend::ast::Symbol>],
        return_type: &Arc<Type>,
    ) -> Arc<Type>;
    fn unknown_union_type(&mut self) -> Arc<Type>;
    fn unknown_empty_object_type(&mut self) -> Arc<Type>;
    fn strict_subtyping_relation(&mut self) -> RelationKind;
    fn marker_super_type(&mut self) -> Arc<Type>;
    fn marker_sub_type(&mut self) -> Arc<Type>;
    fn marker_other_type(&mut self) -> Arc<Type>;
    fn unknown_signature(&mut self) -> Arc<Signature>;
}

macro_rules! global_type_getter {
    ($name:ident, $go_name:expr, $arity:expr) => {
        fn $name(&mut self) -> Arc<Type> {
            self.get_global_type($go_name, $arity, true)
        }
    };
}

impl R19K11CheckerExt for Checker {
    global_type_getter!(
        get_global_typed_property_descriptor_type,
        "TypedPropertyDescriptor",
        1
    );
    global_type_getter!(get_global_class_decorator_context_type, "ClassDecoratorContext", 1);
    global_type_getter!(
        get_global_class_method_decorator_context_type,
        "ClassMethodDecoratorContext",
        2
    );
    global_type_getter!(
        get_global_class_getter_decorator_context_type,
        "ClassGetterDecoratorContext",
        2
    );
    global_type_getter!(
        get_global_class_setter_decorator_context_type,
        "ClassSetterDecoratorContext",
        2
    );
    global_type_getter!(
        get_global_class_accessor_decorator_context_type,
        "ClassAccessorDecoratorContext",
        2
    );
    global_type_getter!(
        get_global_class_accessor_decorator_target_type,
        "ClassAccessorDecoratorTarget",
        2
    );
    global_type_getter!(
        get_global_class_accessor_decorator_result_type,
        "ClassAccessorDecoratorResult",
        2
    );
    global_type_getter!(
        get_global_class_field_decorator_context_type,
        "ClassFieldDecoratorContext",
        2
    );

    fn new_call_signature(
        &mut self,
        type_parameters: &[Arc<Type>],
        this_parameter: Option<&Arc<tsox_frontend::ast::Symbol>>,
        parameters: &[Arc<tsox_frontend::ast::Symbol>],
        return_type: &Arc<Type>,
    ) -> Arc<Signature> {
        self.new_signature(
            SignatureFlags::empty(),
            None,
            type_parameters,
            this_parameter,
            parameters,
            return_type,
            None,
            parameters.len(),
        )
    }

    fn new_function_type(
        &mut self,
        type_parameters: &[Arc<Type>],
        this_parameter: Option<&Arc<tsox_frontend::ast::Symbol>>,
        parameters: &[Arc<tsox_frontend::ast::Symbol>],
        return_type: &Arc<Type>,
    ) -> Arc<Type> {
        let signature = <Self as R19K11CheckerExt>::new_call_signature(
            self,
            type_parameters,
            this_parameter,
            parameters,
            return_type,
        );
        self.get_or_create_type_from_signature(&signature)
    }

    fn unknown_union_type(&mut self) -> Arc<Type> {
        let unknown = self.unknown_type.get().cloned().expect("unknown_type");
        let null = self.null_type.get().cloned().expect("null_type");
        self.get_union_type(vec![unknown, null])
    }

    fn unknown_empty_object_type(&mut self) -> Arc<Type> {
        self.empty_object_type.get().cloned().expect("empty_object_type")
    }

    fn strict_subtyping_relation(&mut self) -> RelationKind {
        RelationKind::StrictSubtype
    }

    fn marker_super_type(&mut self) -> Arc<Type> {
        Arc::clone(self.marker_super_type.get_or_init(|| Checker::new_marker_type_parameter(None)))
    }

    fn marker_sub_type(&mut self) -> Arc<Type> {
        Arc::clone(self.marker_sub_type.get_or_init(|| Checker::new_marker_type_parameter(None)))
    }

    fn marker_other_type(&mut self) -> Arc<Type> {
        Arc::clone(self.marker_other_type.get_or_init(|| Checker::new_marker_type_parameter(None)))
    }

    fn unknown_signature(&mut self) -> Arc<Signature> {
        self.unknown_signature.get().cloned().expect("unknown_signature")
    }
}

pub const OBJECT_FLAGS_IS_UNIFORM_ENUM_COMPUTED: ObjectFlags = ObjectFlags::from_bits_truncate(1 << 28);
pub const OBJECT_FLAGS_IS_UNIFORM_ENUM: ObjectFlags = ObjectFlags::from_bits_truncate(1 << 29);
