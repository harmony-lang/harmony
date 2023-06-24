use std::collections::HashMap;

use crate::{
    ast::{EnumVariant, Expression, Parameter, Statement, Type},
    compiler::Compiler,
    error::{HarmonyError, HarmonyErrorKind},
    token::SourceLocation,
};

type EnumId = usize;
type FunctionId = usize;

#[derive(Debug, Clone)]
pub struct Scope {
    pub parent: Option<Box<Scope>>,
    pub module: Option<(String, SourceLocation)>,
    pub imports: Vec<Import>,
    pub enum_names: HashMap<String, EnumId>,
    pub enums: HashMap<EnumId, Enum>,
    pub function_names: HashMap<String, FunctionId>,
    pub functions: HashMap<FunctionId, Function>,
    pub variable_names: HashMap<String, Type>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            parent: None,
            module: None,
            imports: Vec::new(),
            enum_names: HashMap::new(),
            enums: HashMap::new(),
            function_names: HashMap::new(),
            functions: HashMap::new(),
            variable_names: HashMap::new(),
        }
    }

    pub fn new_child(&mut self) -> Scope {
        Scope {
            parent: Some(Box::new(self.clone())),
            module: None,
            imports: Vec::new(),
            enum_names: HashMap::new(),
            enums: HashMap::new(),
            function_names: HashMap::new(),
            functions: HashMap::new(),
            variable_names: HashMap::new(),
        }
    }

    pub fn is_global_scope(&self) -> bool {
        self.parent.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct Checker {
    pub compiler: Compiler,
    pub statements: Vec<Statement>,
    pub filename: String,
    pub scope: Scope,
}

impl Checker {
    pub fn new(compiler: &Compiler, statements: &Vec<Statement>, filename: &String) -> Checker {
        Checker {
            compiler: compiler.clone(),
            statements: statements.clone(),
            filename: filename.clone(),
            scope: Scope::new(),
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
                    self.scope.module = Some((full_name.clone(), location.clone()));
                }
                _ => {}
            }
        }

        if self.scope.module.is_none() {
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

        let module_name: String = self.scope.module.as_ref().unwrap().0.clone();
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
                    self.scope.module.as_ref().unwrap().1.clone(),
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
            Statement::Enum { name, variants } => {
                let enum_id: EnumId = self.scope.enums.len();
                let (name, location) = name.clone();
                for variant in variants {
                    match variant {
                        EnumVariant::Unit(name, location) => {
                            self.scope.variable_names.insert(
                                name.clone(),
                                Type::Identifier(name.clone(), location.clone()),
                            );
                        }
                        EnumVariant::Tuple(name, location, _) => {
                            self.scope.variable_names.insert(
                                name.clone(),
                                Type::Identifier(name.clone(), location.clone()),
                            );
                        }
                    }
                }
                let enum_: Enum = Enum {
                    name: name.clone(),
                    variants: variants.clone(),
                    location: location.clone(),
                };
                self.scope.enums.insert(enum_id, enum_);
                self.scope.enum_names.insert(name.clone(), enum_id.clone());

                Ok(())
            }
            Statement::Function {
                name,
                parameters,
                return_type,
                body,
            } => {
                let function_id: FunctionId = self.scope.functions.len();
                let (name, location) = name.clone();

                for parameter in parameters {
                    self.scope
                        .variable_names
                        .insert(parameter.name.clone().0.clone(), parameter.type_.clone());
                }

                let function: Function = Function {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    return_type: match return_type.clone() {
                        Some(return_type) => return_type,
                        None => Type::Unit(location.clone()),
                    },
                    body: body.clone(),
                    location: location.clone(),
                };
                self.scope.functions.insert(function_id, function);
                self.scope
                    .function_names
                    .insert(name.clone(), function_id.clone());

                self.check_expression(&body)?;

                for parameter in parameters {
                    self.scope.variable_names.remove(&parameter.name.clone().0);
                }

                Ok(())
            }
            Statement::GenericEnum { name, variants, .. } => {
                let enum_id: EnumId = self.scope.enums.len();
                let (name, location) = name.clone();
                let enum_: Enum = Enum {
                    name: name.clone(),
                    variants: variants.clone(),
                    location: location.clone(),
                };
                self.scope.enums.insert(enum_id, enum_);
                self.scope.enum_names.insert(name.clone(), enum_id.clone());

                Ok(())
            } // _ => todo!("check_statement {:?}", statement),
        }
    }

    fn check_expression(&mut self, expression: &Expression) -> Result<Type, HarmonyError> {
        match expression {
            Expression::PatternMatch {
                expression, cases, ..
            } => {
                // 1. Check for ambiguity
                // 2. Check for unreachable cases
                // 3. Check for exhaustiveness
                // 4. Check for duplicate patterns
                // 5. Make sure all pattern conditions match the type of the expression
                // 6. Make sure all pattern expressions match the expected output type
                // 7. Make sure all pattern expressions are of the same type

                let expression_type: Type = self.check_expression(&expression)?;

                let mut case_condition_types: Vec<Type> = Vec::new();
                let mut case_expression_types: Vec<Type> = Vec::new();
                for case in cases {
                    let case_condition_type: Type = self.check_expression(&case.pattern)?;
                    let case_expression_type: Type = self.check_expression(&case.body)?;
                    case_condition_types.push(case_condition_type);
                    case_expression_types.push(case_expression_type);
                }

                for case_condition_type in &case_condition_types {
                    if case_condition_type != &expression_type {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::CompileTime,
                            format!(
                                "Pattern condition type {:?} does not match expression type {:?}",
                                case_condition_type, expression_type
                            ),
                            None,
                            expression.location(),
                        ));
                    }
                }

                Ok(case_expression_types[0].clone())
            }
            Expression::Access { name, member } => {
                let mut found: bool = false;
                for import in &self.scope.imports {
                    let import_name: String = import.name.clone();
                    if import.alias.is_some() {
                        if import.alias.clone().unwrap() == name.0 {
                            found = true;
                            break;
                        }
                    } else {
                        if import_name == name.0 {
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("Module {} not found", name.0),
                        None,
                        expression.location(),
                    ));
                }

                let module_name: String = name.0.clone();
                for file_path in self.compiler.compiled_files.keys() {
                    let file_name: String = file_path
                        .split("/")
                        .last()
                        .unwrap()
                        .to_string()
                        .split(".")
                        .next()
                        .unwrap()
                        .to_string();
                    if file_name == module_name {
                        let file_scope: Scope = self.compiler.compiled_files[file_path].clone();
                        if let Expression::Call { callee, arguments } = *member.clone() {
                            let name: String = callee.clone().0;
                            if !file_scope.function_names.contains_key(&name) {
                                return Err(HarmonyError::new(
                                    HarmonyErrorKind::Semantic,
                                    format!(
                                        "Function {} not found in module {}",
                                        name, module_name
                                    ),
                                    None,
                                    expression.location(),
                                ));
                            }
                            let function_id: FunctionId = file_scope.function_names[&name].clone();
                            let function: &Function = &file_scope.functions[&function_id];
                            let function_return_type: Type = function.return_type.clone();
                            let mut argument_types: Vec<Type> = Vec::new();
                            for argument in arguments {
                                let argument_type: Type = self.check_expression(&argument)?;
                                argument_types.push(argument_type);
                            }
                            if argument_types.len() != function.parameters.len() {
                                return Err(HarmonyError::new(
                                    HarmonyErrorKind::Semantic,
                                    format!(
                                        "Function {} expects {} arguments, but {} were provided",
                                        name,
                                        function.parameters.len(),
                                        argument_types.len()
                                    ),
                                    None,
                                    expression.location(),
                                ));
                            }
                            for (index, argument_type) in argument_types.iter().enumerate() {
                                let parameter_type: Type = function.parameters[index].type_.clone();
                                if argument_type != &parameter_type {
                                    return Err(HarmonyError::new(
                                        HarmonyErrorKind::Semantic,
                                        format!(
                                            "Function {} expects argument {} to be of type {:?}, but {:?} was provided",
                                            name,
                                            index,
                                            parameter_type,
                                            argument_type
                                        ),
                                        None,
                                        expression.location(),
                                    ));
                                }
                            }
                            return Ok(function_return_type);
                        }
                    }
                }
                let file_scope: Option<Scope> = None;
                if file_scope.is_none() {
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("Failed to locate file scope for module {}", module_name),
                        None,
                        expression.location(),
                    ));
                }
                let file_scope: Scope = file_scope.unwrap();
                if let Expression::Call { callee, arguments } = *member.clone() {
                    let name: String = callee.clone().0;
                    if !file_scope.function_names.contains_key(&name) {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!("Function {} not found in module {}", name, module_name),
                            None,
                            expression.location(),
                        ));
                    }
                    let function_id: FunctionId = file_scope.function_names[&name].clone();
                    let function: &Function = &file_scope.functions[&function_id];
                    let callee_type: Type = function.return_type.clone();
                    let mut argument_types: Vec<Type> = Vec::new();
                    for argument in arguments {
                        argument_types.push(self.clone().check_expression(&argument)?);
                    }
                    if argument_types.len() != function.parameters.len() {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Function {} expects {} arguments, but {} were provided",
                                name,
                                function.parameters.len(),
                                argument_types.len()
                            ),
                            None,
                            expression.location(),
                        ));
                    }
                    for (index, argument_type) in argument_types.iter().enumerate() {
                        if argument_type != &function.parameters[index].type_.clone() {
                            return Err(HarmonyError::new(
                                HarmonyErrorKind::Semantic,
                                format!(
                                    "Function {} expects argument {} to be of type {:?}, but {:?} was provided",
                                    name,
                                    index,
                                    function.parameters[index],
                                    argument_type
                                ),
                                None,
                                expression.location(),
                            ));
                        }
                    }
                    Ok(callee_type)
                } else {
                    Ok(self.check_expression(*&member)?)
                }
            }
            Expression::Call { callee, arguments } => {
                let name: String = callee.clone().0;
                if !self.scope.function_names.contains_key(&name) {
                    for import in &self.scope.imports {
                        let name: String = import.clone().name;
                        for file in self.compiler.compiled_files.keys() {
                            let filename: String = file.split("/").last().unwrap().to_string();
                            let filename: String = filename.split(".").next().unwrap().to_string();
                            let name: String = name.split(".").last().unwrap().to_string();
                            if filename == name {
                                let file_scope: Scope =
                                    self.compiler.compiled_files.get(file).unwrap().clone();
                                for enum_ in file_scope.enums.values() {
                                    let variants: Vec<EnumVariant> = enum_.variants.clone();
                                    for variant in variants {
                                        match variant {
                                            EnumVariant::Unit(enum_member_name, _)
                                            | EnumVariant::Tuple(enum_member_name, _, _) => {
                                                if enum_member_name == callee.0.clone() {
                                                    return Ok(Type::Enum(
                                                        enum_.name.clone(),
                                                        enum_.location.clone(),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    for enum_ in self.scope.enums.values() {
                        let variants: Vec<EnumVariant> = enum_.variants.clone();
                        for variant in variants {
                            match variant {
                                EnumVariant::Unit(enum_member_name, _)
                                | EnumVariant::Tuple(enum_member_name, _, _) => {
                                    if enum_member_name == callee.0.clone() {
                                        return Ok(Type::Enum(
                                            enum_.name.clone(),
                                            enum_.location.clone(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("Function {} not found", name),
                        None,
                        expression.location(),
                    ));
                }
                let function_id: FunctionId = self.scope.function_names[&name].clone();
                let function: &Function = &self.scope.functions[&function_id];
                let callee_type: Type = function.return_type.clone();
                let mut argument_types: Vec<Type> = Vec::new();
                for argument in arguments {
                    argument_types.push(self.clone().check_expression(&argument)?);
                }
                if argument_types.len() != function.parameters.len() {
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!(
                            "Expected {} arguments, found {}",
                            function.parameters.len(),
                            argument_types.len()
                        ),
                        None,
                        expression.location(),
                    ));
                }
                for (index, argument_type) in argument_types.iter().enumerate() {
                    let parameter_type: Type = function.parameters[index].clone().type_;
                    if argument_type != &parameter_type {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Expected argument type {:?}, found {:?}",
                                parameter_type, argument_type
                            ),
                            None,
                            expression.location(),
                        ));
                    }
                }
                Ok(callee_type)
            }
            Expression::Identifier(name, location) => {
                let callee: String = name.clone();
                if !self.scope.variable_names.contains_key(&name.clone()) {
                    for enum_ in self.scope.enums.values() {
                        let variants: Vec<EnumVariant> = enum_.variants.clone();
                        for variant in variants {
                            match variant {
                                EnumVariant::Unit(enum_member_name, _)
                                | EnumVariant::Tuple(enum_member_name, _, _) => {
                                    if enum_member_name == callee.clone() {
                                        return Ok(Type::Enum(
                                            enum_.name.clone(),
                                            enum_.location.clone(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    for import in &self.scope.imports {
                        let name: String = import.clone().name;
                        for file in self.compiler.compiled_files.keys() {
                            let filename: String = file.split("/").last().unwrap().to_string();
                            let filename: String = filename.split(".").next().unwrap().to_string();
                            let name: String = name.split(".").last().unwrap().to_string();
                            if filename == name {
                                let file_scope: Scope =
                                    self.compiler.compiled_files.get(file).unwrap().clone();
                                for enum_ in file_scope.enums.values() {
                                    let variants: Vec<EnumVariant> = enum_.variants.clone();
                                    for variant in variants {
                                        match variant {
                                            EnumVariant::Unit(enum_member_name, _)
                                            | EnumVariant::Tuple(enum_member_name, _, _) => {
                                                if enum_member_name == callee {
                                                    return Ok(Type::Enum(
                                                        enum_.name.clone(),
                                                        enum_.location.clone(),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("Variable {} not found", name.clone()),
                        None,
                        location.clone(),
                    ));
                }
                Ok(self.scope.variable_names[&name.clone()].clone())
            }
            Expression::Binary { left, right, .. } => {
                let left_type: Type = self.check_expression(&*left)?;
                let right_type: Type = self.check_expression(&*right)?;
                if left_type != right_type {
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("Expected left type {:?}, found {:?}", left_type, right_type),
                        None,
                        expression.location(),
                    ));
                }
                Ok(Type::Bool(
                    left.location().clone().merge(&right.location().clone()),
                ))
            }
            Expression::Char(_, location) => Ok(Type::Char(location.clone())),
            Expression::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let condition_type: Type = self.check_expression(&*condition)?;
                if condition_type != Type::Bool(condition.location().clone()) {
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!(
                            "Expected condition type {:?}, found {:?}",
                            Type::Bool(condition.location().clone()),
                            condition_type
                        ),
                        None,
                        expression.location(),
                    ));
                }
                let then_type: Type = self.check_expression(&*then_branch)?;
                let else_type: Type = self.check_expression(&*else_branch)?;
                if then_type != else_type {
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("Expected then type {:?}, found {:?}", then_type, else_type),
                        None,
                        expression.location(),
                    ));
                }
                Ok(then_type)
            }
            Expression::Integer(value, location) => {
                if value.clone() > i64::MAX {
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("Integer {} is too large", value),
                        Some(
                            "Consider using the BigInt type instead (Found in Data.BigInt)"
                                .to_string(),
                        ),
                        location.clone(),
                    ));
                } else if value.clone() < i64::MIN {
                    return Err(HarmonyError::new(
                        HarmonyErrorKind::Semantic,
                        format!("Integer {} is too small", value),
                        Some(
                            "Consider using the BigInt type instead (Found in Data.BigInt)"
                                .to_string(),
                        ),
                        location.clone(),
                    ));
                }
                Ok(Type::Int(location.clone()))
            }
            Expression::List(elements) => {
                let mut element_types: Vec<Type> = Vec::new();
                let mut rest: Option<String> = None;
                for element in elements {
                    if rest.is_some() {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            "Rest expression must be last in list".to_string(),
                            None,
                            element.location(),
                        ));
                    }
                    if let Expression::Rest(id) = element.clone() {
                        if rest.is_some() {
                            return Err(HarmonyError::new(
                                HarmonyErrorKind::Semantic,
                                "Only one rest expression is allowed per list".to_string(),
                                None,
                                id.location(),
                            ));
                        }
                        if let Expression::Identifier(name, location) = *id.clone() {
                            rest = Some(name.clone());
                            self.scope
                                .variable_names
                                .insert(name.clone(), Type::List(None));
                            let rest_type: Type = self.scope.variable_names[&name.clone()].clone();
                            if let Type::List(_) = rest_type {
                                element_types.push(rest_type);
                            } else {
                                return Err(HarmonyError::new(
                                    HarmonyErrorKind::Semantic,
                                    format!(
                                        "Expected rest type {:?}, found {:?}",
                                        Type::List(None),
                                        rest_type
                                    ),
                                    None,
                                    location.clone(),
                                ));
                            }
                        } else {
                            return Err(HarmonyError::new(
                                HarmonyErrorKind::Semantic,
                                "Rest expressions must be identifiers".to_string(),
                                None,
                                id.location(),
                            ));
                        }
                        continue;
                    }
                    element_types.push(self.clone().check_expression(&element)?);
                }
                if element_types.is_empty() {
                    return Ok(Type::List(None));
                }
                let element_type: Type = element_types[0].clone();
                for element in element_types {
                    if let Type::List(None) = element {
                        if rest.is_some() {
                            continue;
                        }
                    }
                    if element != element_type {
                        return Err(HarmonyError::new(
                            HarmonyErrorKind::Semantic,
                            format!(
                                "Expected element type {:?}, found {:?}",
                                element_type, element
                            ),
                            None,
                            expression.location(),
                        ));
                    }
                }
                Ok(Type::List(Some(Box::new(element_type))))
            }
            Expression::Rest(expr) => Err(HarmonyError::new(
                HarmonyErrorKind::Semantic,
                "Rest expressions are not allowed outside of list expressions".to_string(),
                None,
                expr.location(),
            )),
            _ => todo!("check_expression {:?}", expression),
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
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Expression,
    pub location: SourceLocation,
}
