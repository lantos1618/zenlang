use crate::ast::{AstType, Declaration, StructDefinition};
use crate::error::{CompileError, Result};
use crate::lexer::Lexer;
use crate::name_utils;
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

        // Scan all .zen files in the stdlib directory tree instead of hardcoding a list.
        // This ensures new stdlib modules are automatically picked up.
        Self::scan_zen_files(&stdlib_root, &stdlib_root, &mut registry);

        Ok(registry)
    }

    /// Recursively find and parse all .zen files under a directory.
    fn scan_zen_files(root: &Path, dir: &Path, registry: &mut Self) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::scan_zen_files(root, &path, registry);
            } else if path.extension().is_some_and(|ext| ext == "zen") {
                // Compute relative path for module identification
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                let _ = registry.parse_file(&path, &rel);
            }
        }
    }

    fn find_stdlib_root() -> PathBuf {
        // Use centralized stdlib discovery from stdlib_discovery module
        crate::stdlib_discovery::find_stdlib_root_or_default()
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

            let key = name_utils::method_key(receiver, method);
            self.methods.insert(key, sig);
        } else {
            let sig = FunctionSignature {
                name: func.name.clone(),
                module: module_name.to_string(),
                params: func.args.clone(),
                return_type: func.return_type.clone(),
            };

            let key = name_utils::stdlib_func_key(module_name, &func.name);
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
        let key = name_utils::method_key(receiver, method);
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
        let key = name_utils::stdlib_func_key(module, func_name);
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
                .any(|k| k.starts_with(&format!("{}.", type_name)))
    }

    /// Get all known struct names from stdlib
    pub fn get_all_struct_names(&self) -> Vec<&str> {
        self.structs.keys().map(|s| s.as_str()).collect()
    }

    /// Get all function names from a specific module
    pub fn get_module_function_names(&self, module: &str) -> Vec<&str> {
        let prefix = format!("{}::", module);
        self.functions
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .map(|k| &k[prefix.len()..])
            .collect()
    }

    /// Check if a function exists in a module
    pub fn has_function(&self, module: &str, func_name: &str) -> bool {
        let key = name_utils::stdlib_func_key(module, func_name);
        self.functions.contains_key(&key)
    }

    /// Check if a type is a collection type (has methods like new, push, etc.)
    pub fn is_collection_type(&self, type_name: &str) -> bool {
        let new_key = name_utils::method_key(type_name, "new");
        let push_key = name_utils::method_key(type_name, "push");
        let get_key = name_utils::method_key(type_name, "get");

        self.methods.contains_key(&new_key)
            && (self.methods.contains_key(&push_key) || self.methods.contains_key(&get_key))
    }

    /// Check if a name is a math function from stdlib
    pub fn is_math_function(&self, func_name: &str) -> bool {
        self.has_function("math", func_name)
    }

    /// Check if a type is an allocator-related type from stdlib/memory
    /// Looks for types with allocator-like methods (allocate, deallocate)
    pub fn is_allocator_type(&self, type_name: &str) -> bool {
        let alloc_key = name_utils::method_key(type_name, "allocate");
        let dealloc_key = name_utils::method_key(type_name, "deallocate");
        self.methods.contains_key(&alloc_key) || self.methods.contains_key(&dealloc_key)
    }

    /// Check if a struct type is defined in stdlib
    pub fn has_struct(&self, name: &str) -> bool {
        self.structs.contains_key(name)
    }

    /// Get list of stdlib modules by checking what files were parsed
    pub fn get_modules(&self) -> Vec<&str> {
        // Return modules based on what types we've seen in struct_sources
        let mut modules = std::collections::HashSet::new();
        for path in self.struct_sources.values() {
            if let Some(module) = path.split('/').next() {
                // Remove .zen extension if top-level file
                let module = module.trim_end_matches(".zen");
                modules.insert(module);
            }
        }
        modules.into_iter().collect()
    }
}

// ========================================================================
// ARCHITECTURE VIOLATION: Layer 3 stdlib types with compiler support
// ========================================================================
//
// The functions below identify Layer 3 stdlib types that currently receive
// special treatment in the typechecker. This is an architecture violation
// as described in docs/design/SEPARATION_OF_CONCERNS.md:
//
// - Layer 1: Compiler primitives (i32, bool, etc.) - MUST have compiler support
// - Layer 2: Core abstractions (Option, Result, Ptr) - require limited compiler support
// - Layer 3: Stdlib types (Vec, HashMap, String, etc.) - should NOT need compiler support
//
// Currently, Vec/DynVec, HashMap, and HashSet get special method type inference
// in src/typechecker/inference/calls.rs and src/typechecker/method_types.rs.
// This should be eliminated in Phase 5 (trait system) by:
// 1. Defining traits like Collection<T>, Map<K,V>, etc.
// 2. Implementing trait methods with proper signatures in stdlib
// 3. Using trait resolution instead of hardcoded type checks
//
// Until then, these helpers centralize the string comparisons to make them
// less fragile and easier to track.
//
// Related tech debt: See tech_debt_audit.md sections on:
// - "String-based type identity checks"
// - "Trait system needed for stdlib types"
// ========================================================================

/// Check if a type is HashMap (Layer 3 type with temporary compiler support)
///
/// NOTE: This is an architecture violation. HashMap should not need special
/// compiler treatment - it should be a pure stdlib type once traits are implemented.
#[inline]
pub fn is_hashmap(type_name: &str) -> bool {
    type_name == "HashMap"
}

/// Check if a type is HashSet (Layer 3 type with temporary compiler support)
///
/// NOTE: This is an architecture violation. HashSet should not need special
/// compiler treatment - it should be a pure stdlib type once traits are implemented.
#[inline]
pub fn is_hashset(type_name: &str) -> bool {
    type_name == "HashSet"
}

/// Check if a type is Vec or DynVec (Layer 3 types with temporary compiler support)
///
/// NOTE: This is an architecture violation. Vec/DynVec should not need special
/// compiler treatment - they should be pure stdlib types once traits are implemented.
#[inline]
pub fn is_vec_type(type_name: &str) -> bool {
    matches!(type_name, "Vec" | "DynVec")
}

/// Check if a type is any Layer 3 collection type with special compiler treatment
///
/// This includes HashMap, HashSet, Vec, and DynVec - all architecture violations
/// that should be removed in Phase 5 when the trait system is complete.
#[inline]
pub fn is_special_collection(type_name: &str) -> bool {
    is_hashmap(type_name) || is_hashset(type_name) || is_vec_type(type_name)
}
