use std::collections::HashMap;

use crate::{
    ast::{EnumVariant, Expression, Parameter, PatternMatchDirective, Statement, Type},
    compiler::Compiler,
    error::{HarmonyError, HarmonyErrorKind},
    token::{SourceLocation, TokenKind},
};

type EnumId = usize;
type FunctionId = usize;

#[derive(Debug, Clone)]
pub struct Scope {
    pub parent: Option<Box<Scope>>,
    pub module: Option<(String, SourceLocation)>,
    pub imports: Vec<Import>,
    pub enum_names: HashMap<String, EnumId>,
    pub enum_variants: HashMap<String, EnumId>,
    pub enums: HashMap<EnumId, Enum>,
    pub function_names: HashMap<String, FunctionId>,
    pub functions: HashMap<FunctionId, Function>,
}

#[derive(Debug, Clone)]
pub struct LocalScope {
    pub variables: HashMap<String, Variable>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            parent: None,
            module: None,
            imports: Vec::new(),
            enum_names: HashMap::new(),
            enum_variants: HashMap::new(),
            enums: HashMap::new(),
            function_names: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn merge(&mut self, other: &Scope) {
        self.imports.extend(other.imports.clone());
        for (name, id) in other.enum_names.clone() {
            let old_id: usize = id;
            if !self.enum_names.contains_key(&name) {
                let id: usize = self.enums.len();
                self.enum_names.insert(name, id);
                self.enums
                    .insert(id, other.enums.get(&old_id).unwrap().clone());
                for variant in other.enums.get(&old_id).unwrap().variants.clone() {
                    self.enum_variants.insert(
                        match variant {
                            EnumVariant::Unit(name, _) => name,
                            EnumVariant::Tuple(name, _, _) => name,
                        },
                        id,
                    );
                }
            }
        }
        for (name, id) in other.function_names.clone() {
            let old_id: usize = id;
            if !self.function_names.contains_key(&name) {
                let id: usize = self.functions.len();
                self.function_names.insert(name, id);
                self.functions
                    .insert(id, other.functions.get(&old_id).unwrap().clone());
            }
        }
    }
}

impl LocalScope {
    pub fn new() -> LocalScope {
        LocalScope {
            variables: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Checker {
    pub compiler: Compiler,
    pub statements: Vec<Statement>,
    pub filename: String,
    pub global_scope: Scope,
}

impl Checker {
    pub fn new(compiler: &Compiler, statements: &Vec<Statement>, filename: &String) -> Checker {
        Checker {
            compiler: compiler.clone(),
            statements: statements.clone(),
            filename: filename.clone(),
            global_scope: Scope::new(),
        }
    }

    pub fn analyze(&mut self) -> Result<(), HarmonyError> {
        self.lookup_module_decl()?;
        // self.lookup_imports();
        self.check_statements()?;

        Ok(())
    }

    fn lookup_module_decl(&mut self) -> Result<(), HarmonyError> {
        for statement in &self.statements {
            match statement {
                Statement::Module { name, exposing: _ } => {
                    let full_name = name
                        .iter()
                        .map(|(name, _)| name.to_string())
                        .collect::<Vec<String>>()
                        .join(".");
                    let location: SourceLocation = name.first().unwrap().1.clone();
                    self.global_scope.module = Some((full_name.clone(), location.clone()));
                }
                _ => {}
            }
        }

        if self.global_scope.module.is_none() {
            return Err(HarmonyError::new(
                HarmonyErrorKind::CompileTime,
                "Module declaration not found".to_string(),
                None,
                SourceLocation {
                    file: self.filename.clone(),
                    line: 0,
                    column: 0,
                    length: 0,
                },
            ));
        }

        let module_name: String = self.global_scope.module.as_ref().unwrap().0.clone();
        let module_name: Vec<&str> = module_name.split(".").collect::<Vec<&str>>();
        let module_name: String = module_name.last().unwrap().to_string();
        let filename: String = self.filename.clone().replace("\\", "/");
        let filename: Vec<&str> = filename.split("/").collect::<Vec<&str>>();
        let filename: String = filename.last().unwrap().to_string();
        let filename: String = filename
            .split(".")
            .collect::<Vec<&str>>()
            .first()
            .unwrap()
            .to_string();
        if module_name != filename {
            return Err(HarmonyError::new(
                HarmonyErrorKind::CompileTime,
                format!(
                    "Module name '{}' does not match filename '{}'",
                    module_name, filename
                ),
                None,
                SourceLocation {
                    file: self.filename.clone(),
                    line: 0,
                    column: 0,
                    length: 0,
                },
            ));
        }

        Ok(())
    }

    pub fn lookup_imports(&mut self) -> Result<Vec<Import>, HarmonyError> {
        let mut imports: Vec<Import> = Vec::new();
        for statement in &self.statements {
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
                    let alias: Option<String> = match alias {
                        Some((name, _)) => Some(name.to_string()),
                        None => None,
                    };
                    let import: Import = Import {
                        name: full_name.clone(),
                        alias: alias.clone(),
                        exposing: exposing
                            .iter()
                            .map(|(name, _)| name.to_string())
                            .collect::<Vec<String>>(),
                    };
                    imports.push(import);
                }
                _ => {}
            }
        }
        Ok(imports)
    }

    pub fn get_path_of_import(&mut self, import: &Import) -> Result<String, HarmonyError> {
        let mut path: Vec<&str> = import.name.split(".").collect::<Vec<&str>>();
        let module: &str = path.pop().unwrap();
        let mut module_path: String = self.compiler.root.to_str().unwrap().to_string();
        module_path = module_path.replace("\\", "/");
        module_path.push_str("/runtime");
        for dir in path {
            module_path.push_str("/");
            module_path.push_str(dir);
        }
        module_path.push_str("/");
        module_path.push_str(module);
        module_path.push_str(".harm");
        if !std::path::Path::new(&module_path).exists() {
            module_path = self.compiler.root.to_str().unwrap().to_string();
            module_path.push_str("/");
            module_path.push_str(self.filename.as_str());
            module_path = module_path.replace("\\", "/");
            let mut path: Vec<&str> = module_path.split("/").collect::<Vec<&str>>();
            path.pop();
            module_path = path.join("/");
            module_path.push_str("/");
            module_path.push_str(module);
            module_path.push_str(".harm");
            if !std::path::Path::new(&module_path).exists() {
                return Err(HarmonyError::new(
                    HarmonyErrorKind::CompileTime,
                    format!("Module {} not found", import.name),
                    None,
                    self.global_scope.module.as_ref().unwrap().1.clone(),
                ));
            }
        }
        Ok(module_path)
    }

    fn check_statements(&mut self) -> Result<(), HarmonyError> {
        for statement in self.statements.clone() {
            self.check_statement(&statement)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<(), HarmonyError> {
        match statement {
            Statement::Module {
                name: _,
                exposing: _,
            } => Ok(()),
            Statement::Import {
                name: _,
                alias: _,
                exposing: _,
            } => Ok(()),
            Statement::ForeignImport {
                name: _,
                exposing: _,
            } => Ok(()),
            Statement::ForeignFunction {
                name,
                parameters,
                return_type,
                binding: _,
            } => {
                let function_id: FunctionId = self.global_scope.functions.len();
                let (name, location) = name.clone();

                let mut local_scope: LocalScope = LocalScope::new();
                let mut parameter_types: Vec<Type> = Vec::new();
                for parameter in parameters {
                    let parameter_name: String = parameter.name.clone().0.clone();
                    let parameter_type: Type = parameter.type_.clone();
                    parameter_types.push(parameter_type.clone());
                    local_scope.variables.insert(
                        parameter_name.clone(),
                        Variable {
                            name: parameter_name.clone(),
                            type_: parameter_type,
                            location: parameter.name.clone().1,
                            value: None,
                        },
                    );
                }

                let return_type: Type = match return_type.clone() {
                    Some(type_) => type_,
                    None => Type::Unit(location.clone()),
                };

                let function: Function = Function {
                    name: name.clone(),
                    generic_parameters: vec![],
                    parameters: parameters.clone(),
                    return_type: return_type.clone(),
                    body: None,
                    location: location.clone(),
                    local_scope: local_scope.clone(),
                    is_external: true,
                };
                self.global_scope
                    .function_names
                    .insert(name.clone(), function_id);
                self.global_scope.functions.insert(function_id, function);

                Ok(())
            }
            Statement::Function {
                name,
                parameters,
                return_type,
                body,
            } => {
                let function_id: FunctionId = self.global_scope.functions.len();
                let (name, location) = name.clone();

                let mut local_scope: LocalScope = LocalScope::new();
                let mut parameter_types: Vec<Type> = Vec::new();
                for parameter in parameters {
                    let parameter_name: String = parameter.name.clone().0.clone();
                    let parameter_type: Type = parameter.type_.clone();
                    parameter_types.push(parameter_type.clone());
                    local_scope.variables.insert(
                        parameter_name.clone(),
                        Variable {
                            name: parameter_name.clone(),
                            type_: parameter_type,
                            location: parameter.name.clone().1,
                            value: None,
                        },
                    );
                }

                let return_type: Type = match return_type.clone() {
                    Some(type_) => type_,
                    None => Type::Unit(location.clone()),
                };
                let body: Expression = body.clone();

                let function: Function = Function {
                    name: name.clone(),
                    generic_parameters: vec![],
                    parameters: parameters.clone(),
                    return_type: return_type.clone(),
                    body: Some(body.clone()),
                    location: location.clone(),
                    local_scope: local_scope.clone(),
                    is_external: false,
                };
                self.global_scope
                    .function_names
                    .insert(name.clone(), function_id);
                self.global_scope.functions.insert(function_id, function);

                let body_type: Type = self.check_expression(&body, &mut local_scope)?;
                if body_type != return_type {
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!(
                            "Function '{}' return type '{}' does not match body type '{}'",
                            name, return_type, body_type
                        ),
                        None,
                        location.clone(),
                    ));
                }

                Ok(())
            }
            Statement::GenericFunction {
                name,
                generic_parameters,
                parameters,
                return_type,
                body,
            } => {
                let mut generic_parameters_: Vec<String> = vec![];
                for generic_parameter in generic_parameters {
                    generic_parameters_.push(match generic_parameter {
                        Type::GenericParameter(name, _) => name.clone(),
                        _ => {
                            return Err(HarmonyError::new(
                                HarmonyErrorKind::Semantic,
                                format!(
                                    "Expected generic parameter, found '{}'",
                                    generic_parameter
                                ),
                                None,
                                generic_parameter.location(),
                            ))
                        }
                    });
                }

                let function_id: FunctionId = self.global_scope.functions.len();
                let (name, location) = name.clone();

                let mut local_scope: LocalScope = LocalScope::new();
                let mut parameter_types: Vec<Type> = Vec::new();
                for parameter in parameters {
                    let parameter_name: String = parameter.name.clone().0.clone();
                    let parameter_type: Type = parameter.type_.clone();
                    let ty: Type = self
                        .check_type_generic(parameter_type.clone(), generic_parameters_.clone())?;
                    if !generic_parameters.contains(&ty) {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Generic parameter '{}' not found in function '{}'",
                                ty, name
                            ),
                            None,
                            location.clone(),
                        ));
                    }
                    parameter_types.push(ty.clone());
                    local_scope.variables.insert(
                        parameter_name.clone(),
                        Variable {
                            name: parameter_name.clone(),
                            type_: ty,
                            location: parameter.name.clone().1,
                            value: None,
                        },
                    );
                }

                let return_type: Type = match return_type.clone() {
                    Some(type_) => self.check_type_generic(type_, generic_parameters_)?,
                    None => Type::Unit(location.clone()),
                };
                let body: Expression = body.clone();

                let function: Function = Function {
                    name: name.clone(),
                    generic_parameters: generic_parameters.clone(),
                    parameters: parameters.clone(),
                    return_type: return_type.clone(),
                    body: Some(body.clone()),
                    location: location.clone(),
                    local_scope: local_scope.clone(),
                    is_external: false,
                };
                self.global_scope
                    .function_names
                    .insert(name.clone(), function_id);
                self.global_scope.functions.insert(function_id, function);

                let body_type: Type = self.check_expression(&body, &mut local_scope)?;
                if body_type != return_type {
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!(
                            "Function '{}' return type '{}' does not match body type '{}'",
                            name, return_type, body_type
                        ),
                        None,
                        location.clone(),
                    ));
                }

                Ok(())
            }
            Statement::Enum { name, variants } => {
                let enum_id: EnumId = self.global_scope.enums.len();
                let (name, location) = name.clone();

                let mut variant_names: Vec<String> = Vec::new();
                for variant in variants {
                    match variant {
                        EnumVariant::Unit(name, _) => {
                            variant_names.push(name.clone());
                        }
                        EnumVariant::Tuple(name, _, _) => {
                            variant_names.push(name.clone());
                        }
                    }
                }

                let enum_: Enum = Enum {
                    name: name.clone(),
                    variants: variants.clone(),
                    location: location.clone(),
                    generic_parameters: vec![],
                };
                self.global_scope.enum_names.insert(name.clone(), enum_id);
                self.global_scope.enums.insert(enum_id, enum_);
                for name in variant_names {
                    self.global_scope
                        .enum_variants
                        .insert(name.clone(), enum_id);
                }

                Ok(())
            }
            Statement::GenericEnum {
                name,
                generic_parameters,
                variants,
            } => {
                let enum_id: EnumId = self.global_scope.enums.len();
                let (name, location) = name.clone();

                let mut variant_names: Vec<String> = Vec::new();
                for variant in variants {
                    match variant {
                        EnumVariant::Unit(name, _) => {
                            variant_names.push(name.clone());
                        }
                        EnumVariant::Tuple(name, _, _) => {
                            variant_names.push(name.clone());
                        }
                    }
                }

                for variant in variants {
                    match variant {
                        EnumVariant::Tuple(_, _, types) => {
                            for type_ in types {
                                match type_ {
                                    Type::GenericParameter(name, _) => {
                                        if !generic_parameters.contains(type_) {
                                            return Err(HarmonyError::new(
                                                HarmonyErrorKind::Semantic,
                                                format!("Generic parameter {} not found", name),
                                                None,
                                                location.clone(),
                                            ));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }

                let enum_: Enum = Enum {
                    name: name.clone(),
                    variants: variants.clone(),
                    location: location.clone(),
                    generic_parameters: generic_parameters.clone(),
                };
                self.global_scope.enum_names.insert(name.clone(), enum_id);
                self.global_scope.enums.insert(enum_id, enum_);
                for name in variant_names {
                    self.global_scope
                        .enum_variants
                        .insert(name.clone(), enum_id);
                }

                Ok(())
            }
        }
    }

    fn check_type_generic(
        &mut self,
        ty: Type,
        generic_parameters: Vec<String>,
    ) -> Result<Type, HarmonyError> {
        match ty {
            Type::Identifier(name, location) if generic_parameters.contains(&name) => {
                Ok(Type::GenericParameter(name, location))
            }
            Type::List(inner) => Ok(Type::List(Some(Box::new(
                self.check_type_generic(*inner.unwrap(), generic_parameters)?,
            )))),
            _ => Ok(ty),
        }
    }

    pub fn check_expression(
        &mut self,
        expression: &Expression,
        local_scope: &mut LocalScope,
    ) -> Result<Type, HarmonyError> {
        match expression {
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left_type: Type = self.check_expression(left, local_scope)?;
                let right_type: Type = self.check_expression(right, local_scope)?;
                match operator {
                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Asterisk
                    | TokenKind::Slash
                    | TokenKind::Percent => {
                        if let Type::Int(loc1) = left_type.clone() {
                            if let Type::Int(loc2) = right_type {
                                return Ok(Type::Int(loc1.merge(&loc2)));
                            }
                        }
                        if let Type::Float(loc1) = left_type.clone() {
                            if let Type::Float(loc2) = right_type {
                                return Ok(Type::Float(loc1.merge(&loc2)));
                            }
                        }
                        if let Type::Int(loc1) = left_type.clone() {
                            if let Type::Float(loc2) = right_type {
                                return Ok(Type::Float(loc1.merge(&loc2)));
                            }
                        }
                        if let Type::Float(loc1) = left_type.clone() {
                            if let Type::Int(loc2) = right_type {
                                return Ok(Type::Float(loc1.merge(&loc2)));
                            }
                        }
                        if let Type::Char(loc1) = left_type.clone() {
                            if let Type::Char(loc2) = right_type {
                                return Ok(Type::Char(loc1.merge(&loc2)));
                            }
                        }
                        if let Type::GenericParameter(name, loc1) = left_type.clone() {
                            return Ok(Type::GenericParameter(name, loc1));
                        }
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Binary operator '{}' cannot be applied to types '{}' and '{}'",
                                operator,
                                left_type.clone(),
                                right_type
                            ),
                            None,
                            expression.location(),
                        ));
                    }
                    TokenKind::DoubleEquals | TokenKind::NotEquals => {
                        if left_type.clone() == right_type {
                            return Ok(Type::Bool(expression.location().clone()));
                        }
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Binary operator '{}' cannot be applied to types '{}' and '{}'",
                                operator,
                                left_type.clone(),
                                right_type
                            ),
                            None,
                            expression.location(),
                        ));
                    }
                    TokenKind::LessThan
                    | TokenKind::LessThanEquals
                    | TokenKind::GreaterThan
                    | TokenKind::GreaterThanEquals => {
                        if let Type::Int(loc1) = left_type.clone() {
                            if let Type::Int(loc2) = right_type {
                                return Ok(Type::Bool(loc1.merge(&loc2)));
                            }
                        }
                        if let Type::Float(loc1) = left_type.clone() {
                            if let Type::Float(loc2) = right_type {
                                return Ok(Type::Bool(loc1.merge(&loc2)));
                            }
                        }
                        if let Type::Int(loc1) = left_type.clone() {
                            if let Type::Float(loc2) = right_type {
                                return Ok(Type::Bool(loc1.merge(&loc2)));
                            }
                        }
                        if let Type::Float(loc1) = left_type.clone() {
                            if let Type::Int(loc2) = right_type {
                                return Ok(Type::Bool(loc1.merge(&loc2)));
                            }
                        }
                        if let Type::Char(loc1) = left_type.clone() {
                            if let Type::Char(loc2) = right_type {
                                return Ok(Type::Bool(loc1.merge(&loc2)));
                            }
                        }
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Binary operator '{}' cannot be applied to types '{}' and '{}'",
                                operator,
                                left_type.clone(),
                                right_type
                            ),
                            None,
                            expression.location(),
                        ));
                    }
                    TokenKind::And | TokenKind::Or => {
                        if let Type::Bool(loc1) = left_type.clone() {
                            if let Type::Bool(loc2) = right_type {
                                return Ok(Type::Bool(loc1.merge(&loc2)));
                            }
                        }
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Binary operator '{}' cannot be applied to types '{}' and '{}'",
                                operator,
                                left_type.clone(),
                                right_type
                            ),
                            None,
                            expression.location(),
                        ));
                    }
                    TokenKind::PlusPlus => {
                        if let Type::String(loc1) = left_type.clone() {
                            if let Type::String(loc2) = right_type {
                                return Ok(Type::String(loc1.merge(&loc2)));
                            }
                            if let Type::GenericParameter(_, _) = right_type {
                                return Ok(Type::String(loc1));
                            }
                        }
                        if let Type::List(_) = left_type.clone() {
                            if let Type::List(_) = right_type {
                                return Ok(left_type.clone());
                            }
                        }
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Binary operator '{}' cannot be applied to types '{}' and '{}'",
                                operator, left_type, right_type
                            ),
                            None,
                            expression.location(),
                        ));
                    }
                    _ => Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!(
                            "Binary operator '{}' cannot be applied to types '{}' and '{}'",
                            operator, left_type, right_type
                        ),
                        None,
                        expression.location(),
                    )),
                }
            }
            Expression::Identifier(identifier, location) => {
                if let Some(variable) = local_scope.variables.get(identifier) {
                    return Ok(variable.type_.clone());
                }
                if let Some(enum_id) = self.global_scope.enum_variants.get(identifier) {
                    let enum_: Enum = self.global_scope.enums.get(&enum_id).unwrap().clone();
                    let name: String = enum_.name.clone();
                    if enum_.generic_parameters.len() > 0 {
                        return Ok(Type::GenericEnum(
                            name,
                            location.clone(),
                            enum_.generic_parameters,
                        ));
                    }
                    return Ok(Type::Enum(name, location.clone()));
                }
                if let Some(function_id) = self.global_scope.function_names.get(identifier) {
                    let function: Function = self
                        .global_scope
                        .functions
                        .get(&function_id)
                        .unwrap()
                        .clone();
                    let types: Vec<Type> = function
                        .parameters
                        .iter()
                        .map(|parameter| parameter.type_.clone())
                        .collect();
                    return Ok(Type::Function(types, Box::new(function.return_type)));
                }
                return Err(HarmonyError::new(
                    HarmonyErrorKind::Semantic,
                    format!("Variable '{}' is not defined", identifier),
                    None,
                    location.clone(),
                ));
            }
            Expression::Char(_, location) => Ok(Type::Char(location.clone())),
            Expression::Integer(value, location) => {
                if *value > i32::MAX as i64 || *value < i32::MIN as i64 {
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("Integer literal '{}' is too large", value),
                        Some("Consider using the BigInt type instead".to_string()),
                        location.clone(),
                    ));
                }
                Ok(Type::Int(location.clone()))
            }
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_type = self.check_expression(condition, &mut local_scope.clone())?;
                if let Type::Bool(_) = condition_type {
                    let then_type = self.check_expression(then_branch, &mut local_scope.clone())?;
                    let else_type = self.check_expression(else_branch, &mut local_scope.clone())?;
                    if then_type == else_type {
                        return Ok(then_type);
                    }
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!(
                            "If statement branches have mismatched types '{}' and '{}'",
                            then_type, else_type
                        ),
                        None,
                        expression.location(),
                    ));
                }
                return Err(HarmonyError::new(
                    HarmonyErrorKind::Semantic,
                    format!(
                        "If statement condition has type '{}', expected 'Bool'",
                        condition_type
                    ),
                    None,
                    expression.location(),
                ));
            }
            Expression::Call {
                callee,
                generic_arguments,
                arguments,
            } => {
                let callee: String = callee.0.clone();
                if !self.global_scope.function_names.contains_key(&callee) {
                    if self.global_scope.enum_variants.contains_key(&callee) {
                        let enum_id: EnumId = self
                            .global_scope
                            .enum_variants
                            .get(&callee)
                            .unwrap()
                            .clone();
                        let enum_: Enum = self.global_scope.enums.get(&enum_id).unwrap().clone();
                        let name: String = enum_.name.clone();
                        // TODO: make sure to only add the arguments if the call expression is coming
                        //       from a pattern match expression.
                        for (i, argument) in arguments.iter().enumerate() {
                            if let Expression::Identifier(id, location) = argument.clone() {
                                for variant in enum_.variants.clone() {
                                    println!("{:?}", variant);
                                    if let EnumVariant::Tuple(name, _, types) = variant {
                                        if name != callee {
                                            continue;
                                        }
                                        let ty: Type = types.get(i).unwrap().clone();
                                        local_scope.variables.insert(
                                            id.clone(),
                                            Variable {
                                                name: id.clone(),
                                                type_: ty,
                                                location: location.clone(),
                                                value: None,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                        if enum_.generic_parameters.len() > 0 {
                            return Ok(Type::GenericEnum(
                                name,
                                expression.location().clone(),
                                enum_.generic_parameters,
                            ));
                        }
                        return Ok(Type::Enum(name, expression.location().clone()));
                    }
                    if local_scope.variables.contains_key(&callee) {
                        let variable: Variable =
                            local_scope.variables.get(&callee).unwrap().clone();
                        if let Type::Function(types, return_type) = variable.type_ {
                            if types.len() != arguments.len() {
                                return Err(HarmonyError::new(
                                    HarmonyErrorKind::Semantic,
                                    format!(
                                        "Function '{}' expects {} arguments, found {}",
                                        callee,
                                        types.len(),
                                        arguments.len()
                                    ),
                                    None,
                                    expression.location(),
                                ));
                            }
                            for (i, argument) in arguments.iter().enumerate() {
                                let argument_type =
                                    self.check_expression(argument, &mut local_scope.clone())?;
                                if argument_type != types.get(i).unwrap().clone() {
                                    return Err(HarmonyError::new(
                                        HarmonyErrorKind::Semantic,
                                        format!(
                                            "Function '{}' expects argument {} to have type '{}', found '{}'",
                                            callee,
                                            i + 1,
                                            types.get(i).unwrap().clone(),
                                            argument_type
                                        ),
                                        None,
                                        expression.location(),
                                    ));
                                }
                            }
                            return Ok(*return_type.clone());
                        }
                    }
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("Function '{}' is not defined", callee),
                        None,
                        expression.location(),
                    ));
                }
                let function_id: FunctionId = self
                    .global_scope
                    .function_names
                    .get(&callee)
                    .unwrap()
                    .clone();
                let callee_type = self
                    .global_scope
                    .functions
                    .get(&function_id)
                    .unwrap()
                    .clone()
                    .return_type
                    .clone();

                if generic_arguments.len() > 0 {
                    let generic_parameters: Vec<Type> = self
                        .global_scope
                        .functions
                        .get(&function_id)
                        .unwrap()
                        .clone()
                        .generic_parameters;
                    if generic_arguments.len() != generic_parameters.len() {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Function '{}' expects {} generic arguments, found {}",
                                callee,
                                generic_parameters.len(),
                                generic_arguments.len()
                            ),
                            None,
                            expression.location(),
                        ));
                    }
                }

                let mut argument_types = Vec::new();
                for argument in arguments {
                    let argument_type =
                        self.check_expression(argument, &mut local_scope.clone())?;
                    argument_types.push(argument_type);
                }

                Ok(callee_type)
            }
            Expression::PatternMatch {
                expression,
                cases,
                default_case,
            } => {
                let expression_type =
                    self.check_expression(expression, &mut local_scope.clone())?;
                let mut case_types = Vec::new();
                let mut case_body_types = Vec::new();
                for case in cases {
                    let mut case_scope = local_scope.clone();

                    if let Expression::List(elements) = &case.pattern {
                        for (_, element) in elements.iter().enumerate() {
                            if let Expression::Identifier(id, location) = element.clone() {
                                let element_type: Type = match expression_type.clone() {
                                    Type::List(inner) => *inner.unwrap(),
                                    _ => {
                                        return Err(HarmonyError::new(
                                            HarmonyErrorKind::Semantic,
                                            format!(
                                                "Pattern match case has type '{}', expected 'List'",
                                                expression_type
                                            ),
                                            None,
                                            element.location(),
                                        ))
                                    }
                                };
                                case_scope.variables.insert(
                                    id.clone(),
                                    Variable {
                                        name: id.clone(),
                                        type_: element_type,
                                        location: location.clone(),
                                        value: None,
                                    },
                                );
                            } else if let Expression::Rest(expr) = element.clone() {
                                if let Expression::Identifier(id, _) = *expr.clone() {
                                    let element_type: Type = match expression_type.clone() {
                                        Type::List(inner) => *inner.unwrap(),
                                        _ => {
                                            return Err(HarmonyError::new(
                                                HarmonyErrorKind::Semantic,
                                                format!(
                                                "Pattern match case has type '{}', expected 'List'",
                                                expression_type
                                            ),
                                                None,
                                                element.location(),
                                            ))
                                        }
                                    };
                                    case_scope.variables.insert(
                                        id.clone(),
                                        Variable {
                                            name: id.clone(),
                                            type_: element_type,
                                            location: element.location().clone(),
                                            value: None,
                                        },
                                    );
                                }
                            } else {
                                return Err(HarmonyError::new(
                                    HarmonyErrorKind::Semantic,
                                    format!(
                                        "Pattern match case has type '{}', expected 'List'",
                                        expression_type
                                    ),
                                    None,
                                    element.location(),
                                ));
                            }
                        }
                    }

                    let case_type = self.check_expression(&case.pattern, &mut case_scope)?;
                    let case_body_type = self.check_expression(&case.body, &mut case_scope)?;
                    if let PatternMatchDirective::If(expression) = &case.directive {
                        let expression_type =
                            self.check_expression(&expression, &mut case_scope.clone())?;
                        if let Type::Bool(_) = expression_type {
                        } else {
                            return Err(HarmonyError::new(
                                HarmonyErrorKind::Semantic,
                                format!(
                                    "Pattern match case directive has type '{}', expected 'bool'",
                                    expression_type
                                ),
                                None,
                                expression.location(),
                            ));
                        }
                    }

                    if case_type != expression_type {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Pattern match case has type '{}', expected '{}'",
                                case_type, expression_type
                            ),
                            None,
                            case.pattern.location(),
                        ));
                    }
                    case_types.push(case_type);
                    case_body_types.push(case_body_type);
                }
                if let Some(default_case) = default_case {
                    let mut default_case_scope = local_scope.clone();
                    let default_case_type =
                        self.check_expression(default_case, &mut default_case_scope)?;
                    if default_case_type != case_body_types[0].clone() {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Pattern match default case has type '{}', expected '{}'",
                                default_case_type,
                                case_body_types[0].clone()
                            ),
                            None,
                            default_case.location(),
                        ));
                    }
                }
                if case_body_types.len() > 0 {
                    return Ok(case_body_types[0].clone());
                }
                return Err(HarmonyError::new(
                    HarmonyErrorKind::Semantic,
                    "Pattern match has no cases".to_string(),
                    None,
                    expression.location(),
                ));
            }
            Expression::Access {
                name: name_,
                member,
            } => {
                let name: String = name_.0.clone();
                let location = name_.1.clone();
                for import in self.global_scope.imports.iter() {
                    if import.alias.is_some() {
                        let alias: String = import.alias.clone().unwrap();
                        if alias == name {
                            return Ok(self.check_expression(&*member, &mut local_scope.clone())?);
                        }
                    } else {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!("Import '{}' doesn't have an alias.", import.name),
                            Some(
                                "This is currently a bug and will be fixed in the future."
                                    .to_string(),
                            ),
                            expression.location(),
                        ));
                    }
                }
                Err(HarmonyError::new(
                    HarmonyErrorKind::Semantic,
                    format!("Import '{}' not found.", name),
                    None,
                    location,
                ))
            }
            Expression::Index { expression, index } => {
                let expression_type =
                    self.check_expression(expression, &mut local_scope.clone())?;
                let index_type = self.check_expression(index, &mut local_scope.clone())?;
                if let Type::String(location) = expression_type {
                    if let Type::Int(_) = index_type {
                        return Ok(Type::Char(location));
                    }
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("String index has type '{}', expected 'Int'", index_type),
                        None,
                        expression.location(),
                    ));
                }
                if let Type::List(_) = expression_type {
                    if let Type::Int(location) = index_type {
                        return Ok(Type::Any(location));
                    }
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("Array index has type '{}', expected 'Int'", index_type),
                        None,
                        expression.location(),
                    ));
                }
                return Err(HarmonyError::new(
                    HarmonyErrorKind::Semantic,
                    format!(
                        "Index expression has type '{}', expected 'String' or 'Array'",
                        expression_type
                    ),
                    None,
                    expression.location(),
                ));
            }
            Expression::Bool(_, _) => Ok(Type::Bool(expression.location().clone())),
            Expression::List(elements) => {
                let mut element_types = Vec::new();
                let mut found_elements: Vec<Expression> = Vec::new();
                for element in elements {
                    if let Expression::Rest(name) = element {
                        // get the rest of the elements
                        let mut rest_elements = Vec::new();
                        let mut found_rest = false;
                        for element in elements {
                            if let Expression::Rest(_) = element {
                                found_rest = true;
                                continue;
                            }
                            if found_rest {
                                rest_elements.push(element.clone());
                            }
                        }
                        let value = Expression::List(rest_elements);
                        let rest_type = self.check_expression(&value, &mut local_scope.clone())?;
                        if let Expression::Identifier(name, loc) = *name.clone() {
                            local_scope.variables.insert(
                                name.clone(),
                                Variable {
                                    name: name.clone(),
                                    type_: rest_type.clone(),
                                    location: loc.clone(),
                                    value: Some(value.clone()),
                                },
                            );
                        }
                        break;
                    }
                    found_elements.push(element.clone());
                    let element_type = self.check_expression(element, &mut local_scope.clone())?;
                    element_types.push(element_type);
                }
                if element_types.len() > 0 {
                    return Ok(Type::List(Some(Box::new(element_types[0].clone()))));
                }
                Ok(Type::List(Some(Box::new(Type::Any(
                    expression.location().clone(),
                )))))
            }
            Expression::Rest(_) => Err(HarmonyError::new(
                HarmonyErrorKind::Semantic,
                "Rest expression not allowed here.".to_string(),
                Some("Rest expressions are only allowed in lists.".to_string()),
                expression.location(),
            )),
            Expression::String(_, _) => Ok(Type::String(expression.location().clone())),
            Expression::Let {
                name,
                type_annotation,
                value,
                body,
            } => {
                let value_type = self.check_expression(value, &mut local_scope.clone())?;
                if let Some(type_annotation) = type_annotation {
                    if type_annotation != &value_type {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Let expression has type '{}', expected '{}'",
                                value_type, type_annotation
                            ),
                            None,
                            expression.location(),
                        ));
                    }
                }
                let name_: String = name.0.clone();
                let location = name.1.clone();
                local_scope.variables.insert(
                    name_.clone(),
                    Variable {
                        name: name_.clone(),
                        type_: value_type.clone(),
                        location: location.clone(),
                        value: Some(*value.clone()),
                    },
                );
                self.check_expression(body, &mut local_scope.clone())
            }
            Expression::Function {
                parameters,
                return_type,
                ..
            } => {
                let mut parameter_types = Vec::new();
                for parameter in parameters {
                    let parameter_type = parameter.type_.clone();
                    parameter_types.push(parameter_type);
                }
                let function_type = Type::Function(
                    parameter_types,
                    Box::new(return_type.clone().unwrap().clone()),
                );
                Ok(function_type)
            }
            _ => {
                dbg!(expression);
                todo!("check_expression")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Import {
    pub name: String,
    pub alias: Option<String>,
    pub exposing: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub location: SourceLocation,
    pub generic_parameters: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub location: SourceLocation,
    pub generic_parameters: Vec<Type>,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Option<Expression>,
    pub local_scope: LocalScope,
    pub is_external: bool,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub location: SourceLocation,
    pub type_: Type,
    pub value: Option<Expression>,
}
