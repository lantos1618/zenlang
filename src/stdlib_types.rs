use crate::ast::{AstType, Declaration, StructDefinition};
use crate::error::{CompileError, Result};
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static STDLIB_TYPES: OnceLock<StdlibTypeRegistry> = OnceLock::new();

pub fn stdlib_types() -> &'static StdlibTypeRegistry {
    STDLIB_TYPES.get_or_init(|| {
        StdlibTypeRegistry::load().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load stdlib types: {}", e);
            StdlibTypeRegistry::empty()
        })
    })
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used for future signature checking
pub struct MethodSignature {
    pub receiver_type: String,
    pub method_name: String,
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub is_static: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used for future signature checking
pub struct FunctionSignature {
    pub name: String,
    pub module: String,
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
}

pub struct StdlibTypeRegistry {
    structs: HashMap<String, StructDefinition>,
    struct_types: HashMap<String, AstType>,
    struct_sources: HashMap<String, String>, // Type name -> stdlib relative path
    methods: HashMap<String, MethodSignature>,
    functions: HashMap<String, FunctionSignature>,
}

impl StdlibTypeRegistry {
    fn empty() -> Self {
        Self {
            structs: HashMap::new(),
            struct_types: HashMap::new(),
            struct_sources: HashMap::new(),
            methods: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn load() -> Result<Self> {
        let mut registry = Self::empty();
        let stdlib_root = Self::find_stdlib_root();

        if !stdlib_root.exists() {
            return Ok(registry);
        }

        let files_to_parse = [
            // Core types
            "core/option.zen",
            "core/result.zen",
            "core/iterator.zen",
            // Compiler intrinsics wrappers (user-facing `compiler.*`)
            "compiler.zen",
            // Collections
            "collections/vec.zen",
            "collections/hashmap.zen",
            "collections/set.zen",
            "collections/string.zen",
            "collections/queue.zen",
            "collections/stack.zen",
            // Memory
            "memory/gpa.zen",
            "memory/allocator.zen",
            // Other
            "io/io.zen",
            "math.zen",
            "std.zen",
        ];

        for file in &files_to_parse {
            let path = stdlib_root.join(file);
            if path.exists() {
                let _ = registry.parse_file(&path, file);
            }
        }

        Ok(registry)
    }

    fn find_stdlib_root() -> PathBuf {
        // Check environment variable first
        if let Ok(path) = std::env::var("ZEN_STDLIB_PATH") {
            let p = PathBuf::from(path);
            if p.exists() && p.is_dir() {
                return p;
            }
        }

        // Try relative paths
        let candidates = [
            PathBuf::from("./stdlib"),
            PathBuf::from("../stdlib"),
            PathBuf::from("../../stdlib"),
        ];

        for candidate in candidates {
            if candidate.exists() && candidate.is_dir() {
                return candidate;
            }
        }

        PathBuf::from("./stdlib")
    }

    fn parse_file(&mut self, path: &Path, relative_path: &str) -> Result<()> {
        let source = std::fs::read_to_string(path).map_err(|e| {
            CompileError::InternalError(format!("Failed to read {}: {}", path.display(), e), None)
        })?;

        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let lexer = Lexer::new(&source);
        let mut parser = Parser::new(lexer);

        let program = match parser.parse_program() {
            Ok(p) => p,
            Err(_) => return Ok(()),
        };

        for decl in program.declarations {
            match decl {
                Declaration::Struct(struct_def) => {
                    let name = struct_def.name.clone();
                    let ast_type = self.struct_def_to_ast_type(&struct_def);
                    self.struct_types.insert(name.clone(), ast_type);
                    self.struct_sources
                        .insert(name.clone(), relative_path.to_string());
                    self.structs.insert(name, struct_def);
                }
                Declaration::Function(func) => {
                    self.register_function(&func, module_name);
                }
                Declaration::TraitImplementation(trait_impl) => {
                    for method in &trait_impl.methods {
                        let sig = MethodSignature {
                            receiver_type: trait_impl.type_name.clone(),
                            method_name: method.name.clone(),
                            params: method.args.clone(),
                            return_type: method.return_type.clone(),
                            is_static: method
                                .args
                                .first()
                                .map(|(n, _)| n != "self")
                                .unwrap_or(true),
                        };
                        let key = format!("{}::{}", trait_impl.type_name, method.name);
                        self.methods.insert(key, sig);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn register_function(&mut self, func: &crate::ast::Function, module_name: &str) {
        if let Some((receiver, method)) = func.name.split_once('.') {
            let is_static = func
                .args
                .first()
                .map(|(name, _)| name != "self")
                .unwrap_or(true);

            let sig = MethodSignature {
                receiver_type: receiver.to_string(),
                method_name: method.to_string(),
                params: func.args.clone(),
                return_type: func.return_type.clone(),
                is_static,
            };

            let key = format!("{}::{}", receiver, method);
            self.methods.insert(key, sig);
        } else {
            let sig = FunctionSignature {
                name: func.name.clone(),
                module: module_name.to_string(),
                params: func.args.clone(),
                return_type: func.return_type.clone(),
            };

            let key = format!("{}::{}", module_name, &func.name);
            self.functions.insert(key, sig);
        }
    }

    fn struct_def_to_ast_type(&self, struct_def: &StructDefinition) -> AstType {
        let fields: Vec<(String, AstType)> = struct_def
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.type_.clone()))
            .collect();

        AstType::Struct {
            name: struct_def.name.clone(),
            fields,
        }
    }

    pub fn get_string_type(&self) -> AstType {
        self.struct_types
            .get("String")
            .cloned()
            .unwrap_or_else(Self::fallback_string_type)
    }

    fn fallback_string_type() -> AstType {
        AstType::Struct {
            name: "String".to_string(),
            fields: vec![
                ("data".to_string(), AstType::ptr(AstType::U8)),
                ("len".to_string(), AstType::Usize),
                ("capacity".to_string(), AstType::Usize),
                (
                    "allocator".to_string(),
                    AstType::Generic {
                        name: "Allocator".to_string(),
                        type_args: vec![],
                    },
                ),
            ],
        }
    }

    pub fn is_string_type(name: &str) -> bool {
        name == "String"
    }

    /// Get a struct definition by name from stdlib
    pub fn get_struct_definition(&self, name: &str) -> Option<&StructDefinition> {
        self.structs.get(name)
    }

    /// Get the stdlib relative path for a type (e.g., "core/result.zen" for Result)
    pub fn get_type_source_path(&self, type_name: &str) -> Option<&str> {
        self.struct_sources.get(type_name).map(|s| s.as_str())
    }

    pub fn get_method_signature(&self, receiver: &str, method: &str) -> Option<&MethodSignature> {
        let key = format!("{}::{}", receiver, method);
        self.methods.get(&key)
    }

    pub fn get_method_return_type(&self, receiver: &str, method: &str) -> Option<&AstType> {
        self.get_method_signature(receiver, method)
            .map(|sig| &sig.return_type)
    }

    pub fn get_function_signature(
        &self,
        module: &str,
        func_name: &str,
    ) -> Option<&FunctionSignature> {
        let key = format!("{}::{}", module, func_name);
        self.functions.get(&key)
    }

    pub fn get_function_return_type(&self, module: &str, func_name: &str) -> Option<&AstType> {
        self.get_function_signature(module, func_name)
            .map(|sig| &sig.return_type)
    }

    /// Get struct type by name (returns AstType::Struct with fields)
    pub fn get_struct_type(&self, name: &str) -> Option<AstType> {
        self.struct_types.get(name).cloned()
    }

    /// Check if a type requires an allocator (has an 'allocator' field in its struct definition)
    pub fn requires_allocator(&self, type_name: &str) -> bool {
        if let Some(struct_def) = self.structs.get(type_name) {
            struct_def.fields.iter().any(|f| {
                f.name == "allocator"
                    || matches!(&f.type_, AstType::Generic { name, .. } if name == "Allocator")
            })
        } else {
            false
        }
    }

    /// Check if a type has a constructor that returns an instance of itself (e.g., HashMap.new())
    /// Returns the return type of the constructor if found
    pub fn get_constructor_return_type(&self, type_name: &str) -> Option<&AstType> {
        // Check for Type.new method
        self.get_method_return_type(type_name, "new")
    }

    /// Check if a type is known to be a generic collection type
    /// This is determined by whether the type has a .new() method in stdlib
    pub fn is_known_type(&self, type_name: &str) -> bool {
        self.structs.contains_key(type_name)
            || self
                .methods
                .keys()
                .any(|k| k.starts_with(&format!("{}::", type_name)))
    }
}
