use tsox_frontend::ast::*;

/// Go declareSymbolAndAddToSymbolTable 的函数类容器分支：这些容器内的
/// 声明（类型参数、参数、局部声明）进容器节点的 locals 表（parent 不挂），
/// 而非容器符号的 members —— 重载声明的同名类型参数必须在各自的
/// 函数节点 locals 里隔离，进符号 members 会相互覆盖
pub(crate) fn is_function_like_locals_container(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::MappedType
    )
}

pub(crate) fn get_container_flags(kind: SyntaxKind) -> ContainerFlags {
    match kind {
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::InterfaceDeclaration
        | SyntaxKind::TypeLiteral
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::JsxAttributes
        | SyntaxKind::EnumDeclaration => ContainerFlags::IS_CONTAINER,
        SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::IS_FUNCTION_EXPRESSION
                | ContainerFlags::HAS_LOCALS
                | ContainerFlags::IS_THIS_CONTAINER
        }
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::Constructor
        | SyntaxKind::ClassStaticBlockDeclaration => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::HAS_LOCALS
                | ContainerFlags::IS_THIS_CONTAINER
        }

        SyntaxKind::MethodSignature
        | SyntaxKind::CallSignature
        | SyntaxKind::ConstructSignature
        | SyntaxKind::FunctionType
        | SyntaxKind::ConstructorType => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::IndexSignature => ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS,

        SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::MappedType => ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS,
        SyntaxKind::Block | SyntaxKind::ModuleDeclaration | SyntaxKind::SourceFile => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_BLOCK_SCOPED_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::CatchClause
        | SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement => {
            ContainerFlags::IS_BLOCK_SCOPED_CONTAINER | ContainerFlags::HAS_LOCALS
        }
        _ => ContainerFlags::NONE,
    }
}

#[allow(dead_code)]
pub(crate) fn is_block_scoped_container(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::SourceFile
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::Constructor
    )
}

pub(crate) fn is_block_only_container(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::CaseBlock
    )
}

pub(crate) fn is_control_flow_jump_reset_container(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::SourceFile
            | SyntaxKind::ModuleBlock
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::ClassStaticBlockDeclaration
    )
}

pub(crate) fn is_var_container_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::SourceFile
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
    )
}

pub(crate) fn has_locals(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::SourceFile
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::MethodSignature
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::MappedType
    )
}
