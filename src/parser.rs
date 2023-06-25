use crate::{
    ast::{EnumVariant, Expression, Parameter, PatternMatchCase, Statement, Type},
    error::{HarmonyError, HarmonyErrorKind},
    token::{SourceLocation, Token, TokenKind},
};

#[derive(Debug, Clone)]
pub struct Parser {
    pub tokens: Vec<Token>,
    pub index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, index: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, HarmonyError> {
        let mut statements: Vec<Statement> = Vec::new();

        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, HarmonyError> {
        match self.current()?.kind {
            TokenKind::Module => self.parse_module(),
            TokenKind::Import => self.parse_import(),
            TokenKind::Fun => self.parse_function(),
            TokenKind::Enum => self.parse_enum(),
            _ => Err(HarmonyError::new(
                HarmonyErrorKind::Syntax,
                format!("Expected statement, found {:?}", self.current()?.kind),
                None,
                self.current()?.location,
            )),
        }
    }

    fn parse_module(&mut self) -> Result<Statement, HarmonyError> {
        self.expect(TokenKind::Module)?;
        let mut name: Vec<(String, SourceLocation)> = vec![];

        let location: SourceLocation = self.current()?.location;
        name.push((self.expect(TokenKind::Identifier)?.lexeme, location));
        while !self.is_at_end() && self.current()?.kind == TokenKind::Dot {
            self.expect(TokenKind::Dot)?;
            let location: SourceLocation = self.current()?.location;
            name.push((self.expect(TokenKind::Identifier)?.lexeme, location));
        }

        let mut exposing: Vec<(String, SourceLocation)> = vec![];
        if !self.is_at_end() && self.current()?.kind == TokenKind::Exposing {
            self.expect(TokenKind::Exposing)?;
            self.expect(TokenKind::OpenParenthesis)?;

            let location: SourceLocation = self.current()?.location;
            exposing.push((self.expect(TokenKind::Identifier)?.lexeme, location));
            while !self.is_at_end() && self.current()?.kind == TokenKind::Comma {
                self.expect(TokenKind::Comma)?;
                let location: SourceLocation = self.current()?.location;
                exposing.push((self.expect(TokenKind::Identifier)?.lexeme, location));
            }
            self.expect(TokenKind::CloseParenthesis)?;
        }

        Ok(Statement::Module { name, exposing })
    }

    fn parse_import(&mut self) -> Result<Statement, HarmonyError> {
        self.expect(TokenKind::Import)?;
        let mut name: Vec<(String, SourceLocation)> = vec![];

        let location: SourceLocation = self.current()?.location;
        name.push((self.expect(TokenKind::Identifier)?.lexeme, location));
        while !self.is_at_end() && self.current()?.kind == TokenKind::Dot {
            self.expect(TokenKind::Dot)?;
            let location: SourceLocation = self.current()?.location;
            name.push((self.expect(TokenKind::Identifier)?.lexeme, location));
        }

        let mut alias: Option<(String, SourceLocation)> = None;
        if !self.is_at_end() && self.current()?.kind == TokenKind::As {
            self.expect(TokenKind::As)?;
            let location: SourceLocation = self.current()?.location;
            alias = Some((self.expect(TokenKind::Identifier)?.lexeme, location));
        }

        let mut exposing: Vec<(String, SourceLocation)> = vec![];
        if !self.is_at_end() && self.current()?.kind == TokenKind::Exposing {
            self.expect(TokenKind::Exposing)?;
            self.expect(TokenKind::OpenParenthesis)?;
            let location: SourceLocation = self.current()?.location;
            exposing.push((self.expect(TokenKind::Identifier)?.lexeme, location));
            while !self.is_at_end() && self.current()?.kind == TokenKind::Comma {
                self.expect(TokenKind::Comma)?;
                let location: SourceLocation = self.current()?.location;
                exposing.push((self.expect(TokenKind::Identifier)?.lexeme, location));
            }
            self.expect(TokenKind::CloseParenthesis)?;
        }

        Ok(Statement::Import {
            name,
            alias,
            exposing,
        })
    }

    fn parse_function(&mut self) -> Result<Statement, HarmonyError> {
        self.expect(TokenKind::Fun)?;
        let location: SourceLocation = self.current()?.location;
        let name = self.expect(TokenKind::Identifier)?.lexeme;
        let mut parameters: Vec<Parameter> = vec![];
        let mut return_type: Option<Type> = None;
        if !self.is_at_end() && self.current()?.kind == TokenKind::OpenParenthesis {
            self.expect(TokenKind::OpenParenthesis)?;
            if !self.is_at_end() && self.current()?.kind == TokenKind::Identifier {
                let location: SourceLocation = self.current()?.location;
                let identifier = self.expect(TokenKind::Identifier)?.lexeme;
                self.expect(TokenKind::Colon)?;
                let type_ = self.parse_type()?;
                parameters.push(Parameter {
                    name: (identifier, location),
                    type_,
                });
                while !self.is_at_end() && self.current()?.kind == TokenKind::Comma {
                    self.expect(TokenKind::Comma)?;
                    let location: SourceLocation = self.current()?.location;
                    let identifier = self.expect(TokenKind::Identifier)?.lexeme;
                    self.expect(TokenKind::Colon)?;
                    let type_ = self.parse_type()?;
                    parameters.push(Parameter {
                        name: (identifier, location),
                        type_,
                    });
                }
            }
            self.expect(TokenKind::CloseParenthesis)?;
            self.expect(TokenKind::Arrow)?;
            return_type = Some(self.parse_type()?);
        }
        self.expect(TokenKind::Equals)?;
        let body = self.parse_expression()?;
        Ok(Statement::Function {
            name: (name, location),
            parameters,
            return_type,
            body,
        })
    }

    fn parse_enum(&mut self) -> Result<Statement, HarmonyError> {
        self.expect(TokenKind::Enum)?;
        let location: SourceLocation = self.current()?.location;
        let name = self.expect(TokenKind::Identifier)?.lexeme;
        let mut generic_parameters: Vec<Type> = vec![];
        if !self.is_at_end() && self.current()?.kind == TokenKind::LessThan {
            self.expect(TokenKind::LessThan)?;
            generic_parameters.push(self.parse_type()?);
            while !self.is_at_end() && self.current()?.kind == TokenKind::Comma {
                self.expect(TokenKind::Comma)?;
                generic_parameters.push(self.parse_type()?);
            }
            self.expect(TokenKind::GreaterThan)?;
        }
        let mut variants: Vec<EnumVariant> = vec![];
        self.expect(TokenKind::Equals)?;
        variants.push(self.parse_enum_variant(!generic_parameters.is_empty())?);
        while !self.is_at_end() && self.current()?.kind == TokenKind::Pipe {
            self.expect(TokenKind::Pipe)?;
            variants.push(self.parse_enum_variant(!generic_parameters.is_empty())?);
        }
        if generic_parameters.is_empty() {
            Ok(Statement::Enum {
                name: (name, location),
                variants,
            })
        } else {
            Ok(Statement::GenericEnum {
                name: (name, location),
                generic_parameters,
                variants,
            })
        }
    }

    fn parse_enum_variant(&mut self, is_generic: bool) -> Result<EnumVariant, HarmonyError> {
        let location: SourceLocation = self.current()?.location;
        let name = self.expect(TokenKind::Identifier)?.lexeme;
        if !self.is_at_end() && self.current()?.kind == TokenKind::OpenParenthesis {
            self.expect(TokenKind::OpenParenthesis)?;
            let mut types: Vec<Type> = vec![];
            types.push(match self.parse_type()? {
                Type::Identifier(name, location) => {
                    if is_generic {
                        Type::GenericParameter(name, location)
                    } else {
                        Type::Identifier(name, location)
                    }
                }
                type_ => type_,
            });
            while !self.is_at_end() && self.current()?.kind == TokenKind::Comma {
                self.expect(TokenKind::Comma)?;
                types.push(match self.parse_type()? {
                    Type::Identifier(name, location) => {
                        if is_generic {
                            Type::GenericParameter(name, location)
                        } else {
                            Type::Identifier(name, location)
                        }
                    }
                    type_ => type_,
                });
            }
            self.expect(TokenKind::CloseParenthesis)?;
            return Ok(EnumVariant::Tuple(name, location, types));
        }
        Ok(EnumVariant::Unit(name, location))
    }

    fn parse_expression(&mut self) -> Result<Expression, HarmonyError> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, precedence: u8) -> Result<Expression, HarmonyError> {
        let mut left = self.parse_unary_expression()?;
        while !self.is_at_end() && self.current()?.kind.is_binary_operator() {
            let operator = self.current()?.kind.clone();
            let operator_precedence = operator.precedence();
            if operator_precedence < precedence {
                break;
            }
            self.expect(operator.clone())?;
            let right = self.parse_binary_expression(operator_precedence + 1)?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression, HarmonyError> {
        if !self.is_at_end() && self.current()?.kind.is_unary_operator() {
            let operator = self.current()?.kind.clone();
            self.expect(operator.clone())?;
            let right = self.parse_unary_expression()?;
            Ok(Expression::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.parse_index_expression()
        }
    }

    fn parse_index_expression(&mut self) -> Result<Expression, HarmonyError> {
        let mut expression: Expression = self.parse_primary_expression()?;
        while !self.is_at_end() && self.current()?.kind == TokenKind::OpenBracket {
            self.expect(TokenKind::OpenBracket)?;
            let index = self.parse_expression()?;
            self.expect(TokenKind::CloseBracket)?;
            expression = Expression::Index {
                expression: Box::new(expression),
                index: Box::new(index),
            };
        }
        Ok(expression)
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, HarmonyError> {
        match self.current()?.kind {
            TokenKind::Identifier => {
                let location: SourceLocation = self.current()?.location;
                let identifier = self.expect(TokenKind::Identifier)?.lexeme;
                if !self.is_at_end() && self.current()?.kind == TokenKind::OpenParenthesis {
                    self.expect(TokenKind::OpenParenthesis)?;
                    let mut arguments: Vec<Expression> = vec![];
                    if !self.is_at_end() && self.current()?.kind != TokenKind::CloseParenthesis {
                        arguments.push(self.parse_expression()?);
                        while !self.is_at_end() && self.current()?.kind == TokenKind::Comma {
                            self.expect(TokenKind::Comma)?;
                            arguments.push(self.parse_expression()?);
                        }
                    }
                    self.expect(TokenKind::CloseParenthesis)?;
                    Ok(Expression::Call {
                        callee: (identifier, location),
                        arguments,
                    })
                } else if !self.is_at_end() && self.current()?.kind == TokenKind::Dot {
                    self.expect(TokenKind::Dot)?;
                    let member = self.parse_primary_expression()?;
                    Ok(Expression::Access {
                        name: (identifier, location),
                        member: Box::new(member),
                    })
                } else {
                    Ok(Expression::Identifier(identifier, location))
                }
            }
            TokenKind::IntegerLiteral => {
                let location: SourceLocation = self.current()?.location;
                let value = self.expect(TokenKind::IntegerLiteral)?.lexeme;
                Ok(Expression::Integer(value.parse::<i64>().unwrap(), location))
            }
            TokenKind::FloatLiteral => {
                let location: SourceLocation = self.current()?.location;
                let value = self.expect(TokenKind::FloatLiteral)?.lexeme;
                Ok(Expression::Float(value.parse::<f64>().unwrap(), location))
            }
            TokenKind::StringLiteral => {
                let location: SourceLocation = self.current()?.location;
                let value = self.expect(TokenKind::StringLiteral)?.lexeme;
                Ok(Expression::String(value, location))
            }
            TokenKind::CharacterLiteral => {
                let location: SourceLocation = self.current()?.location;
                let value = self.expect(TokenKind::CharacterLiteral)?.lexeme;
                Ok(Expression::Char(value.parse::<char>().unwrap(), location))
            }
            TokenKind::BooleanLiteral => {
                let location: SourceLocation = self.current()?.location;
                let value = self.expect(TokenKind::BooleanLiteral)?.lexeme;
                Ok(Expression::Bool(value.parse::<bool>().unwrap(), location))
            }
            TokenKind::Case => {
                self.expect(TokenKind::Case)?;
                let expression: Expression = self.parse_expression()?;
                self.expect(TokenKind::Of)?;
                let mut cases: Vec<PatternMatchCase> = vec![];
                let mut default_case: Option<Box<Expression>> = None;
                while !self.is_at_end() && self.current()?.kind == TokenKind::Pipe {
                    self.expect(TokenKind::Pipe)?;
                    if !self.is_at_end() && self.current()?.kind == TokenKind::Else {
                        self.expect(TokenKind::Else)?;
                        self.expect(TokenKind::FatArrow)?;
                        default_case = Some(Box::new(self.parse_expression()?));
                        break;
                    }
                    let pattern: Expression = self.parse_expression()?;
                    self.expect(TokenKind::FatArrow)?;
                    let body: Expression = self.parse_expression()?;
                    cases.push(PatternMatchCase { pattern, body });
                }
                self.expect(TokenKind::End)?; // TODO: Make this optional
                Ok(Expression::PatternMatch {
                    expression: Box::new(expression),
                    cases,
                    default_case,
                })
            }
            TokenKind::OpenBracket => {
                self.expect(TokenKind::OpenBracket)?;
                let mut elements: Vec<Expression> = vec![];
                if !self.is_at_end() && self.current()?.kind != TokenKind::CloseBracket {
                    elements.push(self.parse_expression()?);
                    while !self.is_at_end() && self.current()?.kind == TokenKind::Comma {
                        self.expect(TokenKind::Comma)?;
                        elements.push(self.parse_expression()?);
                    }
                }
                self.expect(TokenKind::CloseBracket)?;
                Ok(Expression::List(elements))
            }
            TokenKind::If => {
                self.expect(TokenKind::If)?;
                let condition: Expression = self.parse_expression()?;
                self.expect(TokenKind::Then)?;
                let then_branch: Expression = self.parse_expression()?;
                self.expect(TokenKind::Else)?;
                let else_branch: Expression = self.parse_expression()?;
                Ok(Expression::If {
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                })
            }
            TokenKind::DoubleDot => {
                self.expect(TokenKind::DoubleDot)?;
                let expression: Expression = self.parse_expression()?;
                Ok(Expression::Rest(Box::new(expression)))
            }
            _ => Err(HarmonyError::new(
                HarmonyErrorKind::Syntax,
                format!("Expected expression, found {:?}", self.current()?.kind),
                None,
                self.current()?.location,
            )),
        }
    }

    fn parse_type(&mut self) -> Result<Type, HarmonyError> {
        match self.current()?.kind {
            TokenKind::Identifier => {
                let location: SourceLocation = self.current()?.location;
                let identifier = self.expect(TokenKind::Identifier)?.lexeme;
                if !self.is_at_end() && self.current()?.kind == TokenKind::LessThan {
                    self.expect(TokenKind::LessThan)?;
                    let mut types: Vec<Type> = vec![];
                    types.push(self.parse_type()?);
                    while !self.is_at_end() && self.current()?.kind == TokenKind::Comma {
                        self.expect(TokenKind::Comma)?;
                        types.push(self.parse_type()?);
                    }
                    self.expect(TokenKind::GreaterThan)?;
                    Ok(Type::Generic(identifier, location, types))
                } else {
                    Ok(Type::Identifier(identifier, location))
                }
            }
            TokenKind::Int => {
                let location: SourceLocation = self.current()?.location;
                self.expect(TokenKind::Int)?;
                Ok(Type::Int(location))
            }
            TokenKind::Float => {
                let location: SourceLocation = self.current()?.location;
                self.expect(TokenKind::Float)?;
                Ok(Type::Float(location))
            }
            TokenKind::String => {
                let location: SourceLocation = self.current()?.location;
                self.expect(TokenKind::String)?;
                Ok(Type::String(location))
            }
            TokenKind::Bool => {
                let location: SourceLocation = self.current()?.location;
                self.expect(TokenKind::Bool)?;
                Ok(Type::Bool(location))
            }
            TokenKind::Char => {
                let location: SourceLocation = self.current()?.location;
                self.expect(TokenKind::Char)?;
                Ok(Type::Char(location))
            }
            TokenKind::OpenBracket => {
                self.expect(TokenKind::OpenBracket)?;
                let inner_type: Type = self.parse_type()?;
                self.expect(TokenKind::CloseBracket)?;
                Ok(Type::List(Some(Box::new(inner_type))))
            }
            _ => Err(HarmonyError::new(
                HarmonyErrorKind::Syntax,
                format!("Expected type, found {:?}", self.current()?.kind),
                None,
                self.current()?.location,
            )),
        }
    }

    fn current(&self) -> Result<Token, HarmonyError> {
        if self.index >= self.tokens.len() {
            return Err(HarmonyError::new(
                HarmonyErrorKind::Syntax,
                "Unexpected end of file".to_string(),
                None,
                self.tokens[self.tokens.len() - 1].clone().location,
            ));
        }
        Ok(self.tokens[self.index].clone())
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, HarmonyError> {
        if self.current()?.kind == kind {
            let token = self.current()?;
            self.index += 1;
            Ok(token)
        } else {
            Err(HarmonyError::new(
                HarmonyErrorKind::Syntax,
                format!("Expected {:?}, found {:?}", kind, self.current()?.kind),
                None,
                self.current()?.location,
            ))
        }
    }

    fn is_at_end(&self) -> bool {
        self.index >= self.tokens.len()
    }
}
