//! Go format/rules.go 的移植：135 条格式化规则规格。
//! 规则按 high → user → low 优先级排列，processPair 依序取第一条命中。

use crate::ast::SyntaxKind;
use crate::format::rule::{
    kind_range, kinds_range, r, RuleAction, RuleFlags, RuleSpec, TokenRange,
};
use crate::format::rule_context_2::*;

use super::rule_context::FormattingContext;

fn all_tokens() -> Vec<SyntaxKind> {
    use SyntaxKind::*;
    vec![
        Identifier, PrivateIdentifier, PropertySignature, CommaToken,
        SemicolonToken, ColonToken, QuestionToken, QuestionQuestionToken, DotToken,
        DotDotDotToken, EqualsToken, EqualsEqualsToken, EqualsEqualsEqualsToken,
        ExclamationEqualsToken, ExclamationEqualsEqualsToken, LessThanToken,
        LessThanSlashToken, GreaterThanToken, LessThanEqualsToken, GreaterThanEqualsToken,
        PlusToken, MinusToken, AsteriskToken, AsteriskAsteriskToken, SlashToken,
        SlashEqualsToken, PercentToken, PlusPlusToken, MinusMinusToken,
        LessThanLessThanToken, GreaterThanGreaterThanToken,
        GreaterThanGreaterThanGreaterThanToken, AmpersandToken, BarToken,
        CaretToken, AmpersandAmpersandToken, BarBarToken, ExclamationToken, TildeToken,
        QuestionDotToken, OpenParenToken, CloseParenToken, OpenBracketToken,
        CloseBracketToken, OpenBraceToken, CloseBraceToken, AtToken, BacktickToken,
        DefaultKeyword, EqualsGreaterThanToken, NumericLiteral, BigIntLiteral,
        StringLiteral, JsxText, JsxTextAllWhiteSpaces, NoSubstitutionTemplateLiteral,
        TemplateHead, TemplateMiddle, TemplateTail, RegularExpressionLiteral, TrueKeyword,
        FalseKeyword, NullKeyword, ThisKeyword, SuperKeyword, NewKeyword,
        ModuleKeyword, RequireKeyword, YieldKeyword, AwaitKeyword, AsyncKeyword,
        PublicKeyword, PrivateKeyword, ProtectedKeyword, ReadonlyKeyword,
        AbstractKeyword, AccessorKeyword, DeclareKeyword, OverrideKeyword, EnumKeyword,
        ExportKeyword, ImportKeyword, ClassKeyword, InterfaceKeyword, TypeKeyword,
        FromKeyword, KeyOfKeyword, InferKeyword, AsKeyword, IsKeyword, SatisfiesKeyword,
        ConstKeyword, LetKeyword, VarKeyword, FunctionKeyword, ConstructorKeyword,
        GetKeyword, SetKeyword, StaticKeyword, InKeyword, InstanceOfKeyword, OfKeyword,
        IfKeyword, ElseKeyword, DoKeyword, WhileKeyword, ForKeyword, ReturnKeyword,
        SwitchKeyword, CaseKeyword, BreakKeyword, ContinueKeyword, TryKeyword,
        CatchKeyword, FinallyKeyword, ThrowKeyword, WithKeyword, DeleteKeyword,
        VoidKeyword, TypeOfKeyword,
    ]
}

fn any_token() -> TokenRange {
    TokenRange { tokens: all_tokens() }
}

fn any_token_except(tokens: &[SyntaxKind]) -> TokenRange {
    let excluded = tokens.to_vec();
    TokenRange { tokens: all_tokens().into_iter().filter(|t| !excluded.contains(t)).collect() }
}

fn keywords() -> TokenRange {
    let kw: Vec<SyntaxKind> = all_tokens()
        .into_iter()
        .filter(|t| crate::scanner::is_jsx_line_break::is_keyword(*t))
        .collect();
    TokenRange { tokens: kw }
}

fn binary_operators() -> TokenRange {
    use SyntaxKind::*;
    kinds_range(&[
        EqualsToken, PlusEqualsToken, MinusEqualsToken, AsteriskEqualsToken,
        SlashEqualsToken, PercentEqualsToken, LessThanLessThanEqualsToken,
        GreaterThanGreaterThanEqualsToken, GreaterThanGreaterThanGreaterThanEqualsToken,
        AmpersandEqualsToken, BarEqualsToken, CaretEqualsToken, QuestionQuestionEqualsToken,
        LessThanToken, GreaterThanToken, LessThanEqualsToken, GreaterThanEqualsToken,
        EqualsEqualsToken, EqualsEqualsEqualsToken, ExclamationEqualsToken,
        ExclamationEqualsEqualsToken, AsteriskToken, SlashToken, PercentToken, PlusToken,
        MinusToken, AsteriskAsteriskToken, LessThanLessThanToken, GreaterThanGreaterThanToken,
        GreaterThanGreaterThanGreaterThanToken, AmpersandToken, BarToken, CaretToken,
        AmpersandAmpersandToken, BarBarToken, QuestionQuestionToken,
    ])
}

fn binary_keyword_operators() -> TokenRange {
    kinds_range(&[
        SyntaxKind::InKeyword,
        SyntaxKind::InstanceOfKeyword,
        SyntaxKind::OfKeyword,
        SyntaxKind::AsKeyword,
        SyntaxKind::IsKeyword,
        SyntaxKind::SatisfiesKeyword,
    ])
}

fn unary_prefix_operators() -> TokenRange {
    kinds_range(&[
        SyntaxKind::PlusPlusToken,
        SyntaxKind::MinusMinusToken,
        SyntaxKind::TildeToken,
        SyntaxKind::ExclamationToken,
    ])
}

fn unary_prefix_expressions() -> TokenRange {
    kinds_range(&[
        SyntaxKind::NumericLiteral,
        SyntaxKind::BigIntLiteral,
        SyntaxKind::Identifier,
        SyntaxKind::OpenParenToken,
        SyntaxKind::OpenBracketToken,
        SyntaxKind::OpenBraceToken,
        SyntaxKind::ThisKeyword,
        SyntaxKind::NewKeyword,
    ])
}

fn unary_preincrement_expressions() -> TokenRange {
    kinds_range(&[
        SyntaxKind::Identifier,
        SyntaxKind::OpenParenToken,
        SyntaxKind::ThisKeyword,
        SyntaxKind::NewKeyword,
    ])
}

fn unary_postincrement_expressions() -> TokenRange {
    kinds_range(&[
        SyntaxKind::Identifier,
        SyntaxKind::CloseParenToken,
        SyntaxKind::CloseBracketToken,
        SyntaxKind::NewKeyword,
    ])
}

fn comments() -> TokenRange {
    kinds_range(&[
        SyntaxKind::SingleLineCommentTrivia,
        SyntaxKind::MultiLineCommentTrivia,
    ])
}

fn type_names() -> TokenRange {
    let mut tokens = vec![SyntaxKind::Identifier];
    tokens.extend(
        kinds_range(&[
            SyntaxKind::AnyKeyword,
            SyntaxKind::AssertsKeyword,
            SyntaxKind::BigIntKeyword,
            SyntaxKind::BooleanKeyword,
            SyntaxKind::FalseKeyword,
            SyntaxKind::InferKeyword,
            SyntaxKind::KeyOfKeyword,
            SyntaxKind::NeverKeyword,
            SyntaxKind::NullKeyword,
            SyntaxKind::NumberKeyword,
            SyntaxKind::ObjectKeyword,
            SyntaxKind::ReadonlyKeyword,
            SyntaxKind::StringKeyword,
            SyntaxKind::SymbolKeyword,
            SyntaxKind::TypeOfKeyword,
            SyntaxKind::TrueKeyword,
            SyntaxKind::VoidKeyword,
            SyntaxKind::UndefinedKeyword,
            SyntaxKind::UniqueKeyword,
            SyntaxKind::UnknownKeyword,
        ])
        .tokens,
    );
    TokenRange { tokens }
}

fn any_token_including_multiline_comments() -> TokenRange {
    let mut tokens = all_tokens();
    tokens.push(SyntaxKind::MultiLineCommentTrivia);
    TokenRange { tokens }
}

fn any_token_including_eof() -> TokenRange {
    let mut tokens = all_tokens();
    tokens.push(SyntaxKind::EndOfFile);
    TokenRange { tokens }
}

fn function_open_brace_left() -> TokenRange {
    any_token_including_multiline_comments()
}

fn type_script_open_brace_left() -> TokenRange {
    kinds_range(&[
        SyntaxKind::Identifier,
        SyntaxKind::GreaterThanToken,
        SyntaxKind::MultiLineCommentTrivia,
        SyntaxKind::ClassKeyword,
        SyntaxKind::ExportKeyword,
        SyntaxKind::ImportKeyword,
    ])
}

fn control_open_brace_left() -> TokenRange {
    kinds_range(&[
        SyntaxKind::CloseParenToken,
        SyntaxKind::MultiLineCommentTrivia,
        SyntaxKind::DoKeyword,
        SyntaxKind::TryKeyword,
        SyntaxKind::FinallyKeyword,
        SyntaxKind::ElseKeyword,
        SyntaxKind::CatchKeyword,
    ])
}

pub(crate) fn get_all_rules() -> Vec<RuleSpec> {
    use SyntaxKind::*;
    let mut rules: Vec<RuleSpec> = Vec::new();

    let mut add = |name: &'static str,
                   left: TokenRange,
                   right: TokenRange,
                   context: Vec<crate::format::rule::ContextPredicate>,
                   action: RuleAction,
                   flags: RuleFlags| {
        rules.push(r(name).rule(left, right, context, action, flags));
    };

    // 高优先级公共规则（不可被用户选项影响）
    add("IgnoreBeforeComment", any_token(), comments(), Vec::new(),
        RuleAction::STOP_PROCESSING_SPACE_ACTIONS, RuleFlags::None);
    add("IgnoreAfterLineComment", kind_range(SyntaxKind::SingleLineCommentTrivia), any_token(), Vec::new(),
        RuleAction::STOP_PROCESSING_SPACE_ACTIONS, RuleFlags::None);
    add("NotSpaceBeforeColon", any_token(), kind_range(SyntaxKind::ColonToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_not_binary_op_context, FormattingContext::is_not_type_annotation_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterColon", kind_range(SyntaxKind::ColonToken), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_not_binary_op_context, FormattingContext::is_next_token_parent_not_jsx_namespaced_name],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBeforeQuestionMark", any_token(), kind_range(SyntaxKind::QuestionToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_not_binary_op_context, FormattingContext::is_not_type_annotation_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterQuestionMarkInConditionalOperator", kind_range(SyntaxKind::QuestionToken), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_conditional_operator_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterQuestionMark", kind_range(SyntaxKind::QuestionToken), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_non_optional_property_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBeforeDot", any_token(), kinds_range(&[SyntaxKind::DotToken, SyntaxKind::QuestionDotToken]),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_not_property_access_on_integer_literal],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterDot", kinds_range(&[SyntaxKind::DotToken, SyntaxKind::QuestionDotToken]), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBetweenImportParenInImportType", kind_range(SyntaxKind::ImportKeyword), kind_range(SyntaxKind::OpenParenToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_import_type_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);

    add("NoSpaceAfterUnaryPrefixOperator", unary_prefix_operators(), unary_prefix_expressions(),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_not_binary_op_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterUnaryPreincrementOperator", kind_range(SyntaxKind::PlusPlusToken), unary_preincrement_expressions(),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterUnaryPredecrementOperator", kind_range(SyntaxKind::MinusMinusToken), unary_preincrement_expressions(),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBeforeUnaryPostincrementOperator", unary_postincrement_expressions(), kind_range(SyntaxKind::PlusPlusToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_not_statement_condition_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBeforeUnaryPostdecrementOperator", unary_postincrement_expressions(), kind_range(SyntaxKind::MinusMinusToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_not_statement_condition_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);

    add("SpaceAfterPostincrementWhenFollowedByAdd", kind_range(SyntaxKind::PlusPlusToken), kind_range(SyntaxKind::PlusToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_binary_op_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterAddWhenFollowedByUnaryPlus", kind_range(SyntaxKind::PlusToken), kind_range(SyntaxKind::PlusToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_binary_op_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterAddWhenFollowedByPreincrement", kind_range(SyntaxKind::PlusToken), kind_range(SyntaxKind::PlusPlusToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_binary_op_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterPostdecrementWhenFollowedBySubtract", kind_range(SyntaxKind::MinusMinusToken), kind_range(SyntaxKind::MinusToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_binary_op_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterSubtractWhenFollowedByUnaryMinus", kind_range(SyntaxKind::MinusToken), kind_range(SyntaxKind::MinusToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_binary_op_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterSubtractWhenFollowedByPredecrement", kind_range(SyntaxKind::MinusToken), kind_range(SyntaxKind::MinusMinusToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_binary_op_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);

    add("NoSpaceAfterCloseBrace", kind_range(SyntaxKind::CloseBraceToken), kinds_range(&[SyntaxKind::CommaToken, SyntaxKind::SemicolonToken]),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NewLineBeforeCloseBraceInBlockContext", any_token_including_multiline_comments(), kind_range(SyntaxKind::CloseBraceToken),
        vec![FormattingContext::is_multiline_block_context],
        RuleAction::INSERT_NEW_LINE, RuleFlags::None);
    add("SpaceAfterCloseBrace", kind_range(SyntaxKind::CloseBraceToken), any_token_except(&[SyntaxKind::CloseParenToken]),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_after_code_block_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBetweenCloseBraceAndElse", kind_range(SyntaxKind::CloseBraceToken), kind_range(SyntaxKind::ElseKeyword),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBetweenCloseBraceAndWhile", kind_range(SyntaxKind::CloseBraceToken), kind_range(SyntaxKind::WhileKeyword),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBetweenEmptyBraceBrackets", kind_range(SyntaxKind::OpenBraceToken), kind_range(SyntaxKind::CloseBraceToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_object_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterConditionalClosingParen", kind_range(SyntaxKind::CloseParenToken), kind_range(SyntaxKind::OpenBracketToken),
        vec![FormattingContext::is_control_decl_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBetweenFunctionKeywordAndStar", kind_range(SyntaxKind::FunctionKeyword), kind_range(SyntaxKind::AsteriskToken),
        vec![FormattingContext::is_function_declaration_or_function_expression_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterStarInGeneratorDeclaration", kind_range(SyntaxKind::AsteriskToken), kind_range(SyntaxKind::Identifier),
        vec![FormattingContext::is_function_declaration_or_function_expression_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterFunctionInFuncDecl", kind_range(SyntaxKind::FunctionKeyword), any_token(),
        vec![FormattingContext::is_function_decl_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NewLineAfterOpenBraceInBlockContext", kind_range(SyntaxKind::OpenBraceToken), any_token(),
        vec![FormattingContext::is_multiline_block_context],
        RuleAction::INSERT_NEW_LINE, RuleFlags::None);
    add("SpaceAfterGetSetInMember", kinds_range(&[SyntaxKind::GetKeyword, SyntaxKind::SetKeyword]), kind_range(SyntaxKind::Identifier),
        vec![FormattingContext::is_function_decl_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBetweenYieldKeywordAndStar", kind_range(SyntaxKind::YieldKeyword), kind_range(SyntaxKind::AsteriskToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_yield_or_yield_star_with_operand],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceBetweenYieldOrYieldStarAndOperand", kinds_range(&[SyntaxKind::YieldKeyword, SyntaxKind::AsteriskToken]), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_yield_or_yield_star_with_operand],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBetweenReturnAndSemicolon", kind_range(SyntaxKind::ReturnKeyword), kind_range(SyntaxKind::SemicolonToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterCertainKeywords",
        kinds_range(&[SyntaxKind::VarKeyword, SyntaxKind::ThrowKeyword, SyntaxKind::NewKeyword, SyntaxKind::DeleteKeyword, SyntaxKind::ReturnKeyword, SyntaxKind::TypeOfKeyword, SyntaxKind::AwaitKeyword]),
        any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterLetConstInVariableDeclaration", kinds_range(&[SyntaxKind::LetKeyword, SyntaxKind::ConstKeyword]), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_start_of_variable_declaration_list],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBeforeOpenParenInFuncCall", any_token(), kind_range(SyntaxKind::OpenParenToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_function_call_or_new_context, FormattingContext::is_previous_token_not_comma],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceBeforeBinaryKeywordOperator", any_token(), binary_keyword_operators(),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_binary_op_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterBinaryKeywordOperator", binary_keyword_operators(), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_binary_op_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterVoidOperator", kind_range(SyntaxKind::VoidKeyword), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_void_op_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBetweenAsyncAndOpenParen", kind_range(SyntaxKind::AsyncKeyword), kind_range(SyntaxKind::OpenParenToken),
        vec![FormattingContext::is_arrow_function_context, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBetweenAsyncAndFunctionKeyword", kind_range(SyntaxKind::AsyncKeyword),
        kinds_range(&[SyntaxKind::FunctionKeyword, SyntaxKind::Identifier]),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBetweenTagAndTemplateString", kinds_range(&[SyntaxKind::Identifier, SyntaxKind::CloseParenToken]),
        kinds_range(&[SyntaxKind::NoSubstitutionTemplateLiteral, SyntaxKind::TemplateHead]),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceBeforeJsxAttribute", any_token(), kind_range(SyntaxKind::Identifier),
        vec![FormattingContext::is_next_token_parent_jsx_attribute, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBeforeSlashInJsxOpeningElement", any_token(), kind_range(SyntaxKind::SlashToken),
        vec![FormattingContext::is_jsx_self_closing_element_context, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBeforeGreaterThanTokenInJsxOpeningElement", kind_range(SyntaxKind::SlashToken), kind_range(SyntaxKind::GreaterThanToken),
        vec![FormattingContext::is_jsx_self_closing_element_context, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBeforeEqualInJsxAttribute", any_token(), kind_range(SyntaxKind::EqualsToken),
        vec![FormattingContext::is_jsx_attribute_context, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterEqualInJsxAttribute", kind_range(SyntaxKind::EqualsToken), any_token(),
        vec![FormattingContext::is_jsx_attribute_context, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBeforeJsxNamespaceColon", kind_range(SyntaxKind::Identifier), kind_range(SyntaxKind::ColonToken),
        vec![FormattingContext::is_next_token_parent_jsx_namespaced_name],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterJsxNamespaceColon", kind_range(SyntaxKind::ColonToken), kind_range(SyntaxKind::Identifier),
        vec![FormattingContext::is_next_token_parent_jsx_namespaced_name],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterModuleImport", kinds_range(&[SyntaxKind::ModuleKeyword, SyntaxKind::RequireKeyword]), kind_range(SyntaxKind::OpenParenToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterCertainTypeScriptKeywords",
        kinds_range(&[
            SyntaxKind::AbstractKeyword, SyntaxKind::AccessorKeyword, SyntaxKind::ClassKeyword,
            SyntaxKind::DeclareKeyword, SyntaxKind::DefaultKeyword, SyntaxKind::EnumKeyword,
            SyntaxKind::ExportKeyword, SyntaxKind::ExtendsKeyword, SyntaxKind::GetKeyword,
            SyntaxKind::ImplementsKeyword, SyntaxKind::ImportKeyword, SyntaxKind::InterfaceKeyword,
            SyntaxKind::ModuleKeyword, SyntaxKind::NamespaceKeyword, SyntaxKind::OverrideKeyword,
            SyntaxKind::PrivateKeyword, SyntaxKind::PublicKeyword, SyntaxKind::ProtectedKeyword,
            SyntaxKind::ReadonlyKeyword, SyntaxKind::SetKeyword, SyntaxKind::StaticKeyword,
            SyntaxKind::TypeKeyword, SyntaxKind::FromKeyword, SyntaxKind::KeyOfKeyword,
            SyntaxKind::InferKeyword,
        ]),
        any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBeforeCertainTypeScriptKeywords", any_token(),
        kinds_range(&[SyntaxKind::ExtendsKeyword, SyntaxKind::ImplementsKeyword, SyntaxKind::FromKeyword]),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterModuleName", kind_range(SyntaxKind::StringLiteral), kind_range(SyntaxKind::OpenBraceToken),
        vec![FormattingContext::is_module_decl_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBeforeArrow", any_token(), kind_range(SyntaxKind::EqualsGreaterThanToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterArrow", kind_range(SyntaxKind::EqualsGreaterThanToken), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterEllipsis", kind_range(SyntaxKind::DotDotDotToken), kind_range(SyntaxKind::Identifier),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterOptionalParameters", kind_range(SyntaxKind::QuestionToken),
        kinds_range(&[SyntaxKind::CloseParenToken, SyntaxKind::CommaToken]),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_not_binary_op_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBetweenEmptyInterfaceBraceBrackets", kind_range(SyntaxKind::OpenBraceToken), kind_range(SyntaxKind::CloseBraceToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_object_type_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);

    let generic_ctx = vec![
        FormattingContext::is_non_jsx_same_line_token_context,
        FormattingContext::is_type_argument_or_parameter_or_assertion_context,
    ];
    add("NoSpaceBeforeOpenAngularBracket", type_names(), kind_range(SyntaxKind::LessThanToken),
        generic_ctx.clone(), RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBetweenCloseParenAndAngularBracket", kind_range(SyntaxKind::CloseParenToken), kind_range(SyntaxKind::LessThanToken),
        generic_ctx.clone(), RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterOpenAngularBracket", kind_range(SyntaxKind::LessThanToken), any_token(),
        generic_ctx.clone(), RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBeforeCloseAngularBracket", any_token(), kind_range(SyntaxKind::GreaterThanToken),
        generic_ctx.clone(), RuleAction::DELETE_SPACE, RuleFlags::None);
    let mut generic_ctx2 = generic_ctx.clone();
    generic_ctx2.push(FormattingContext::is_not_function_decl_context);
    generic_ctx2.push(FormattingContext::is_non_type_assertion_context);
    add("NoSpaceAfterCloseAngularBracket", kind_range(SyntaxKind::GreaterThanToken),
        kinds_range(&[SyntaxKind::OpenParenToken, SyntaxKind::OpenBracketToken, SyntaxKind::GreaterThanToken, SyntaxKind::CommaToken]),
        generic_ctx2, RuleAction::DELETE_SPACE, RuleFlags::None);

    add("SpaceBeforeAt", kinds_range(&[SyntaxKind::CloseParenToken, SyntaxKind::Identifier]), kind_range(SyntaxKind::AtToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterAt", kind_range(SyntaxKind::AtToken), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterDecorator", any_token(),
        kinds_range(&[
            SyntaxKind::AbstractKeyword, SyntaxKind::Identifier, SyntaxKind::ExportKeyword,
            SyntaxKind::DefaultKeyword, SyntaxKind::ClassKeyword, SyntaxKind::StaticKeyword,
            SyntaxKind::PublicKeyword, SyntaxKind::PrivateKeyword, SyntaxKind::ProtectedKeyword,
            SyntaxKind::GetKeyword, SyntaxKind::SetKeyword, SyntaxKind::OpenBracketToken,
            SyntaxKind::AsteriskToken,
        ]),
        vec![FormattingContext::is_end_of_decorator_context_on_same_line],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBeforeNonNullAssertionOperator", any_token(), kind_range(SyntaxKind::ExclamationToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_non_null_assertion_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterNewKeywordOnConstructorSignature", kind_range(SyntaxKind::NewKeyword), kind_range(SyntaxKind::OpenParenToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_constructor_signature_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceLessThanAndNonJSXTypeAnnotation", kind_range(SyntaxKind::LessThanToken), kind_range(SyntaxKind::LessThanToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);

    // ---- 用户可配置规则 ----
    add("SpaceAfterConstructor", kind_range(SyntaxKind::ConstructorKeyword), kind_range(SyntaxKind::OpenParenToken),
        vec![is_insert_space_after_constructor_enabled, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterConstructor", kind_range(SyntaxKind::ConstructorKeyword), kind_range(SyntaxKind::OpenParenToken),
        vec![is_insert_space_after_constructor_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterComma", kind_range(SyntaxKind::CommaToken), any_token(),
        vec![is_insert_space_after_comma_enabled, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_non_jsx_element_or_fragment_context, FormattingContext::is_next_token_not_close_bracket, FormattingContext::is_next_token_not_close_paren],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterComma", kind_range(SyntaxKind::CommaToken), any_token(),
        vec![is_insert_space_after_comma_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_non_jsx_element_or_fragment_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterAnonymousFunctionKeyword", kinds_range(&[SyntaxKind::FunctionKeyword, SyntaxKind::AsteriskToken]), kind_range(SyntaxKind::OpenParenToken),
        vec![is_insert_space_after_anonymous_function_enabled, FormattingContext::is_function_decl_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterAnonymousFunctionKeyword", kinds_range(&[SyntaxKind::FunctionKeyword, SyntaxKind::AsteriskToken]), kind_range(SyntaxKind::OpenParenToken),
        vec![is_insert_space_after_anonymous_function_disabled_or_undef, FormattingContext::is_function_decl_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterKeywordInControl", keywords(), kind_range(SyntaxKind::OpenParenToken),
        vec![is_insert_space_after_keywords_in_control_enabled, FormattingContext::is_control_decl_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterKeywordInControl", keywords(), kind_range(SyntaxKind::OpenParenToken),
        vec![is_insert_space_after_keywords_in_control_disabled_or_undef, FormattingContext::is_control_decl_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterOpenParen", kind_range(SyntaxKind::OpenParenToken), any_token(),
        vec![is_insert_space_nonempty_paren_enabled, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBeforeCloseParen", any_token(), kind_range(SyntaxKind::CloseParenToken),
        vec![is_insert_space_nonempty_paren_enabled, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBetweenOpenParens", kind_range(SyntaxKind::OpenParenToken), kind_range(SyntaxKind::OpenParenToken),
        vec![is_insert_space_nonempty_paren_enabled, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBetweenParens", kind_range(SyntaxKind::OpenParenToken), kind_range(SyntaxKind::CloseParenToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterOpenParen", kind_range(SyntaxKind::OpenParenToken), any_token(),
        vec![is_insert_space_nonempty_paren_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBeforeCloseParen", any_token(), kind_range(SyntaxKind::CloseParenToken),
        vec![is_insert_space_nonempty_paren_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterOpenBracket", kind_range(SyntaxKind::OpenBracketToken), any_token(),
        vec![is_insert_space_nonempty_bracket_enabled, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBeforeCloseBracket", any_token(), kind_range(SyntaxKind::CloseBracketToken),
        vec![is_insert_space_nonempty_bracket_enabled, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBetweenBrackets", kind_range(SyntaxKind::OpenBracketToken), kind_range(SyntaxKind::CloseBracketToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterOpenBracket", kind_range(SyntaxKind::OpenBracketToken), any_token(),
        vec![is_insert_space_nonempty_bracket_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBeforeCloseBracket", any_token(), kind_range(SyntaxKind::CloseBracketToken),
        vec![is_insert_space_nonempty_bracket_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterOpenBrace", kind_range(SyntaxKind::OpenBraceToken), any_token(),
        vec![is_insert_space_nonempty_braces_enabled_or_undef, FormattingContext::is_brace_wrapped_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBeforeCloseBrace", any_token(), kind_range(SyntaxKind::CloseBraceToken),
        vec![is_insert_space_nonempty_braces_enabled_or_undef, FormattingContext::is_brace_wrapped_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterOpenBrace", kind_range(SyntaxKind::OpenBraceToken), any_token(),
        vec![is_insert_space_nonempty_braces_disabled, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBeforeCloseBrace", any_token(), kind_range(SyntaxKind::CloseBraceToken),
        vec![is_insert_space_nonempty_braces_disabled, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceBetweenEmptyBraceBrackets", kind_range(SyntaxKind::OpenBraceToken), kind_range(SyntaxKind::CloseBraceToken),
        vec![is_insert_space_empty_braces_enabled],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBetweenEmptyBraceBrackets2", kind_range(SyntaxKind::OpenBraceToken), kind_range(SyntaxKind::CloseBraceToken),
        vec![is_insert_space_empty_braces_disabled, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterTemplateHeadAndMiddle", kinds_range(&[SyntaxKind::TemplateHead, SyntaxKind::TemplateMiddle]), any_token(),
        vec![is_insert_space_template_braces_enabled, FormattingContext::is_non_jsx_text_context],
        RuleAction::INSERT_SPACE, RuleFlags::CanDeleteNewLines);
    add("SpaceBeforeTemplateMiddleAndTail", any_token(), kinds_range(&[SyntaxKind::TemplateMiddle, SyntaxKind::TemplateTail]),
        vec![is_insert_space_template_braces_enabled, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterTemplateHeadAndMiddle", kinds_range(&[SyntaxKind::TemplateHead, SyntaxKind::TemplateMiddle]), any_token(),
        vec![is_insert_space_template_braces_disabled_or_undef, FormattingContext::is_non_jsx_text_context],
        RuleAction::DELETE_SPACE, RuleFlags::CanDeleteNewLines);
    add("NoSpaceBeforeTemplateMiddleAndTail", any_token(), kinds_range(&[SyntaxKind::TemplateMiddle, SyntaxKind::TemplateTail]),
        vec![is_insert_space_template_braces_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterOpenBraceInJsxExpression", kind_range(SyntaxKind::OpenBraceToken), any_token(),
        vec![is_insert_space_jsx_braces_enabled, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_jsx_expression_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBeforeCloseBraceInJsxExpression", any_token(), kind_range(SyntaxKind::CloseBraceToken),
        vec![is_insert_space_jsx_braces_enabled, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_jsx_expression_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterOpenBraceInJsxExpression", kind_range(SyntaxKind::OpenBraceToken), any_token(),
        vec![is_insert_space_jsx_braces_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_jsx_expression_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBeforeCloseBraceInJsxExpression", any_token(), kind_range(SyntaxKind::CloseBraceToken),
        vec![is_insert_space_jsx_braces_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_jsx_expression_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterSemicolonInFor", kind_range(SyntaxKind::SemicolonToken), any_token(),
        vec![is_insert_space_after_semicolon_in_for_enabled, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_for_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterSemicolonInFor", kind_range(SyntaxKind::SemicolonToken), any_token(),
        vec![is_insert_space_after_semicolon_in_for_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_for_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    let bin_ctx_on = vec![is_insert_space_binary_operators_enabled, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_binary_op_context];
    let bin_ctx_off = vec![is_insert_space_binary_operators_disabled, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_binary_op_context];
    add("SpaceBeforeBinaryOperator", any_token(), binary_operators(),
        bin_ctx_on.clone(), RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterBinaryOperator", binary_operators(), any_token(),
        bin_ctx_on, RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBeforeBinaryOperator", any_token(), binary_operators(),
        bin_ctx_off.clone(), RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterBinaryOperator", binary_operators(), any_token(),
        bin_ctx_off, RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceBeforeOpenParenInFuncDecl", any_token(), kind_range(SyntaxKind::OpenParenToken),
        vec![is_insert_space_before_function_paren_enabled, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_function_decl_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBeforeOpenParenInFuncDecl", any_token(), kind_range(SyntaxKind::OpenParenToken),
        vec![is_insert_space_before_function_paren_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_function_decl_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NewLineBeforeOpenBraceInControl", control_open_brace_left(), kind_range(SyntaxKind::OpenBraceToken),
        vec![is_place_open_brace_newline_control_enabled, FormattingContext::is_control_decl_context, FormattingContext::is_before_multiline_block_context],
        RuleAction::INSERT_NEW_LINE, RuleFlags::CanDeleteNewLines);
    add("NewLineBeforeOpenBraceInFunction", function_open_brace_left(), kind_range(SyntaxKind::OpenBraceToken),
        vec![is_place_open_brace_newline_functions_enabled, FormattingContext::is_function_decl_context, FormattingContext::is_before_multiline_block_context],
        RuleAction::INSERT_NEW_LINE, RuleFlags::CanDeleteNewLines);
    add("NewLineBeforeOpenBraceInTypeScriptDeclWithBlock", type_script_open_brace_left(), kind_range(SyntaxKind::OpenBraceToken),
        vec![is_place_open_brace_newline_functions_enabled, FormattingContext::is_type_script_decl_with_block_context, FormattingContext::is_before_multiline_block_context],
        RuleAction::INSERT_NEW_LINE, RuleFlags::CanDeleteNewLines);
    add("SpaceAfterTypeAssertion", kind_range(SyntaxKind::GreaterThanToken), any_token(),
        vec![is_insert_space_after_type_assertion_enabled, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_type_assertion_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceAfterTypeAssertion", kind_range(SyntaxKind::GreaterThanToken), any_token(),
        vec![is_insert_space_after_type_assertion_disabled_or_undef, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_type_assertion_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceBeforeTypeAnnotation", any_token(), kinds_range(&[SyntaxKind::QuestionToken, SyntaxKind::ColonToken]),
        vec![is_insert_space_before_type_annotation_enabled, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_type_annotation_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("NoSpaceBeforeTypeAnnotation", any_token(), kinds_range(&[SyntaxKind::QuestionToken, SyntaxKind::ColonToken]),
        vec![is_insert_space_before_type_annotation_disabled, FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_type_annotation_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoOptionalSemicolon", kind_range(SyntaxKind::SemicolonToken), any_token_including_eof(),
        vec![is_semicolon_preference_remove, FormattingContext::is_semicolon_deletion_context],
        RuleAction::DELETE_TOKEN, RuleFlags::None);
    add("OptionalSemicolon", any_token(), any_token_including_eof(),
        vec![is_semicolon_preference_insert],
        RuleAction::INSERT_TRAILING_SEMICOLON, RuleFlags::None);

    // ---- 低优先级公共规则 ----
    add("NoSpaceBeforeSemicolon", any_token(), kind_range(SyntaxKind::SemicolonToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceBeforeOpenBraceInControl", control_open_brace_left(), kind_range(SyntaxKind::OpenBraceToken),
        vec![is_place_open_brace_newline_control_disabled_or_same_line, FormattingContext::is_control_decl_context, FormattingContext::is_not_format_on_enter, FormattingContext::is_same_line_token_or_before_block_context],
        RuleAction::INSERT_SPACE, RuleFlags::CanDeleteNewLines);
    add("SpaceBeforeOpenBraceInFunction", function_open_brace_left(), kind_range(SyntaxKind::OpenBraceToken),
        vec![is_place_open_brace_newline_functions_disabled_or_same_line, FormattingContext::is_function_decl_context, FormattingContext::is_before_block_context, FormattingContext::is_not_format_on_enter, FormattingContext::is_same_line_token_or_before_block_context],
        RuleAction::INSERT_SPACE, RuleFlags::CanDeleteNewLines);
    add("SpaceBeforeOpenBraceInTypeScriptDeclWithBlock", type_script_open_brace_left(), kind_range(SyntaxKind::OpenBraceToken),
        vec![is_place_open_brace_newline_functions_disabled_or_same_line, FormattingContext::is_type_script_decl_with_block_context, FormattingContext::is_not_format_on_enter, FormattingContext::is_same_line_token_or_before_block_context],
        RuleAction::INSERT_SPACE, RuleFlags::CanDeleteNewLines);
    add("NoSpaceBeforeComma", any_token(), kind_range(SyntaxKind::CommaToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceBeforeOpenBracket", any_token_except(&[SyntaxKind::AsyncKeyword, SyntaxKind::CaseKeyword]), kind_range(SyntaxKind::OpenBracketToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("NoSpaceAfterCloseBracket", kind_range(SyntaxKind::CloseBracketToken), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_not_before_block_in_function_declaration_context],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceAfterSemicolon", kind_range(SyntaxKind::SemicolonToken), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBetweenForAndAwaitKeyword", kind_range(SyntaxKind::ForKeyword), kind_range(SyntaxKind::AwaitKeyword),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceBetweenDotDotDotAndTypeName", kind_range(SyntaxKind::DotDotDotToken), type_names(),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::DELETE_SPACE, RuleFlags::None);
    add("SpaceBetweenStatements", kinds_range(&[SyntaxKind::CloseParenToken, SyntaxKind::DoKeyword, SyntaxKind::ElseKeyword, SyntaxKind::CaseKeyword]), any_token(),
        vec![FormattingContext::is_non_jsx_same_line_token_context, FormattingContext::is_non_jsx_element_or_fragment_context, FormattingContext::is_not_for_context],
        RuleAction::INSERT_SPACE, RuleFlags::None);
    add("SpaceAfterTryCatchFinally", kinds_range(&[SyntaxKind::TryKeyword, SyntaxKind::CatchKeyword, SyntaxKind::FinallyKeyword]), kind_range(SyntaxKind::OpenBraceToken),
        vec![FormattingContext::is_non_jsx_same_line_token_context as crate::format::rule::ContextPredicate],
        RuleAction::INSERT_SPACE, RuleFlags::None);

    rules
}

fn is_insert_space_before_type_annotation_enabled(context: &mut crate::format::rule_context::FormattingContext) -> bool {
    context.options.insert_space_before_type_annotation
}

fn is_insert_space_before_type_annotation_disabled(context: &mut crate::format::rule_context::FormattingContext) -> bool {
    !context.options.insert_space_before_type_annotation
}
