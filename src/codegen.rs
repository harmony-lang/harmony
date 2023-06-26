use crate::{
    ast::{EnumVariant, Expression, Statement},
    checker::Checker,
    token::TokenKind,
};

#[derive(Debug, Clone)]
pub struct Codegen {
    pub statements: Vec<Statement>,
    pub checker: Checker,
    pub names: Vec<String>,
}

impl Codegen {
    pub fn new(statements: &Vec<Statement>, checker: &Checker) -> Codegen {
        Codegen {
            statements: statements.clone(),
            checker: checker.clone(),
            names: Vec::new(),
        }
    }

    pub fn generate(&mut self) -> String {
        let mut code: String = String::new();

        for statement in self.statements.clone() {
            code.push_str(self.generate_statement(&statement).as_str());
        }

        code
    }

    pub fn generate_statement(&mut self, statement: &Statement) -> String {
        let mut code: String = String::new();
        match statement {
            Statement::Import {
                name,
                alias,
                exposing,
            } => {
                let full_name: String = name
                    .iter()
                    .map(|(name, _)| name.to_string())
                    .collect::<Vec<String>>()
                    .join(".");
                let mut imported: Vec<String> = Vec::new();
                for import in self.checker.global_scope.imports.clone() {
                    if import.name == full_name.clone() && !imported.contains(&full_name.clone()) {
                        let full_path = self
                            .checker
                            .get_path_of_import(&import)
                            .unwrap()
                            .replace(".harm", ".mjs");
                        if exposing.len() > 0 {
                            code.push_str(
                                format!(
                                    "import {{ {} }} from \"file:///{}\";\n",
                                    exposing
                                        .iter()
                                        .map(|(name, _)| name.to_string())
                                        .collect::<Vec<String>>()
                                        .join(", "),
                                    full_path
                                )
                                .as_str(),
                            );
                        } else {
                            code.push_str(
                                format!(
                                    "import * as {} from \"file:///{}\";\n",
                                    alias.clone().unwrap().0.clone(),
                                    full_path
                                )
                                .as_str(),
                            );
                        }
                        imported.push(full_name.clone());
                    }
                }
            }
            Statement::Enum { name, variants } => {
                code.push_str(format!("export const {} = {{\n", name.0.clone()).as_str());
                for variant in variants {
                    match variant {
                        EnumVariant::Unit(name, _) => {
                            self.names.push(name.clone());
                            code.push_str(
                                format!("    {}: {{ \"{}\": undefined }},\n", name, name).as_str(),
                            );
                        }
                        EnumVariant::Tuple(name, _, types) => {
                            self.names.push(name.clone());
                            let args: Vec<String> = types
                                .iter()
                                .enumerate()
                                .map(|(i, _)| format!("value{}", i))
                                .collect();
                            code.push_str(
                                format!("    {}: ({}) => ({{\n", name, args.join(", ")).as_str(),
                            );
                            for (i, _) in types.iter().enumerate() {
                                code.push_str(
                                    format!("        \"{}{}\": value{},\n", name, i, i).as_str(),
                                );
                            }
                            code.push_str("    }),\n");
                        }
                    }
                }
                code.push_str("};\n");
                for variant in variants.clone() {
                    let name1: String = name.clone().0;
                    match variant {
                        EnumVariant::Unit(name, _) | EnumVariant::Tuple(name, _, _) => {
                            code.push_str(
                                format!("export const {} = {}.{};\n", name, name1.clone(), name)
                                    .as_str(),
                            );
                        }
                    }
                }
            }
            Statement::GenericEnum {
                name,
                generic_parameters: _,
                variants,
            } => {
                code.push_str(format!("export const {} = {{\n", name.0.clone()).as_str());
                for variant in variants {
                    match variant {
                        EnumVariant::Unit(name, _) => {
                            self.names.push(name.clone());
                            code.push_str(
                                format!("    {}: {{ \"{}\": undefined }},\n", name, name).as_str(),
                            );
                        }
                        EnumVariant::Tuple(name, _, types) => {
                            self.names.push(name.clone());
                            let args: Vec<String> = types
                                .iter()
                                .enumerate()
                                .map(|(i, _)| format!("value{}", i))
                                .collect();
                            code.push_str(
                                format!("    {}: ({}) => ({{\n", name, args.join(", ")).as_str(),
                            );
                            for (i, _) in types.iter().enumerate() {
                                code.push_str(
                                    format!("        \"{}{}\": value{},\n", name, i, i).as_str(),
                                );
                            }
                            code.push_str("    }),\n");
                        }
                    }
                }
                code.push_str("};\n");
                for variant in variants.clone() {
                    let name1: String = name.clone().0;
                    match variant {
                        EnumVariant::Unit(name, _) | EnumVariant::Tuple(name, _, _) => {
                            code.push_str(
                                format!("export const {} = {}.{};\n", name, name1.clone(), name)
                                    .as_str(),
                            );
                        }
                    }
                }
            }
            Statement::ExternFunction {
                name,
                parameters,
                return_type: _,
                binding,
            } => {
                let name: String = name.clone().0;
                self.names.push(name.clone());
                let binding: String = binding.clone().0;
                let mut args: Vec<String> = Vec::new();
                for (i, _) in parameters.iter().enumerate() {
                    args.push(format!("arg{}", i));
                }
                code.push_str(
                    format!("export var {} = ({}) => {{\n", name, args.join(", ")).as_str(),
                );
                let mut placeholders: Vec<String> = Vec::new();
                binding.find("%").map(|index| {
                    let mut binding: String = binding.clone();
                    binding.replace_range(index..index + 1, "arg");
                    placeholders.push(binding);
                });
                if placeholders.len() == 0 {
                    code.push_str(
                        format!("    return {}({});\n", binding, args.join(", ")).as_str(),
                    );
                } else {
                    code.push_str(
                        format!("    return {};\n", placeholders.join(", ").replace("%", ""))
                            .as_str(),
                    );
                }
                code.push_str("}\n");
            }
            Statement::Function {
                name,
                parameters,
                return_type: _,
                body,
            } => {
                let name: String = name.clone().0;
                self.names.push(name.clone());
                if name == "main" && parameters.len() == 0 {
                    code.push_str("var main = (() => {\n");
                    code.push_str(
                        format!("    return {};\n", self.generate_expression(body)).as_str(),
                    );
                    code.push_str("})();\n");
                    return code;
                }
                let mut args: Vec<String> = Vec::new();
                for parameter in parameters {
                    args.push(format!("{}", parameter.name.0.clone()));
                }
                code.push_str(
                    format!("export var {} = ({}) => {{\n", name, args.join(", ")).as_str(),
                );
                code.push_str(format!("    return {};\n", self.generate_expression(body)).as_str());
                code.push_str("}\n");
            }
            _ => {}
        }
        code
    }

    fn generate_expression(&mut self, expression: &Expression) -> String {
        match expression {
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                if let Expression::List(_) = *right.clone() {
                    if operator == &TokenKind::PlusPlus {
                        let left: String = self.generate_expression(left);
                        let right: String = self.generate_expression(right);
                        return format!("{}.concat({})", left, right);
                    }
                }
                let left: String = self.generate_expression(left);
                let right: String = self.generate_expression(right);
                match operator {
                    TokenKind::Plus => format!("{} + {}", left, right),
                    TokenKind::Minus => format!("{} - {}", left, right),
                    TokenKind::Asterisk => format!("{} * {}", left, right),
                    TokenKind::Slash => format!("{} / {}", left, right),
                    TokenKind::Percent => format!("{} % {}", left, right),
                    TokenKind::DoubleEquals => format!("{} === {}", left, right),
                    TokenKind::NotEquals => format!("{} !== {}", left, right),
                    TokenKind::LessThan => format!("{} < {}", left, right),
                    TokenKind::LessThanEquals => format!("{} <= {}", left, right),
                    TokenKind::GreaterThan => format!("{} > {}", left, right),
                    TokenKind::GreaterThanEquals => format!("{} >= {}", left, right),
                    TokenKind::And => format!("{} && {}", left, right),
                    TokenKind::Or => format!("{} || {}", left, right),
                    TokenKind::PlusPlus => format!("{} + {}", left, right),
                    _ => unreachable!(),
                }
            }
            Expression::Identifier(id, _) => id.clone(),
            Expression::Char(value, _) => format!("'{}'", value),
            Expression::Integer(value, _) => value.clone().to_string(),
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition: String = self.generate_expression(condition);
                let then_branch: String = self.generate_expression(then_branch);
                let else_branch: String = self.generate_expression(else_branch);
                format!("{} ? {} : {}", condition, then_branch, else_branch)
            }
            Expression::Call { callee, arguments } => {
                let callee: String = callee.0.clone();
                let mut args: Vec<String> = Vec::new();
                for arg in arguments {
                    args.push(self.generate_expression(arg));
                }
                format!("{}({})", callee, args.join(", "))
            }
            Expression::Index { expression, index } => {
                let expression: String = self.generate_expression(expression);
                let index: String = self.generate_expression(index);
                format!("{}[{}]", expression, index)
            }
            Expression::PatternMatch {
                expression,
                cases,
                default_case,
            } => {
                let mut code: String = String::new();
                code.push_str("(() => {\n");
                code.push_str(
                    format!(
                        "    const __condition = {};\n",
                        self.generate_expression(expression)
                    )
                    .as_str(),
                );

                for case in cases {
                    let pattern: Expression = case.pattern.clone();
                    let body: Expression = case.body.clone();

                    if let Expression::Call { callee, arguments } = pattern.clone() {
                        if self
                            .checker
                            .global_scope
                            .enum_variants
                            .contains_key(&callee.0)
                        {
                            let mut conds: Vec<String> = Vec::new();
                            for (i, _) in arguments.iter().enumerate() {
                                conds.push(format!("__condition.{}{} !== undefined", callee.0, i));
                            }
                            code.push_str(format!("    if ({}) {{\n", conds.join(" && ")).as_str());
                            for (i, argument) in arguments.iter().enumerate() {
                                code.push_str(
                                    format!(
                                        "        const {} = __condition.{}{};\n",
                                        self.generate_expression(argument),
                                        callee.0,
                                        i
                                    )
                                    .as_str(),
                                );
                            }
                        } else {
                            code.push_str(
                                format!(
                                    "    if (__condition === {}) {{\n",
                                    self.generate_expression(&pattern)
                                )
                                .as_str(),
                            );
                        }
                    } else if let Expression::Identifier(id, _) = pattern.clone() {
                        if self.checker.global_scope.enum_variants.contains_key(&id) {
                            code.push_str(
                                format!("    if (__condition.{} === undefined) {{\n", id.clone())
                                    .as_str(),
                            );
                        } else {
                            code.push_str(
                                format!(
                                    "    if (__condition === {}) {{\n",
                                    self.generate_expression(&pattern)
                                )
                                .as_str(),
                            );
                        }
                    } else {
                        code.push_str(
                            format!(
                                "    if (__condition === {}) {{\n",
                                self.generate_expression(&pattern)
                            )
                            .as_str(),
                        );
                    }

                    code.push_str(
                        format!("        return {};\n", self.generate_expression(&body)).as_str(),
                    );
                    code.push_str("    }\n");
                }

                if default_case.is_none() {
                    code.push_str("    throw new Error(\"Pattern match failed\");\n");
                } else {
                    code.push_str(
                        format!(
                            "    return {};\n",
                            self.generate_expression(default_case.as_ref().unwrap())
                        )
                        .as_str(),
                    );
                }

                code.push_str("})()");
                code
            }
            Expression::Access { name, member } => {
                let name: String = name.0.clone();
                let member: String = self.generate_expression(&*member);
                format!("{}.{}", name, member)
            }
            Expression::Bool(value, _) => value.clone().to_string(),
            Expression::List(elements) => {
                let mut code: String = String::new();
                code.push_str("[");
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        code.push_str(", ");
                    }
                    code.push_str(self.generate_expression(element).as_str());
                }
                code.push_str("]");
                code
            }
            Expression::String(value, _) => format!("\"{}\"", value),
            Expression::Rest(name) => {
                let name: String = self.generate_expression(&*name);
                format!("...{}", name)
            }
            Expression::Let {
                name,
                type_annotation: _,
                value,
                body,
            } => {
                let name: String = name.0.clone();
                let value: String = self.generate_expression(value);
                let body: String = self.generate_expression(body);
                format!("(({}) => {})({})", name, body, value)
            }
            _ => todo!("generate_expression not implemented for {:?}", expression),
        }
    }
}
