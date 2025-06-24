use super::*;

#[derive(Debug, Clone)]
pub struct StructTypeInfo<'ctx> {
    /// The LLVM struct type
    pub llvm_type: StructType<'ctx>,
    /// Mapping from field name to (index, type)
    pub fields: HashMap<String, (usize, AstType)>,
}
// Move any struct registration, lookup, or field access helpers here as needed. 