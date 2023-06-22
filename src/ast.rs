use crate::token::{SourceLocation, TokenKind};

#[derive(Debug, Clone)]
pub enum Statement {
    Module {
        name: Vec<(String, SourceLocation)>, // formatted as Foo.Bar.Baz
        exposing: Vec<(String, SourceLocation)>,
    },
    Import {
        name: Vec<(String, SourceLocation)>, // formatted as Foo.Bar.Baz
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
        else_branch: Option<Box<Expression>>,
    },
    Access {
        name: (String, SourceLocation),
        member: Box<Expression>,
    },
    Rest(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Type {
    Int(SourceLocation),
    Float(SourceLocation),
    String(SourceLocation),
    Bool(SourceLocation),
    Char(SourceLocation),

    Generic(String, SourceLocation, Vec<Type>),

    List(Box<Type>),

    Function(Vec<Type>, Box<Type>),

    Identifier(String, SourceLocation),
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
