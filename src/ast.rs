use crate::token::{SourceLocation, TokenKind};

#[derive(Debug, Clone)]
pub enum Statement {
    Module {
        name: Vec<(String, SourceLocation)>,
        exposing: Vec<(String, SourceLocation)>,
    },
    Import {
        name: Vec<(String, SourceLocation)>,
        alias: Option<(String, SourceLocation)>,
        exposing: Vec<(String, SourceLocation)>,
    },
    Function {
        name: (String, SourceLocation),
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Expression,
    },
    Enum {
        name: (String, SourceLocation),
        variants: Vec<EnumVariant>,
    },
    GenericEnum {
        name: (String, SourceLocation),
        generic_parameters: Vec<Type>,
        variants: Vec<EnumVariant>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: TokenKind,
        right: Box<Expression>,
    },
    Unary {
        operator: TokenKind,
        right: Box<Expression>,
    },
    Call {
        callee: (String, SourceLocation),
        arguments: Vec<Expression>,
    },
    Identifier(String, SourceLocation),
    Integer(i64, SourceLocation),
    Float(f64, SourceLocation),
    String(String, SourceLocation),
    Bool(bool, SourceLocation),
    Char(char, SourceLocation),
    PatternMatch {
        expression: Box<Expression>,
        cases: Vec<PatternMatchCase>,
        default_case: Option<Box<Expression>>,
    },
    List(Vec<Expression>),
    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },
    Access {
        name: (String, SourceLocation),
        member: Box<Expression>,
    },
    Rest(Box<Expression>),
    Index {
        expression: Box<Expression>,
        index: Box<Expression>,
    },
}

impl Expression {
    pub fn location(&self) -> SourceLocation {
        match self {
            Expression::Binary { left, right, .. } => left.location().merge(&right.location()),
            Expression::Unary { right, .. } => right.location(),
            Expression::Call { callee, .. } => callee.1.clone(),
            Expression::Identifier(_, location) => location.clone(),
            Expression::Integer(_, location) => location.clone(),
            Expression::Float(_, location) => location.clone(),
            Expression::String(_, location) => location.clone(),
            Expression::Bool(_, location) => location.clone(),
            Expression::Char(_, location) => location.clone(),
            Expression::PatternMatch { expression, .. } => expression.location(),
            Expression::List(expressions) => expressions
                .first()
                .map(|expression| expression.location())
                .unwrap_or(SourceLocation::default()),
            Expression::If { condition, .. } => condition.location(),
            Expression::Access { name, member } => name.1.merge(&member.location()),
            Expression::Rest(expression) => expression.location(),
            Expression::Index { expression, index } => {
                expression.location().merge(&index.location())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Unit(SourceLocation),
    Int(SourceLocation),
    Float(SourceLocation),
    String(SourceLocation),
    Bool(SourceLocation),
    Char(SourceLocation),

    Generic(String, SourceLocation, Vec<Type>),

    List(Option<Box<Type>>),

    Function(Vec<Type>, Box<Type>),

    Any(SourceLocation),

    Enum(String, SourceLocation),
    Identifier(String, SourceLocation),
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Unit(_), Type::Unit(_)) => true,
            (Type::Int(_), Type::Int(_)) => true,
            (Type::Float(_), Type::Float(_)) => true,
            (Type::String(_), Type::String(_)) => true,
            (Type::Bool(_), Type::Bool(_)) => true,
            (Type::Char(_), Type::Char(_)) => true,
            (Type::Generic(name1, _, _), Type::Generic(name2, _, _)) => name1 == name2,
            (Type::Generic(name1, _, _), Type::Enum(name2, _)) => name1 == name2,
            (Type::Enum(name1, _), Type::Generic(name2, _, _)) => name1 == name2,
            (Type::List(type1), Type::List(type2)) => {
                if let (Some(type1), Some(type2)) = (type1.as_ref(), type2.as_ref()) {
                    type1 == type2
                } else {
                    true
                }
            }
            (
                Type::Function(parameters1, return_type1),
                Type::Function(parameters2, return_type2),
            ) => parameters1 == parameters2 && return_type1 == return_type2,
            (Type::Identifier(name1, _), Type::Identifier(name2, _)) => name1 == name2,
            (Type::Identifier(name1, _), Type::Enum(name2, _)) => name1 == name2,
            (Type::Enum(name1, _), Type::Identifier(name2, _)) => name1 == name2,
            (Type::Enum(name1, _), Type::Enum(name2, _)) => name1 == name2,
            (Type::Any(_), _) => true,
            (_, Type::Any(_)) => true,
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: (String, SourceLocation),
    pub type_: Type,
}

#[derive(Debug, Clone)]
pub struct PatternMatchCase {
    pub pattern: Expression,
    pub body: Expression,
}

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Unit(String, SourceLocation),
    Tuple(String, SourceLocation, Vec<Type>),
    // Struct(String, SourceLocation, Vec<(String, Type)>),
}
