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
            enums: HashMap::new(),
            function_names: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn merge(&mut self, other: &Scope) {
        self.imports.extend(other.imports.clone());
        self.enum_names.extend(other.enum_names.clone());
        self.enums.extend(other.enums.clone());
        self.function_names.extend(other.function_names.clone());
        self.functions.extend(other.functions.clone());
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
                    parameters: parameters.clone(),
                    return_type: return_type.clone(),
                    body: body.clone(),
                    location: location.clone(),
                    local_scope: local_scope.clone(),
                };
                self.global_scope
                    .function_names
                    .insert(name.clone(), function_id);
                self.global_scope.functions.insert(function_id, function);

                Ok(())
            }
            _ => {
                dbg!(statement);
                todo!("check_statement")
            }
        }
    }

    fn check_expression(&mut self, expression: &Expression) -> Result<Type, HarmonyError> {
        match expression {
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
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub location: SourceLocation,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Expression,
    pub local_scope: LocalScope,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub location: SourceLocation,
    pub type_: Type,
    pub value: Option<Expression>,
}
