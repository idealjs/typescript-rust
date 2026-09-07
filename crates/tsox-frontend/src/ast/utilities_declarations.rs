use crate::ast::*;

pub fn is_declaration(node: &Node) -> bool {
    if node.kind == SyntaxKind::TypeParameter {
        return node.parent.is_some();
    }
    is_declaration_node(node)
}

pub fn is_declaration_node(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::BindingElement
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::SpreadAssignment
            | SyntaxKind::EnumMember
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ImportClause
            | SyntaxKind::NamespaceImport
            | SyntaxKind::NamespaceExport
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::MissingDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::JsxAttribute
            | SyntaxKind::JsxSpreadAttribute
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::TypeParameter
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::NamedTupleMember
    )
}

pub fn can_have_symbol(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ArrowFunction
            | SyntaxKind::BinaryExpression
            | SyntaxKind::BindingElement
            | SyntaxKind::CallExpression
            | SyntaxKind::CallSignature
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::ConstructorType
            | SyntaxKind::ConstructSignature
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::EnumMember
            | SyntaxKind::ExportAssignment
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionType
            | SyntaxKind::GetAccessor
            | SyntaxKind::ImportClause
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::IndexSignature
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::JsxAttribute
            | SyntaxKind::JsxAttributes
            | SyntaxKind::JsxSpreadAttribute
            | SyntaxKind::MappedType
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::NamedTupleMember
            | SyntaxKind::NamespaceExport
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::NamespaceImport
            | SyntaxKind::NewExpression
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::Parameter
            | SyntaxKind::PropertyAccessExpression
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::SetAccessor
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::SourceFile
            | SyntaxKind::SpreadAssignment
            | SyntaxKind::StringLiteral
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::TypeLiteral
            | SyntaxKind::TypeParameter
            | SyntaxKind::VariableDeclaration
    )
}

pub fn can_have_modifiers(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::TypeParameter
            | SyntaxKind::Parameter
            | SyntaxKind::PropertySignature
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::IndexSignature
            | SyntaxKind::ConstructorType
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::ClassExpression
            | SyntaxKind::VariableStatement
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::ExportDeclaration
    )
}

pub fn can_have_decorators(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Parameter
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ClassExpression
            | SyntaxKind::ClassDeclaration
    )
}
