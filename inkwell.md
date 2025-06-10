Skip to content
 
Search Gists
Search...
All gists
Back to GitHub
@lantos1618
lantos1618/pls
Created 4 months ago
Code
Revisions
1
Clone this repository at &lt;script src=&quot;https://gist.github.com/lantos1618/2054c559a93d4df538cbc2adcf58bdb4.js&quot;&gt;&lt;/script&gt;
<script src="https://gist.github.com/lantos1618/2054c559a93d4df538cbc2adcf58bdb4.js"></script>
inkwell api
pls

```
type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;
struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}
```
```rust
pub enum Token {
    Binary,
    Comma,
    Comment,
    Def,
    Else,
    EOF,
    Extern,
    For,
    Ident(String),
    If,
    In,
    LParen,
    Number(f64),
    Op(char),
    RParen,
    Then,
    Unary,
    Var,
}
pub struct LexError {
    pub error: &'static str,
    pub index: usize,
}
pub type LexResult = Result<Token, LexError>;
pub struct Lexer<'a> {
    input: &'a str,
    chars: Box<Peekable<Chars<'a>>>,
    pos: usize,
}
pub enum Expr {
    Binary {
        op: char,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        fn_name: String,
        args: Vec<Expr>,
    },
    Conditional {
        cond: Box<Expr>,
        consequence: Box<Expr>,
        alternative: Box<Expr>,
    },
    For {
        var_name: String,
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
        body: Box<Expr>,
    },
    Number(f64),
    Variable(String),
    VarIn {
        variables: Vec<(String, Option<Expr>)>,
        body: Box<Expr>,
    },
}
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
    pub is_op: bool,
    pub prec: usize,
}
pub struct Function {
    pub prototype: Prototype,
    pub body: Option<Expr>,
    pub is_anon: bool,
}
pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    prec: &'a mut HashMap<char, i32>,
}
pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub function: &'a Function,
    variables: HashMap<String, PointerValue<'ctx>>,
    fn_value_opt: Option<FunctionValue<'ctx>>,
}
```
```rust
pub enum Token {
    Binary,
    Comma,
    Comment,
    Def,
    Else,
    EOF,
    Extern,
    For,
    Ident(String),
    If,
    In,
    LParen,
    Number(f64),
    Op(char),
    RParen,
    Then,
    Unary,
    Var,
}
pub struct LexError {
    pub error: &'static str,
    pub index: usize,
}
pub type LexResult = Result<Token, LexError>;
pub struct Lexer<'a> {
    input: &'a str,
    chars: Box<Peekable<Chars<'a>>>,
    pos: usize,
}
pub enum Expr {
    Binary {
        op: char,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        fn_name: String,
        args: Vec<Expr>,
    },
    Conditional {
        cond: Box<Expr>,
        consequence: Box<Expr>,
        alternative: Box<Expr>,
    },
    For {
        var_name: String,
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
        body: Box<Expr>,
    },
    Number(f64),
    Variable(String),
    VarIn {
        variables: Vec<(String, Option<Expr>)>,
        body: Box<Expr>,
    },
}
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
    pub is_op: bool,
    pub prec: usize,
}
pub struct Function {
    pub prototype: Prototype,
    pub body: Option<Expr>,
    pub is_anon: bool,
}
pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    prec: &'a mut HashMap<char, i32>,
}
pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub function: &'a Function,
    variables: HashMap<String, PointerValue<'ctx>>,
    fn_value_opt: Option<FunctionValue<'ctx>>,
}
```
```rust
pub(crate) struct VersionRange {
    start: Option<Version>,
    limits: RangeLimits,
    end: Option<Version>,
    features: &'static [&'static str],
}
struct Version {
    major: u32,
    minor: u32,
    span: Span,
}
pub struct VersionFolder {
    result: Result<()>,
}
```
```rust
struct EnumVariant {
    llvm_variant: Ident,
    rust_variant: Ident,
    attrs: Vec<Attribute>,
}
#[derive(Default)]
struct EnumVariants {
    variants: Vec<EnumVariant>,
    error: Option<Error>,
}
pub struct LLVMEnumType {
    name: Ident,
    decl: syn::ItemEnum,
    variants: EnumVariants,
}
```
```rust
pub struct Attribute {
    pub(crate) attribute: LLVMAttributeRef,
}
pub enum AttributeLoc {
    Return,
    Param(u32),
    Function,
}
```
```rust
pub struct BasicBlock<'ctx> {
    pub(crate) basic_block: LLVMBasicBlockRef,
    _marker: PhantomData<&'ctx ()>,
}
pub struct InstructionIter<'ctx>(Option<InstructionValue<'ctx>>);
```
```rust
pub enum ComdatSelectionKind {
    Any,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
pub struct Comdat(pub(crate) LLVMComdatRef);
```
```rust
pub(crate) struct ContextImpl(pub(crate) LLVMContextRef);
pub struct Context {
    pub(crate) context: ContextImpl,
}
pub struct ContextRef<'ctx> {
    pub(crate) context: ContextImpl,
    _marker: PhantomData<&'ctx Context>,
}
```
```rust
pub struct DataLayout {
    pub(crate) data_layout: LLVMStringOrRaw,
}
```
```rust
pub enum Token {
    Binary,
    Comma,
    Comment,
    Def,
    Else,
    EOF,
    Extern,
    For,
    Ident(String),
    If,
    In,
    LParen,
    Number(f64),
    Op(char),
    RParen,
    Then,
    Unary,
    Var,
}
pub struct LexError {
    pub error: &'static str,
    pub index: usize,
}
pub type LexResult = Result<Token, LexError>;
pub struct Lexer<'a> {
    input: &'a str,
    chars: Box<Peekable<Chars<'a>>>,
    pos: usize,
}
pub enum Expr {
    Binary {
        op: char,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        fn_name: String,
        args: Vec<Expr>,
    },
    Conditional {
        cond: Box<Expr>,
        consequence: Box<Expr>,
        alternative: Box<Expr>,
    },
    For {
        var_name: String,
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
        body: Box<Expr>,
    },
    Number(f64),
    Variable(String),
    VarIn {
        variables: Vec<(String, Option<Expr>)>,
        body: Box<Expr>,
    },
}
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
    pub is_op: bool,
    pub prec: usize,
}
pub struct Function {
    pub prototype: Prototype,
    pub body: Option<Expr>,
    pub is_anon: bool,
}
pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    prec: &'a mut HashMap<char, i32>,
}
pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub function: &'a Function,
    variables: HashMap<String, PointerValue<'ctx>>,
    fn_value_opt: Option<FunctionValue<'ctx>>,
}
```
```rust
pub enum FunctionLookupError {
    JITNotEnabled,
    FunctionNotFound,
}
pub enum RemoveModuleError {
    ModuleNotOwned,
    IncorrectModuleOwner,
    LLVMError(LLVMString),
}
pub struct ExecutionEngine<'ctx> {
    execution_engine: Option<ExecEngineInner<'ctx>>,
    target_data: Option<TargetData>,
    jit_mode: bool,
}
struct ExecEngineInner<'ctx>(Rc<LLVMExecutionEngineRef>, PhantomData<&'ctx Context>);
pub struct JitFunction<'ctx, F> {
    _execution_engine: ExecEngineInner<'ctx>,
    inner: F,
}
pub trait UnsafeFunctionPointer: private::SealedUnsafeFunctionPointer {}
mod private {
    pub trait SealedUnsafeFunctionPointer: Copy {}
}
pub struct MangledSymbol(*mut libc::c_char);
pub struct LLVMError(LLVMErrorRef);
pub struct Orc(LLVMOrcJITStackRef);
```
```rust
pub struct Intrinsic {
    id: u32,
}
```
```rust
pub struct Attribute {
    pub(crate) attribute: LLVMAttributeRef,
}
pub enum AttributeLoc {
    Return,
    Param(u32),
    Function,
}
```
```rust
pub struct BasicBlock<'ctx> {
    pub(crate) basic_block: LLVMBasicBlockRef,
    _marker: PhantomData<&'ctx ()>,
}
pub struct InstructionIter<'ctx>(Option<InstructionValue<'ctx>>);
```
```rust
pub struct Comdat(pub(crate) LLVMComdatRef);
```
```rust
pub(crate) struct ContextImpl(pub(crate) LLVMContextRef);
pub struct Context {
    pub(crate) context: ContextImpl,
}
pub struct ContextRef<'ctx> {
    pub(crate) context: ContextImpl,
    _marker: PhantomData<&'ctx Context>,
}
```
```rust
pub struct DataLayout {
    pub(crate) data_layout: LLVMStringOrRaw,
}
```
```rust
pub struct DebugInfoBuilder<'ctx> {
    pub(crate) builder: LLVMDIBuilderRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DIScope<'ctx> {
    metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub trait AsDIScope<'ctx> {
    #[allow(clippy::wrong_self_convention)]
    fn as_debug_info_scope(self) -> DIScope<'ctx>;
}
pub struct DIFile<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DICompileUnit<'ctx> {
    file: DIFile<'ctx>,
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DINamespace<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DISubprogram<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    pub(crate) _marker: PhantomData<&'ctx Context>,
}
pub struct DIType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DIDerivedType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DIBasicType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DICompositeType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DISubroutineType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DILexicalBlock<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DILocation<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    pub(crate) _marker: PhantomData<&'ctx Context>,
}
pub struct DILocalVariable<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DIGlobalVariableExpression<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub struct DIExpression<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
pub enum DWARFEmissionKind {
    None,
    Full,
    LineTablesOnly,
}
pub enum DWARFSourceLanguage {
    C89,
    C,
    Ada83,
    CPlusPlus,
    Cobol74,
    Cobol85,
    Fortran77,
    Fortran90,
    Pascal83,
    Modula2,
    Java,
    C99,
    Ada95,
    Fortran95,
    PLI,
    ObjC,
    ObjCPlusPlus,
    UPC,
    D,
    Python,
    OpenCL,
    Go,
    Modula3,
    Haskell,
    CPlusPlus03,
    CPlusPlus11,
    OCaml,
    Rust,
    C11,
    Swift,
    Julia,
    Dylan,
    CPlusPlus14,
    Fortran03,
    Fortran08,
    RenderScript,
    BLISS,
    MipsAssembler,
    GOOGLERenderScript,
    BORLANDDelphi,
    Kotlin,
    Zig,
    Crystal,
    CPlusPlus17,
    CPlusPlus20,
    C17,
    Fortran18,
    Ada2005,
    Ada2012,
    Mojo,
}
```
```rust
pub struct MemoryBuffer {
    pub(crate) memory_buffer: LLVMMemoryBufferRef,
}
```
```rust
pub trait McjitMemoryManager: std::fmt::Debug {
    fn allocate_code_section(
        &mut self,
        size: libc::uintptr_t,
        alignment: libc::c_uint,
        section_id: libc::c_uint,
        section_name: &str,
    ) -> *mut u8;
    fn allocate_data_section(
        &mut self,
        size: libc::uintptr_t,
        alignment: libc::c_uint,
        section_id: libc::c_uint,
        section_name: &str,
        is_read_only: bool,
    ) -> *mut u8;
    fn finalize_memory(&mut self) -> Result<(), String>;
    fn destroy(&mut self);
}
pub struct MemoryManagerAdapter {
    pub memory_manager: Box<dyn McjitMemoryManager>,
}
```
```rust
pub enum Linkage {
    Appending,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
pub struct Module<'ctx> {
    data_layout: RefCell<Option<DataLayout>>,
    pub(crate) module: Cell<LLVMModuleRef>,
    pub(crate) owned_by_ee: RefCell<Option<ExecutionEngine<'ctx>>>,
    _marker: PhantomData<&'ctx Context>,
}
pub struct FunctionIterator<'ctx>(Option<FunctionValue<'ctx>>);
pub struct InstructionIter<'ctx>(Option<InstructionValue<'ctx>>);
pub struct GlobalIterator<'ctx>(Option<GlobalValue<'ctx>>);
pub enum FlagBehavior {
    Error,
    Warning,
    Require,
    Override,
    Append,
    AppendUnique,
}
```
```rust
pub struct ObjectFile {
    object_file: LLVMObjectFileRef,
}
pub struct SectionIterator {
    section_iterator: LLVMSectionIteratorRef,
    object_file: LLVMObjectFileRef,
    before_first: bool,
}
pub struct Section {
    section: LLVMSectionIteratorRef,
    object_file: LLVMObjectFileRef,
}
pub struct RelocationIterator {
    relocation_iterator: LLVMRelocationIteratorRef,
    section_iterator: LLVMSectionIteratorRef,
    object_file: LLVMObjectFileRef,
    before_first: bool,
}
pub struct Relocation {
    relocation: LLVMRelocationIteratorRef,
    object_file: LLVMObjectFileRef,
}
pub struct SymbolIterator {
    symbol_iterator: LLVMSymbolIteratorRef,
    object_file: LLVMObjectFileRef,
    before_first: bool,
}
pub struct Symbol {
    symbol: LLVMSymbolIteratorRef,
}
```
```rust
pub struct PassManagerBuilder {
    pass_manager_builder: LLVMPassManagerBuilderRef,
}
pub struct PassManager<T> {
    pub(crate) pass_manager: LLVMPassManagerRef,
    sub_type: PhantomData<T>,
}
pub struct PassRegistry {
    pass_registry: LLVMPassRegistryRef,
}
pub struct PassBuilderOptions {
    pub(crate) options_ref: LLVMPassBuilderOptionsRef,
}
```
```rust
pub(crate) struct DiagnosticInfo {
    diagnostic_info: LLVMDiagnosticInfoRef,
}
```
```rust
pub struct LLVMString {
    pub(crate) ptr: *const c_char,
}
pub(crate) enum LLVMStringOrRaw {
    Owned(LLVMString),
    Borrowed(*const c_char),
}
pub enum LoadLibraryError {
    UnicodeError,
    LoadingError,
}
```
```rust
pub enum CodeModel {
    Default,
    JITDefault,
    Small,
    Kernel,
    Medium,
    Large,
}
pub enum RelocMode {
    Default,
    Static,
    PIC,
    DynamicNoPic,
}
pub enum FileType {
    Assembly,
    Object,
}
pub struct InitializationConfig {
    pub asm_parser: bool,
    pub asm_printer: bool,
    pub base: bool,
    pub disassembler: bool,
    pub info: bool,
    pub machine_code: bool,
}
pub struct TargetTriple {
    pub(crate) triple: LLVMString,
}
pub struct Target {
    target: LLVMTargetRef,
}
pub struct TargetMachine {
    pub(crate) target_machine: LLVMTargetMachineRef,
}
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}
pub struct TargetData {
    pub(crate) target_data: LLVMTargetDataRef,
}
pub struct TargetMachineOptions(Option<LLVMTargetMachineOptionsRef>);
```
```rust
pub struct ArrayType<'ctx> {
    array_type: Type<'ctx>,
}
pub struct InstructionIter<'ctx>(Option<InstructionValue<'ctx>>);
```
```rust
pub enum AnyTypeEnum<'ctx> {
    ArrayType(ArrayType<'ctx>),
    FloatType(FloatType<'ctx>),
    FunctionType(FunctionType<'ctx>),
    IntType(IntType<'ctx>),
    PointerType(PointerType<'ctx>),
    StructType(StructType<'ctx>),
    VectorType(VectorType<'ctx>),
    ScalableVectorType(ScalableVectorType<'ctx>),
    VoidType(VoidType<'ctx>),
}
pub enum BasicTypeEnum<'ctx> {
    ArrayType(ArrayType<'ctx>),
    FloatType(FloatType<'ctx>),
    IntType(IntType<'ctx>),
    PointerType(PointerType<'ctx>),
    StructType(StructType<'ctx>),
    VectorType(VectorType<'ctx>),
    ScalableVectorType(ScalableVectorType<'ctx>),
}
pub enum BasicMetadataTypeEnum<'ctx> {
    ArrayType(ArrayType<'ctx>),
    FloatType(FloatType<'ctx>),
    IntType(IntType<'ctx>),
    PointerType(PointerType<'ctx>),
    StructType(StructType<'ctx>),
    VectorType(VectorType<'ctx>),
    ScalableVectorType(ScalableVectorType<'ctx>),
    MetadataType(MetadataType<'ctx>),
}
```
```rust
pub struct FloatType<'ctx> {
    float_type: Type<'ctx>,
}
```
```rust
pub struct FunctionType<'ctx> {
    fn_type: Type<'ctx>,
}
pub struct ParamValueIter<'ctx> {
    param_iter_value: LLVMValueRef,
    start: bool,
    _marker: PhantomData<&'ctx ()>,
}
pub struct BasicBlockIter<'ctx>(Option<BasicBlock<'ctx>>);
```
```rust
pub enum StringRadix {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16,
    Alphanumeric = 36,
}
pub struct IntType<'ctx> {
    int_type: Type<'ctx>,
}
```
```rust
pub struct MetadataType<'ctx> {
    metadata_type: Type<'ctx>,
}
```
```rust
pub struct PointerType<'ctx> {
    ptr_type: Type<'ctx>,
}
```
```rust
pub struct ScalableVectorType<'ctx> {
    scalable_vec_type: Type<'ctx>,
}
```
```rust
pub struct StructType<'ctx> {
    struct_type: Type<'ctx>,
}
pub struct FieldTypesIter<'ctx> {
    st: StructType<'ctx>,
    i: u32,
    count: u32,
}
```
```rust
pub trait AsTypeRef {
    fn as_type_ref(&self) -> LLVMTypeRef;
}
pub trait AnyType<'ctx>: AsTypeRef + Debug {
    fn as_any_type_enum(&self) -> AnyTypeEnum<'ctx>;
    fn print_to_string(&self) -> LLVMString;
}
pub unsafe trait BasicType<'ctx>: AnyType<'ctx> {
    fn as_basic_type_enum(&self) -> BasicTypeEnum<'ctx>;
    fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx>;
    fn is_sized(&self) -> bool;
    fn size_of(&self) -> Option<IntValue<'ctx>>;
    fn array_type(&self, size: u32) -> ArrayType<'ctx>;
    fn ptr_type(&self, address_space: AddressSpace) -> PointerType<'ctx>;
}
pub unsafe trait IntMathType<'ctx>: BasicType<'ctx> {
    type ValueType: IntMathValue<'ctx>;
    type MathConvType: FloatMathType<'ctx>;
    type PtrConvType: PointerMathType<'ctx>;
}
pub unsafe trait FloatMathType<'ctx>: BasicType<'ctx> {
    type ValueType: FloatMathValue<'ctx>;
    type MathConvType: IntMathType<'ctx>;
}
pub unsafe trait PointerMathType<'ctx>: BasicType<'ctx> {
    type ValueType: PointerMathValue<'ctx>;
    type PtrConvType: IntMathType<'ctx>;
}
pub unsafe trait VectorBaseValue<'ctx>: BasicType<'ctx> {
    unsafe fn new(value: LLVMValueRef) -> Self;
}
```
```rust
pub struct VectorType<'ctx> {
    vec_type: Type<'ctx>,
}
```
```rust
pub struct VoidType<'ctx> {
    void_type: Type<'ctx>,
}
```
```rust
pub struct ArrayValue<'ctx> {
    array_value: Value<'ctx>,
}
```
```rust
pub struct BasicValueUse<'ctx>(LLVMUseRef, PhantomData<&'ctx ()>);
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
```
```rust
pub struct CallSiteValue<'ctx>(Value<'ctx>);
```
```rust
pub enum AggregateValueEnum<'ctx> {
    ArrayValue(ArrayValue<'ctx>),
    StructValue(StructValue<'ctx>),
}
pub enum AnyValueEnum<'ctx> {
    ArrayValue(ArrayValue<'ctx>),
    IntValue(IntValue<'ctx>),
    FloatValue(FloatValue<'ctx>),
    PhiValue(PhiValue<'ctx>),
    FunctionValue(FunctionValue<'ctx>),
    PointerValue(PointerValue<'ctx>),
    StructValue(StructValue<'ctx>),
    VectorValue(VectorValue<'ctx>),
    ScalableVectorValue(ScalableVectorValue<'ctx>),
    InstructionValue(InstructionValue<'ctx>),
    MetadataValue(MetadataValue<'ctx>),
}
pub enum BasicValueEnum<'ctx> {
    ArrayValue(ArrayValue<'ctx>),
    IntValue(IntValue<'ctx>),
    FloatValue(FloatValue<'ctx>),
    PointerValue(PointerValue<'ctx>),
    StructValue(StructValue<'ctx>),
    VectorValue(VectorValue<'ctx>),
    ScalableVectorValue(ScalableVectorValue<'ctx>),
}
pub enum BasicMetadataValueEnum<'ctx> {
    ArrayValue(ArrayValue<'ctx>),
    IntValue(IntValue<'ctx>),
    FloatValue(FloatValue<'ctx>),
    PointerValue(PointerValue<'ctx>),
    StructValue(StructValue<'ctx>),
    VectorValue(VectorValue<'ctx>),
    ScalableVectorValue(ScalableVectorValue<'ctx>),
    MetadataValue(MetadataValue<'ctx>),
}
```
```rust
pub struct FloatValue<'ctx> {
    float_value: Value<'ctx>,
}
```
```rust
pub struct FunctionValue<'ctx> {
    fn_value: Value<'ctx>,
}
pub struct BasicBlockIter<'ctx>(Option<BasicBlock<'ctx>>);
pub struct ParamValueIter<'ctx> {
    param_iter_value: LLVMValueRef,
    start: bool,
    _marker: PhantomData<&'ctx ()>,
}
```
```rust
pub struct GenericValue<'ctx> {
    pub(crate) generic_value: LLVMGenericValueRef,
    _phantom: PhantomData<&'ctx ()>,
}
```
```rust
pub struct GlobalValue<'ctx> {
    global_value: Value<'ctx>,
}
pub enum UnnamedAddress {
    None,
    Local,
    Global,
}
```
```rust
pub enum InstructionOpcode {
    Add,
    AddrSpaceCast,
    Alloca,
    And,
    AShr,
    AtomicCmpXchg,
    AtomicRMW,
    BitCast,
    Br,
    Call,
    CallBr,
    CatchPad,
    CatchRet,
    CatchSwitch,
    CleanupPad,
    CleanupRet,
    ExtractElement,
    ExtractValue,
    FNeg,
    FAdd,
    FCmp,
    FDiv,
    Fence,
    FMul,
    FPExt,
    FPToSI,
    FPToUI,
    FPTrunc,
    Freeze,
    FRem,
    FSub,
    GetElementPtr,
    ICmp,
    IndirectBr,
    InsertElement,
    InsertValue,
    IntToPtr,
    Invoke,
    LandingPad,
    Load,
    LShr,
    Mul,
    Or,
    Phi,
    PtrToInt,
    Resume,
    Return,
    SDiv,
    Select,
    SExt,
    Shl,
    ShuffleVector,
    SIToFP,
    SRem,
    Store,
    Sub,
    Switch,
    Trunc,
    UDiv,
    UIToFP,
    Unreachable,
    URem,
    UserOp1,
    UserOp2,
    VAArg,
    Xor,
    ZExt,
}
pub struct InstructionValue<'ctx> {
    instruction_value: Value<'ctx>,
}
pub struct OperandIter<'ctx>(Option<Either<BasicValueEnum<'ctx>, BasicBlock<'ctx>>>, u32, u32, InstructionValue<'ctx>);
pub struct OperandUseIter<'ctx>(BasicValueUse<'ctx>, u32, u32, InstructionValue<'ctx>);
```
```rust
pub enum StringRadix {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16,
    Alphanumeric = 36,
}
pub struct IntValue<'ctx> {
    int_value: Value<'ctx>,
}
```
```rust
pub struct MetadataValue<'ctx> {
    metadata_value: Value<'ctx>,
}
```
```rust
pub struct PhiValue<'ctx> {
    phi_value: Value<'ctx>,
}
pub struct IncomingIter<'ctx> {
    pv: PhiValue<'ctx>,
    i: u32,
    count: u32,
}
```
```rust
pub struct PointerValue<'ctx> {
    ptr_value: Value<'ctx>,
}
```
```rust
pub struct ScalableVectorValue<'ctx> {
    scalable_vec_value: Value<'ctx>,
}
```
```rust
pub struct StructValue<'ctx> {
    struct_value: Value<'ctx>,
}
pub struct FieldValueIter<'ctx> {
    sv: StructValue<'ctx>,
    i: u32,
    count: u32,
}
```
```rust
pub trait AsValueRef {
    fn as_value_ref(&self) -> LLVMValueRef;
}
pub trait AnyValue<'ctx>: AsValueRef + Debug {
    fn as_any_value_enum(&self) -> AnyValueEnum<'ctx>;
    fn print_to_string(&self) -> LLVMString;
}
pub unsafe trait BasicValue<'ctx>: AnyValue<'ctx> {
    fn as_basic_value_enum(&self) -> BasicValueEnum<'ctx>;
    fn as_instruction_value(&self) -> Option<InstructionValue<'ctx>>;
    fn get_first_use(&self) -> Option<BasicValueUse>;
    fn set_name(&self, name: &str);
    fn array_type(&self, size: u32) -> ArrayType<'ctx>;
    fn ptr_type(&self, address_space: AddressSpace) -> PointerType<'ctx>;
}
pub unsafe trait IntMathValue<'ctx>: BasicValue<'ctx> {
    type BaseType: IntMathType<'ctx>;
    unsafe fn new(value: LLVMValueRef) -> Self;
}
pub unsafe trait FloatMathValue<'ctx>: BasicValue<'ctx> {
    type ValueType: FloatMathValue<'ctx>;
    type MathConvType: IntMathType<'ctx>;
    unsafe fn new(value: LLVMValueRef) -> Self;
}
pub unsafe trait PointerMathValue<'ctx>: BasicValue<'ctx> {
    type ValueType: PointerMathValue<'ctx>;
    type PtrConvType: IntMathType<'ctx>;
    unsafe fn new(value: LLVMValueRef) -> Self;
}
pub unsafe trait VectorBaseValue<'ctx>: BasicValue<'ctx> {
    unsafe fn new(value: LLVMValueRef) -> Self;
}
```
```rust
pub struct VectorValue<'ctx> {
    vec_value: Value<'ctx>,
}
```
```rust
pub struct VoidType<'ctx> {
    void_type: Type<'ctx>,
}
```
```rust
pub struct OperandBundle<'ctx> {
    bundle: Cell<LLVMOperandBundleRef>,
    _marker: PhantomData<&'ctx Context>,
}
pub struct OperandBundleIter<'a, 'ctx> {
    instruction: &'a CallSiteValue<'ctx>,
    current: u32,
    size: u32,
}
pub struct OperandBundleArgsIter<'a, 'ctx> {
    bundle: &'a OperandBundle<'ctx>,
    current: u32,
    size: u32,
}
```
```rust
pub struct Value<'ctx> {
    value: LLVMValueRef,
    _marker: PhantomData<&'ctx ()>,
}
```
```rust
pub enum IntPredicate {
    EQ,
    NE,
    UGT,
    UGE,
    ULT,
    ULE,
    SGT,
    SGE,
    SLT,
    SLE,
}
pub enum FloatPredicate {
    OEQ,
    OGE,
    OGT,
    OLE,
    OLT,
    ONE,
    ORD,
    PredicateFalse,
    PredicateTrue,
    UEQ,
    UGE,
    UGT,
    ULE,
    ULT,
    UNE,
    UNO,
}
pub enum AtomicOrdering {
    NotAtomic,
    Unordered,
    Monotonic,
    Acquire,
    Release,
    AcquireRelease,
    SequentiallyConsistent,
}
pub enum AtomicRMWBinOp {
    Xchg,
    Add,
    Sub,
    And,
    Nand,
    Or,
    Xor,
    Max,
    Min,
    UMax,
    UMin,
    FAdd,
    FSub,
    FMax,
    FMin,
}
pub enum OptimizationLevel {
    None = 0,
    Less = 1,
    Default = 2,
    Aggressive = 3,
}
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    LocalDynamicTLSModel,
    InitialExecTLSModel,
    LocalExecTLSModel,
}
pub enum DLLStorageClass {
    Default,
    Import,
    Export,
}
pub enum InlineAsmDialect {
    ATT,
    Intel,
}
pub enum AttributeLoc {
    Return,
    Param(u32),
    Function,
}
pub enum Linkage {
    Appending,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
```
```rust
pub enum InstructionOpcode {
    Add,
    AddrSpaceCast,
    Alloca,
    And,
    AShr,
    AtomicCmpXchg,
    AtomicRMW,
    BitCast,
    Br,
    Call,
    CallBr,
    CatchPad,
    CatchRet,
    CatchSwitch,
    CleanupPad,
    CleanupRet,
    ExtractElement,
    ExtractValue,
    FNeg,
    FAdd,
    FCmp,
    FDiv,
    Fence,
    FMul,
    FPExt,
    FPToSI,
    FPToUI,
    FPTrunc,
    Freeze,
    FRem,
    FSub,
    GetElementPtr,
    ICmp,
    IndirectBr,
    InsertElement,
    InsertValue,
    IntToPtr,
    Invoke,
    LandingPad,
    Load,
    LShr,
    Mul,
    Or,
    Phi,
    PtrToInt,
    Resume,
    Return,
    SDiv,
    Select,
    SExt,
    Shl,
    ShuffleVector,
    SIToFP,
    SRem,
    Store,
    Sub,
    Switch,
    Trunc,
    UDiv,
    UIToFP,
    Unreachable,
    URem,
    UserOp1,
    UserOp2,
    VAArg,
    Xor,
    ZExt,
}
```
```rust
pub enum AttributeLoc {
    Return,
    Param(u32),
    Function,
}
```
```rust
pub enum AddressSpace {
    Generic = 0,
    推测 = 256,
    OpenCLGlobal = 1,
    OpenCLLocal = 2,
    OpenCLConstant = 3,
    OpenCLGeneric = 4,
    CudaGeneric = 256,
    CudaShared = 3,
    CudaConstant = 4,
    CudaLocal = 5,
    AMDGPUGlobal = 1,
    AMDGPUAddressSpaceFlat = 5,
    AMDGPUConstant = 4,
    AMDGPUCodeObjectConstant = 9,
    AMDGPUGlobal зерк = 1,
    AMDGPULocal = 3,
    AMDGPUParam = 2,
    AMDGPUPrivate = 5,
    AMDGPUKernarg = 4,
    AMDGPUDispatchPtr = 6,
    AMDGPUQueuePtr = 7,
    AMDGPU তরঙ্গ面 = 8,
    AMDGPUGlobalBuffer = 10,
    AMDGPUFlatGlobalBuffer = 11,
    RenderScript = 1,
    SPIRGeneric = 0,
    SPIRGlobal = 1,
    SPIRWorkgroupLocal = 2,
    SPIRPrivate = 3,
    SPIRConstant = 4,
    SPIRLocal = 5,
    SYCLGeneric = 0,
    SYCLGlobal = 1,
    SYCLWorkItemGroupLocal = 2,
    SYCLPrivate = 3,
    SYCLConstant = 4,
    SYCLLocal = 5,
    SYCLCGeneric = 0,
    SYCLCGlobal = 1,
    SYCLCWorkItemGroupLocal = 2,
    SYCLCConstant = 4,
    SYCLCLocal = 5,
    SYCLBuiltin = 6,
    SYCLCBuiltin = 6,
    NVCPUGlobal = 1,
    NVCPUGeneric = 256,
}
```
```rust
pub enum StringRadix {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16,
    Alphanumeric = 36,
}
```
```rust
pub enum AttributeLoc {
    Return,
    Param(u32),
    Function,
}
```
```rust
pub enum ComdatSelectionKind {
    Any,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
```
```rust
pub enum DLLStorageClass {
    Default,
    Import,
    Export,
}
```
```rust
pub enum DWARFEmissionKind {
    None,
    Full,
    LineTablesOnly,
}
```
```rust
pub enum DWARFSourceLanguage {
    C89,
    C,
    Ada83,
    CPlusPlus,
    Cobol74,
    Cobol85,
    Fortran77,
    Fortran90,
    Pascal83,
    Modula2,
    Java,
    C99,
    Ada95,
    Fortran95,
    PLI,
    ObjC,
    ObjCPlusPlus,
    UPC,
    D,
    Python,
    OpenCL,
    Go,
    Modula3,
    Haskell,
    CPlusPlus03,
    CPlusPlus11,
    OCaml,
    Rust,
    C11,
    Swift,
    Julia,
    Dylan,
    CPlusPlus14,
    Fortran03,
    Fortran08,
    RenderScript,
    BLISS,
    MipsAssembler,
    GOOGLERenderScript,
    BORLANDDelphi,
    Kotlin,
    Zig,
    Crystal,
    CPlusPlus17,
    CPlusPlus20,
    C17,
    Fortran18,
    Ada2005,
    Ada2012,
    Mojo,
}
```
```rust
pub enum FileType {
    Assembly,
    Object,
}
```
```rust
pub enum FloatPredicate {
    OEQ,
    OGE,
    OGT,
    OLE,
    OLT,
    ONE,
    ORD,
    PredicateFalse,
    PredicateTrue,
    UEQ,
    UGE,
    UGT,
    ULE,
    ULT,
    UNE,
    UNO,
}
```
```rust
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}
```
```rust
pub enum InlineAsmDialect {
    ATT,
    Intel,
}
```
```rust
pub enum IntPredicate {
    EQ,
    NE,
    UGT,
    UGE,
    ULT,
    ULE,
    SGT,
    SGE,
    SLT,
    SLE,
}
```
```rust
pub enum Linkage {
    Appending,
    External,
    AvailableExternally,
    LinkOnceAny,
    LinkOnceODR,
    WeakAny,
    WeakODR,
    Common,
    Private,
    Internal,
    ExternWeak,
    LinkerPrivate,
    LinkerPrivateWeak,
    DLLExport,
    DLLImport,
    Ghost,
    LinkOnceODRAutoHide,
}
```
```rust
pub enum OptimizationLevel {
    None = 0,
    Less = 1,
    Default = 2,
    Aggressive = 3,
}
```
```rust
pub enum RelocMode {
    Default,
    Static,
    PIC,
    DynamicNoPic,
}
```
```rust
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    LocalDynamicTLSModel,
    InitialExecTLSModel,
    LocalExecTLSModel,
}
```
```rust
pub enum UnnamedAddress {
    None,
    Local,
    Global,
}
```
```rust
pub enum LLVMTailCallKind {
    None,
    Tail,
    MustTail,
    NoTail,
}
```
```rust
pub enum InstructionOpcode {
    Add,
    AddrSpaceCast,
    Alloca,
    And,
    AShr,
    AtomicCmpXchg,
    AtomicRMW,
    BitCast,
    Br,
    Call,
    CallBr,
    CatchPad,
    CatchRet,
    CatchSwitch,
    CleanupPad,
    CleanupRet,
    ExtractElement,
    ExtractValue,
    FNeg,
    FAdd,
    FCmp,
    FDiv,
    Fence,
    FMul,
    FPExt,
    FPToSI,
    FPToUI,
    FPTrunc,
    Freeze,
    FRem,
    FSub,
    GetElementPtr,
    ICmp,
    IndirectBr,
    InsertElement,
    InsertValue,
    IntToPtr,
    Invoke,
    LandingPad,
    Load,
    LShr,
    Mul,
    Or,
    Phi,
    PtrToInt,
    Resume,
    Return,
    SDiv,
    Select,
    SExt,
    Shl,
    ShuffleVector,
    SIToFP,
    SRem,
    Store,
    Sub,
    Switch,
    Trunc,
    UDiv,
    UIToFP,
    Unreachable,
    URem,
    UserOp1,
    UserOp2,
    VAArg,
    Xor,
    ZExt,
}
```
```rust
pub enum AtomicOrdering {
    NotAtomic,
    Unordered,
    Monotonic,
    Acquire,
    Release,
    AcquireRelease,
    SequentiallyConsistent,
}
```
```rust
pub enum AtomicRMWBinOp {
    Xchg,
    Add,
    Sub,
    And,
    Nand,
    Or,
    Xor,
    Max,
    Min,
    UMax,
    UMin,
    FAdd,
    FSub,
    FMax,
    FMin,
}
```
```rust
pub enum BasicMetadataValueEnum<'ctx> {
    ArrayValue(ArrayValue<'ctx>),
    IntValue(IntValue<'ctx>),
    FloatValue(FloatValue<'ctx>),
    PointerValue(PointerValue<'ctx>),
    StructValue(StructValue<'ctx>),
    VectorValue(VectorValue<'ctx>),
    ScalableVectorValue(ScalableVectorValue<'ctx>),
    MetadataValue(MetadataValue<'ctx>),
}
```
```rust
pub enum BasicTypeEnum<'ctx> {
    ArrayType(ArrayType<'ctx>),
    FloatType(FloatType<'ctx>),
    IntType(IntType<'ctx>),
    PointerType(PointerType<'ctx>),
    StructType(StructType<'ctx>),
    VectorType(VectorType<'ctx>),
    ScalableVectorType(ScalableVectorType<'ctx>),
}
```
```rust
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}
```
```rust
pub enum ComdatSelectionKind {
    Any,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
```
```rust
pub enum DLLStorageClass {
    Default,
    Import,
    Export,
}
```
```rust
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
```
```rust
pub enum FileType {
    Assembly,
    Object,
}
```
```rust
pub enum FloatPredicate {
    OEQ,
    OGE,
    OGT,
    OLE,
    OLT,
    ONE,
    ORD,
    PredicateFalse,
    PredicateTrue,
    UEQ,
    UGE,
    UGT,
    ULE,
    ULT,
    UNE,
    UNO,
}
```
```rust
pub enum FunctionLookupError {
    FunctionNotFound,
    JITNotEnabled,
}
```
```rust
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}
```
```rust
pub enum InlineAsmDialect {
    ATT,
    Intel,
}
```
```rust
pub enum IntPredicate {
    EQ,
    NE,
    UGT,
    UGE,
    ULT,
    ULE,
    SGT,
    SGE,
    SLT,
    SLE,
}
```
```rust
pub enum LoadLibraryError {
    LoadingError,
    UnicodeError,
}
```
```rust
pub enum OptimizationLevel {
    Aggressive,
    Default,
    Less,
    None,
}
```
```rust
pub enum RemoveModuleError {
    IncorrectModuleOwner,
    LLVMError(LLVMString),
    ModuleNotOwned,
}
```
```rust
pub enum RelocMode {
    Default,
    DynamicNoPic,
    PIC,
    Static,
}
```
```rust
pub enum StringRadix {
    Alphanumeric,
    Binary,
    Decimal,
    Hexadecimal,
    Octal,
}
```
```rust
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    InitialExecTLSModel,
    LocalDynamicTLSModel,
    LocalExecTLSModel,
}
```
```rust
pub enum UnnamedAddress {
    Global,
    Local,
    None,
}
```
```rust
pub enum LLVMTailCallKind {
    MustTail,
    NoTail,
    None,
    Tail,
}
```
```rust
pub enum BasicMetadataTypeEnum<'ctx> {
    ArrayType(ArrayType<'ctx>),
    FloatType(FloatType<'ctx>),
    IntType(IntType<'ctx>),
    MetadataType(MetadataType<'ctx>),
    PointerType(PointerType<'ctx>),
    ScalableVectorType(ScalableVectorType<'ctx>),
    StructType(StructType<'ctx>),
    VectorType(VectorType<'ctx>),
}
```
```rust
pub enum BasicTypeEnum<'ctx> {
    ArrayType(ArrayType<'ctx>),
    FloatType(FloatType<'ctx>),
    IntType(IntType<'ctx>),
    PointerType(PointerType<'ctx>),
    ScalableVectorType(ScalableVectorType<'ctx>),
    StructType(StructType<'ctx>),
    VectorType(VectorType<'ctx>),
}
```
```rust
pub enum DWARFEmissionKind {
    Full,
    LineTablesOnly,
    None,
}
```
```rust
pub enum DWARFSourceLanguage {
    Ada2005,
    Ada2012,
    Ada95,
    Ada83,
    BLISS,
    BORLANDDelphi,
    C,
    C11,
    C17,
    C89,
    C99,
    CPlusPlus,
    CPlusPlus03,
    CPlusPlus11,
    CPlusPlus14,
    CPlusPlus17,
    CPlusPlus20,
    Cobol74,
    Cobol85,
    Crystal,
    D,
    Dylan,
    Fortran03,
    Fortran08,
    Fortran18,
    Fortran77,
    Fortran90,
    Fortran95,
    GOOGLERenderScript,
    Go,
    Haskell,
    Java,
    Julia,
    Kotlin,
    MipsAssembler,
    Modula2,
    Modula3,
    Mojo,
    OCaml,
    OpenCL,
    Pascal83,
    PLI,
    Python,
    RenderScript,
    Rust,
    SPIRConstant,
    SPIRGeneric,
    SPIRGlobal,
    SPIRLocal,
    SPIRPrivate,
    SYCLCBuiltin,
    SYCLCGeneric,
    SYCLCGlobal,
    SYCLCLocal,
    SYCLCWorkItemGroupLocal,
    SYCLConstant,
    SYCLGeneric,
    SYCLGlobal,
    SYCLLocal,
    SYCLPrivate,
    SYCLWorkItemGroupLocal,
    Swift,
    UPC,
    Zig,
}
```
```rust
pub enum InstructionOpcode {
    AShr,
    Add,
    AddrSpaceCast,
    Alloca,
    And,
    AtomicCmpXchg,
    AtomicRMW,
    BitCast,
    Br,
    Call,
    CallBr,
    CatchPad,
    CatchRet,
    CatchSwitch,
    CleanupPad,
    CleanupRet,
    ExtractElement,
    ExtractValue,
    FAdd,
    FCmp,
    FDiv,
    FNeg,
    FPExt,
    FPToSI,
    FPToUI,
    FPTrunc,
    FRem,
    FSub,
    FMul,
    Fence,
    Freeze,
    GetElementPtr,
    ICmp,
    IndirectBr,
    InsertElement,
    InsertValue,
    IntToPtr,
    Invoke,
    LandingPad,
    LShr,
    Load,
    Mul,
    Or,
    PHI,
    PtrToInt,
    Return,
    Resume,
    SDiv,
    SExt,
    SIToFP,
    SRem,
    Select,
    Shl,
    ShuffleVector,
    Store,
    Sub,
    Switch,
    Trunc,
    UDiv,
    UIToFP,
    Unreachable,
    URem,
    UserOp1,
    UserOp2,
    VAArg,
    Xor,
    ZExt,
}
```
```rust
pub enum AttributeLoc {
    Function,
    Param(u32),
    Return,
}
```
```rust
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}
```
```rust
pub enum CodeModel {
    Default,
    JITDefault,
    Kernel,
    Large,
    Medium,
    Small,
}
```
```rust
pub enum DLLStorageClass {
    Default,
    Export,
    Import,
}
```
```rust
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
```
```rust
pub enum FileType {
    Assembly,
    Object,
}
```
```rust
pub enum FloatPredicate {
    OEQ,
    OGE,
    OGT,
    OLE,
    OLT,
    ONE,
    ORD,
    PredicateFalse,
    PredicateTrue,
    UEQ,
    UGE,
    UGT,
    ULE,
    ULT,
    UNE,
    UNO,
}
```
```rust
pub enum FunctionLookupError {
    FunctionNotFound,
    JITNotEnabled,
}
```
```rust
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}
```
```rust
pub enum InlineAsmDialect {
    ATT,
    Intel,
}
```
```rust
pub enum IntPredicate {
    EQ,
    NE,
    SGE,
    SGT,
    SLE,
    SLT,
    UGE,
    UGT,
    ULE,
    ULT,
}
```
```rust
pub enum LoadLibraryError {
    LoadingError,
    UnicodeError,
}
```
```rust
pub enum OptimizationLevel {
    Aggressive,
    Default,
    Less,
    None,
}
```
```rust
pub enum RemoveModuleError {
    IncorrectModuleOwner,
    LLVMError(LLVMString),
    ModuleNotOwned,
}
```
```rust
pub enum RelocMode {
    Default,
    DynamicNoPic,
    PIC,
    Static,
}
```
```rust
pub enum StringRadix {
    Alphanumeric,
    Binary,
    Decimal,
    Hexadecimal,
    Octal,
}
```
```rust
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    InitialExecTLSModel,
    LocalDynamicTLSModel,
    LocalExecTLSModel,
}
```
```rust
pub enum UnnamedAddress {
    Global,
    Local,
    None,
}
```
```rust
pub enum LLVMTailCallKind {
    MustTail,
    NoTail,
    None,
    Tail,
}
```
```rust
pub enum PassBuilderOptionsPassBuilderOptionsRefEnum {
    FunctionPassManager(PassManager<FunctionValue<'ctx>>),
    ModulePassManager(PassManager<Module<'ctx>>),
    PassBuilderOptions(PassBuilderOptions),
    LoopAnalysisManager,
    FunctionAnalysisManager,
    CGSCCAnalysisManager,
    ModuleAnalysisManager,
}
```
```rust
pub enum VersionRange {
    Infinite,
    VersionSpecific(Version),
    VersionRange(Range<Version>),
    VersionRangeInclusive(RangeInclusive<Version>),
}
```
```rust
pub enum Version {
    Version40,
    Version50,
    Version60,
    Version70,
    Version80,
    Version90,
    Version100,
    Version110,
    Version120,
    Version130,
    Version140,
    Version150,
    Version160,
    Version170,
    Version180,
    VersionLatest,
}
```
```rust
pub enum ComdatSelectionKind {
    Any,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
```
```rust
pub enum DLLStorageClass {
    Default,
    Export,
    Import,
}
```
```rust
pub enum DWARFEmissionKind {
    Full,
    LineTablesOnly,
    None,
}
```
```rust
pub enum DWARFSourceLanguage {
    Ada2005,
    Ada2012,
    Ada95,
    Ada83,
    BLISS,
    BORLANDDelphi,
    C,
    C11,
    C17,
    C89,
    C99,
    CPlusPlus,
    CPlusPlus03,
    CPlusPlus11,
    CPlusPlus14,
    CPlusPlus17,
    CPlusPlus20,
    Cobol74,
    Cobol85,
    Crystal,
    D,
    Dylan,
    Fortran03,
    Fortran08,
    Fortran18,
    Fortran77,
    Fortran90,
    Fortran95,
    GOOGLERenderScript,
    Go,
    Haskell,
    Java,
    Julia,
    Kotlin,
    MipsAssembler,
    Modula2,
    Modula3,
    Mojo,
    OCaml,
    OpenCL,
    Pascal83,
    PLI,
    Python,
    RenderScript,
    Rust,
    SPIRConstant,
    SPIRGeneric,
    SPIRGlobal,
    SPIRLocal,
    SPIRPrivate,
    SYCLCBuiltin,
    SYCLCGeneric,
    SYCLCGlobal,
    SYCLCLocal,
    SYCLCWorkItemGroupLocal,
    SYCLConstant,
    SYCLGeneric,
    SYCLGlobal,
    SYCLLocal,
    SYCLPrivate,
    SYCLWorkItemGroupLocal,
    Swift,
    UPC,
    Zig,
}
```
```rust
pub enum FileType {
    Assembly,
    Object,
}
```
```rust
pub enum FloatPredicate {
    OEQ,
    OGE,
    OGT,
    OLE,
    OLT,
    ONE,
    ORD,
    PredicateFalse,
    PredicateTrue,
    UEQ,
    UGE,
    UGT,
    ULE,
    ULT,
    UNE,
    UNO,
}
```
```rust
pub enum FunctionLookupError {
    FunctionNotFound,
    JITNotEnabled,
}
```
```rust
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}
```
```rust
pub enum InlineAsmDialect {
    ATT,
    Intel,
}
```
```rust
pub enum IntPredicate {
    EQ,
    NE,
    SGE,
    SGT,
    SLE,
    SLT,
    UGE,
    UGT,
    ULE,
    ULT,
}
```
```rust
pub enum LoadLibraryError {
    LoadingError,
    UnicodeError,
}
```
```rust
pub enum OptimizationLevel {
    Aggressive,
    Default,
    Less,
    None,
}
```
```rust
pub enum RemoveModuleError {
    IncorrectModuleOwner,
    LLVMError(LLVMString),
    ModuleNotOwned,
}
```
```rust
pub enum RelocMode {
    Default,
    DynamicNoPic,
    PIC,
    Static,
}
```
```rust
pub enum StringRadix {
    Alphanumeric,
    Binary,
    Decimal,
    Hexadecimal,
    Octal,
}
```
```rust
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    InitialExecTLSModel,
    LocalDynamicTLSModel,
    LocalExecTLSModel,
}
```
```rust
pub enum UnnamedAddress {
    Global,
    Local,
    None,
}
```
```rust
pub enum LLVMTailCallKind {
    MustTail,
    NoTail,
    None,
    Tail,
}
```
```rust
pub enum DIFlags {
    Zero,
    Private,
    Protected,
    Public,
    FwdDecl,
    AppleBlock,
    Virtual,
    Artificial,
    Explicit,
    Prototyped,
    ObjcClassComplete,
    ObjectPointer,
    Vector,
    StaticMember,
    LValueReference,
    RValueReference,
    Reserved,
    SingleInheritance,
    MultipleInheritance,
    VirtualInheritance,
    IntroducedVirtual,
    BitField,
    NoReturn,
    TypePassByValue,
    TypePassByReference,
    Thunk,
    IndirectVirtualBase,
}
```
```rust
pub enum DWARFEmissionKind {
    None,
    Full,
    LineTablesOnly,
}
```
```rust
pub enum DWARFSourceLanguage {
    Ada2005,
    Ada2012,
    Ada95,
    Ada83,
    BLISS,
    BORLANDDelphi,
    C,
    C11,
    C17,
    C89,
    C99,
    CPlusPlus,
    CPlusPlus03,
    CPlusPlus11,
    CPlusPlus14,
    CPlusPlus17,
    CPlusPlus20,
    Cobol74,
    Cobol85,
    Crystal,
    D,
    Dylan,
    Fortran03,
    Fortran08,
    Fortran18,
    Fortran77,
    Fortran90,
    Fortran95,
    GOOGLERenderScript,
    Go,
    Haskell,
    Java,
    Julia,
    Kotlin,
    MipsAssembler,
    Modula2,
    Modula3,
    Mojo,
    OCaml,
    OpenCL,
    Pascal83,
    PLI,
    Python,
    RenderScript,
    Rust,
    SPIRConstant,
    SPIRGeneric,
    SPIRGlobal,
    SPIRLocal,
    SPIRPrivate,
    SYCLCBuiltin,
    SYCLCGeneric,
    SYCLCGlobal,
    SYCLCLocal,
    SYCLCWorkItemGroupLocal,
    SYCLConstant,
    SYCLGeneric,
    SYCLGlobal,
    SYCLLocal,
    SYCLPrivate,
    SYCLWorkItemGroupLocal,
    Swift,
    UPC,
    Zig,
}
```
```rust
pub enum InlineAsmDialect {
    ATT,
    Intel,
}
```
```rust
pub enum LLVMTailCallKind {
    MustTail,
    NoTail,
    None,
    Tail,
}
```
```rust
pub enum OptimizationLevel {
    Aggressive,
    Default,
    Less,
    None,
}
```
```rust
pub enum Version {
    Version100,
    Version110,
    Version120,
    Version130,
    Version140,
    Version150,
    Version160,
    Version170,
    Version180,
    Version40,
    Version50,
    Version60,
    Version70,
    Version80,
    Version90,
}
```
```rust
pub enum AttributeLoc {
    Function,
    Param(u32),
    Return,
}
```
```rust
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}
```
```rust
pub enum CodeModel {
    Default,
    JITDefault,
    Kernel,
    Large,
    Medium,
    Small,
}
```
```rust
pub enum DLLStorageClass {
    Default,
    Export,
    Import,
}
```
```rust
pub enum FileType {
    Assembly,
    Object,
}
```
```rust
pub enum FloatPredicate {
    OEQ,
    OGE,
    OGT,
    OLE,
    OLT,
    ONE,
    ORD,
    PredicateFalse,
    PredicateTrue,
    UEQ,
    UGE,
    UGT,
    ULE,
    ULT,
    UNE,
    UNO,
}
```
```rust
pub enum FunctionLookupError {
    FunctionNotFound,
    JITNotEnabled,
}
```
```rust
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}
```
```rust
pub enum IntPredicate {
    EQ,
    NE,
    SGE,
    SGT,
    SLE,
    SLT,
    UGE,
    UGT,
    ULE,
    ULT,
}
```
```rust
pub enum LoadLibraryError {
    LoadingError,
    UnicodeError,
}
```
```rust
pub enum OptimizationLevel {
    Aggressive,
    Default,
    Less,
    None,
}
```
```rust
pub enum RemoveModuleError {
    IncorrectModuleOwner,
    LLVMError(LLVMString),
    ModuleNotOwned,
}
```
```rust
pub enum RelocMode {
    Default,
    DynamicNoPic,
    PIC,
    Static,
}
```
```rust
pub enum StringRadix {
    Alphanumeric,
    Binary,
    Decimal,
    Hexadecimal,
    Octal,
}
```
```rust
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    InitialExecTLSModel,
    LocalDynamicTLSModel,
    LocalExecTLSModel,
}
```
```rust
pub enum UnnamedAddress {
    Global,
    Local,
    None,
}
```
```rust
pub enum LLVMTailCallKind {
    MustTail,
    NoTail,
    None,
    Tail,
}
```
```rust
pub enum PassBuilderOptionsPassBuilderOptionsRefEnum {
    CGSCCAnalysisManager,
    FunctionAnalysisManager,
    FunctionPassManager(PassManager<FunctionValue<'ctx>>),
    LoopAnalysisManager,
    ModuleAnalysisManager,
    PassBuilderOptions(PassBuilderOptions),
}
```
```rust
pub enum VersionRange {
    Infinite,
    VersionRange(Range<Version>),
    VersionRangeInclusive(RangeInclusive<Version>),
    VersionSpecific(Version),
}
```
```rust
pub enum Version {
    Version100,
    Version110,
    Version120,
    Version130,
    Version140,
    Version150,
    Version160,
    Version170,
    Version180,
    Version40,
    Version50,
    Version60,
    Version70,
    Version80,
    Version90,
    VersionLatest,
}
```
```rust
pub enum AddressSpace {
    AMDGPUAddressSpaceFlat = 5,
    AMDGPUCodeObjectConstant = 9,
    AMDGPUConstant = 4,
    AMDGPUDispatchPtr = 6,
    AMDGPUFlatGlobalBuffer = 11,
    AMDGPUGlobal = 1,
    AMDGPUGlobalBuffer = 10,
    AMDGPUKernarg = 4,
    AMDGPULocal = 3,
    AMDGPUParam = 2,
    AMDGPUPrivate = 5,
    AMDGPUQueuePtr = 7,
    AMDGPU তরঙ্গ面 = 8,
    CudaConstant = 4,
    CudaGeneric = 256,
    CudaLocal = 5,
    CudaShared = 3,
    Generic = 0,
    NVCPUGeneric = 256,
    NVCPUGlobal = 1,
    OpenCLConstant = 3,
    OpenCLGeneric = 4,
    OpenCLGlobal = 1,
    OpenCLLocal = 2,
    RenderScript = 1,
    SYCLCBuiltin = 6,
    SYCLCGeneric = 0,
    SYCLCGlobal = 1,
    SYCLCLocal = 5,
    SYCLCWorkItemGroupLocal = 2,
    SYCLConstant = 4,
    SYCLGeneric = 0,
    SYCLGlobal = 1,
    SYCLLocal = 5,
    SYCLPrivate = 3,
    SYCLWorkItemGroupLocal = 2,
    SPIRConstant = 4,
    SPIRGeneric = 0,
    SPIRGlobal = 1,
    SPIRLocal = 5,
    SPIRPrivate = 3,
    SPIRWorkgroupLocal = 2,
    推测 = 256,
}
```
```rust
pub enum AtomicOrdering {
    Acquire,
    AcquireRelease,
    Monotonic,
    NotAtomic,
    Release,
    SequentiallyConsistent,
    Unordered,
}
```
```rust
pub enum AtomicRMWBinOp {
    Add,
    And,
    FAdd,
    FMax,
    FMin,
    FSub,
    Max,
    Min,
    Nand,
    Or,
    SameSize,
    Sub,
    UMax,
    UMin,
    Xchg,
    Xor,
}
```
```rust
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}
```
```rust
pub enum CodeModel {
    Default,
    JITDefault,
    Kernel,
    Large,
    Medium,
    Small,
}
```
```rust
pub enum ComdatSelectionKind {
    Any,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
```
```rust
pub enum DLLStorageClass {
    Default,
    Export,
    Import,
}
```
```rust
pub enum DWARFEmissionKind {
    Full,
    LineTablesOnly,
    None,
}
```
```rust
pub enum DWARFSourceLanguage {
    Ada2005,
    Ada2012,
    Ada95,
    Ada83,
    BLISS,
    BORLANDDelphi,
    C,
    C11,
    C17,
    C89,
    C99,
    CPlusPlus,
    CPlusPlus03,
    CPlusPlus11,
    CPlusPlus14,
    CPlusPlus17,
    CPlusPlus20,
    Cobol74,
    Cobol85,
    Crystal,
    D,
    Dylan,
    Fortran03,
    Fortran08,
    Fortran18,
    Fortran77,
    Fortran90,
    Fortran95,
    GOOGLERenderScript,
    Go,
    Haskell,
    Java,
    Julia,
    Kotlin,
    MipsAssembler,
    Modula2,
    Modula3,
    Mojo,
    OCaml,
    OpenCL,
    Pascal83,
    PLI,
    Python,
    RenderScript,
    Rust,
    SPIRConstant,
    SPIRGeneric,
    SPIRGlobal,
    SPIRLocal,
    SPIRPrivate,
    SYCLCBuiltin,
    SYCLCGeneric,
    SYCLCGlobal,
    SYCLCLocal,
    SYCLCWorkItemGroupLocal,
    SYCLConstant,
    SYCLGeneric,
    SYCLGlobal,
    SYCLLocal,
    SYCLPrivate,
    SYCLWorkItemGroupLocal,
    Swift,
    UPC,
    Zig,
}
```
```rust
pub enum FileType {
    Assembly,
    Object,
}
```
```rust
pub enum FloatPredicate {
    OEQ,
    OGE,
    OGT,
    OLE,
    OLT,
    ONE,
    ORD,
    PredicateFalse,
    PredicateTrue,
    UEQ,
    UGE,
    UGT,
    ULE,
    ULT,
    UNE,
    UNO,
}
```
```rust
pub enum FunctionLookupError {
    FunctionNotFound,
    JITNotEnabled,
}
```
```rust
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}
```
```rust
pub enum InlineAsmDialect {
    ATT,
    Intel,
}
```
```rust
pub enum IntPredicate {
    EQ,
    NE,
    SGE,
    SGT,
    SLE,
    SLT,
    UGE,
    UGT,
    ULE,
    ULT,
}
```
```rust
pub enum LoadLibraryError {
    LoadingError,
    UnicodeError,
}
```
```rust
pub enum OptimizationLevel {
    Aggressive,
    Default,
    Less,
    None,
}
```
```rust
pub enum RemoveModuleError {
    IncorrectModuleOwner,
    LLVMError(LLVMString),
    ModuleNotOwned,
}
```
```rust
pub enum RelocMode {
    Default,
    DynamicNoPic,
    PIC,
    Static,
}
```
```rust
pub enum StringRadix {
    Alphanumeric,
    Binary,
    Decimal,
    Hexadecimal,
    Octal,
}
```
```rust
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    InitialExecTLSModel,
    LocalDynamicTLSModel,
    LocalExecTLSModel,
}
```
```rust
pub enum UnnamedAddress {
    Global,
    Local,
    None,
}
```
```rust
pub enum LLVMTailCallKind {
    MustTail,
    NoTail,
    None,
    Tail,
}
```
```rust
pub enum PassBuilderOptionsPassBuilderOptionsRefEnum {
    CGSCCAnalysisManager,
    FunctionAnalysisManager,
    FunctionPassManager(PassManager<FunctionValue<'ctx>>),
    LoopAnalysisManager,
    ModuleAnalysisManager,
    PassBuilderOptions(PassBuilderOptions),
}
```
```rust
pub enum VersionRange {
    Infinite,
    VersionRange(Range<Version>),
    VersionRangeInclusive(RangeInclusive<Version>),
    VersionSpecific(Version),
}
```
```rust
pub enum Version {
    Version100,
    Version110,
    Version120,
    Version130,
    Version140,
    Version150,
    Version160,
    Version170,
    Version180,
    Version40,
    Version50,
    Version60,
    Version70,
    Version80,
    Version90,
    VersionLatest,
}
```
```rust
pub enum AddressSpace {
    AMDGPUAddressSpaceFlat = 5,
    AMDGPUCodeObjectConstant = 9,
    AMDGPUConstant = 4,
    AMDGPUDispatchPtr = 6,
    AMDGPUFlatGlobalBuffer = 11,
    AMDGPUGlobal = 1,
    AMDGPUGlobalBuffer = 10,
    AMDGPUKernarg = 4,
    AMDGPULocal = 3,
    AMDGPUParam = 2,
    AMDGPUPrivate = 5,
    AMDGPUQueuePtr = 7,
    AMDGPU तरंग面 = 8,
    CudaConstant = 4,
    CudaGeneric = 256,
    CudaLocal = 5,
    CudaShared = 3,
    Generic = 0,
    NVCPUGeneric = 256,
    NVCPUGlobal = 1,
    OpenCLConstant = 3,
    OpenCLGeneric = 4,
    OpenCLGlobal = 1,
    OpenCLLocal = 2,
    RenderScript = 1,
    SYCLCBuiltin = 6,
    SYCLCGeneric = 0,
    SYCLCGlobal = 1,
    SYCLCLocal = 5,
    SYCLCWorkItemGroupLocal = 2,
    SYCLConstant = 4,
    SYCLGeneric = 0,
    SYCLGlobal = 1,
    SYCLLocal = 5,
    SYCLPrivate = 3,
    SYCLWorkItemGroupLocal = 2,
    SPIRConstant = 4,
    SPIRGeneric = 0,
    SPIRGlobal = 1,
    SPIRLocal = 5,
    SPIRPrivate = 3,
    SPIRWorkgroupLocal = 2,
    推测 = 256,
}
```
```rust
pub enum AtomicOrdering {
    Acquire,
    AcquireRelease,
    Monotonic,
    NotAtomic,
    Release,
    SequentiallyConsistent,
    Unordered,
}
```
```rust
pub enum AtomicRMWBinOp {
    Add,
    And,
    FAdd,
    FMax,
    FMin,
    FSub,
    Max,
    Min,
    Nand,
    Or,
    SameSize,
    Sub,
    UMax,
    UMin,
    Xchg,
    Xor,
}
```
```rust
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}
```
```rust
pub enum CodeModel {
    Default,
    JITDefault,
    Kernel,
    Large,
    Medium,
    Small,
}
```
```rust
pub enum ComdatSelectionKind {
    Any,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
```
```rust
pub enum DLLStorageClass {
    Default,
    Export,
    Import,
}
```
```rust
pub enum DWARFEmissionKind {
    Full,
    LineTablesOnly,
    None,
}
```
```rust
pub enum DWARFSourceLanguage {
    Ada2005,
    Ada2012,
    Ada95,
    Ada83,
    BLISS,
    BORLANDDelphi,
    C,
    C11,
    C17,
    C89,
    C99,
    CPlusPlus,
    CPlusPlus03,
    CPlusPlus11,
    CPlusPlus14,
    CPlusPlus17,
    CPlusPlus20,
    Cobol74,
    Cobol85,
    Crystal,
    D,
    Dylan,
    Fortran03,
    Fortran08,
    Fortran18,
    Fortran77,
    Fortran90,
    Fortran95,
    GOOGLERenderScript,
    Go,
    Haskell,
    Java,
    Julia,
    Kotlin,
    MipsAssembler,
    Modula2,
    Modula3,
    Mojo,
    OCaml,
    OpenCL,
    Pascal83,
    PLI,
    Python,
    RenderScript,
    Rust,
    SPIRConstant,
    SPIRGeneric,
    SPIRGlobal,
    SPIRLocal,
    SPIRPrivate,
    SYCLCBuiltin,
    SYCLCGeneric,
    SYCLCGlobal,
    SYCLCLocal,
    SYCLCWorkItemGroupLocal,
    SYCLConstant,
    SYCLGeneric,
    SYCLGlobal,
    SYCLLocal,
    SYCLPrivate,
    SYCLWorkItemGroupLocal,
    Swift,
    UPC,
    Zig,
}
```
```rust
pub enum FileType {
    Assembly,
    Object,
}
```
```rust
pub enum FloatPredicate {
    OEQ,
    OGE,
    OGT,
    OLE,
    OLT,
    ONE,
    ORD,
    PredicateFalse,
    PredicateTrue,
    UEQ,
    UGE,
    UGT,
    ULE,
    ULT,
    UNE,
    UNO,
}
```
```rust
pub enum FunctionLookupError {
    FunctionNotFound,
    JITNotEnabled,
}
```
```rust
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}
```
```rust
pub enum InlineAsmDialect {
    ATT,
    Intel,
}
```
```rust
pub enum IntPredicate {
    EQ,
    NE,
    SGE,
    SGT,
    SLE,
    SLT,
    UGE,
    UGT,
    ULE,
    ULT,
}
```
```rust
pub enum LoadLibraryError {
    LoadingError,
    UnicodeError,
}
```
```rust
pub enum OptimizationLevel {
    Aggressive,
    Default,
    Less,
    None,
}
```
```rust
pub enum RemoveModuleError {
    IncorrectModuleOwner,
    LLVMError(LLVMString),
    ModuleNotOwned,
}
```
```rust
pub enum RelocMode {
    Default,
    DynamicNoPic,
    PIC,
    Static,
}
```
```rust
pub enum StringRadix {
    Alphanumeric,
    Binary,
    Decimal,
    Hexadecimal,
    Octal,
}
```
```rust
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    InitialExecTLSModel,
    LocalDynamicTLSModel,
    LocalExecTLSModel,
}
```
```rust
pub enum UnnamedAddress {
    Global,
    Local,
    None,
}
```
```rust
pub enum LLVMTailCallKind {
    MustTail,
    NoTail,
    None,
    Tail,
}
```
```rust
pub enum PassBuilderOptionsPassBuilderOptionsRefEnum {
    CGSCCAnalysisManager,
    FunctionAnalysisManager,
    FunctionPassManager(PassManager<FunctionValue<'ctx>>),
    LoopAnalysisManager,
    ModuleAnalysisManager,
    PassBuilderOptions(PassBuilderOptions),
}
```
```rust
pub enum VersionRange {
    Infinite,
    VersionRange(Range<Version>),
    VersionRangeInclusive(RangeInclusive<Version>),
    VersionSpecific(Version),
}
```
```rust
pub enum Version {
    Version100,
    Version110,
    Version120,
    Version130,
    Version140,
    Version150,
    Version160,
    Version170,
    Version180,
    Version40,
    Version50,
    Version60,
    Version70,
    Version80,
    Version90,
    VersionLatest,
}
```
```rust
pub enum AddressSpace {
    AMDGPUAddressSpaceFlat = 5,
    AMDGPUCodeObjectConstant = 9,
    AMDGPUConstant = 4,
    AMDGPUDispatchPtr = 6,
    AMDGPUFlatGlobalBuffer = 11,
    AMDGPUGlobal = 1,
    AMDGPUGlobalBuffer = 10,
    AMDGPUKernarg = 4,
    AMDGPULocal = 3,
    AMDGPUParam = 2,
    AMDGPUPrivate = 5,
    AMDGPUQueuePtr = 7,
    AMDGPU तरंग面 = 8,
    CudaConstant = 4,
    CudaGeneric = 256,
    CudaLocal = 5,
    CudaShared = 3,
    Generic = 0,
    NVCPUGeneric = 256,
    NVCPUGlobal = 1,
    OpenCLConstant = 3,
    OpenCLGeneric = 4,
    OpenCLGlobal = 1,
    OpenCLLocal = 2,
    RenderScript = 1,
    SYCLCBuiltin = 6,
    SYCLCGeneric = 0,
    SYCLCGlobal = 1,
    SYCLCLocal = 5,
    SYCLCWorkItemGroupLocal = 2,
    SYCLConstant = 4,
    SYCLGeneric = 0,
    SYCLGlobal = 1,
    SYCLLocal = 5,
    SYCLPrivate = 3,
    SYCLWorkItemGroupLocal = 2,
    SPIRConstant = 4,
    SPIRGeneric = 0,
    SPIRGlobal = 1,
    SPIRLocal = 5,
    SPIRPrivate = 3,
    SPIRWorkgroupLocal = 2,
    推测 = 256,
}
```
```rust
pub enum AtomicOrdering {
    Acquire,
    AcquireRelease,
    Monotonic,
    NotAtomic,
    Release,
    SequentiallyConsistent,
    Unordered,
}
```
```rust
pub enum AtomicRMWBinOp {
    Add,
    And,
    FAdd,
    FMax,
    FMin,
    FSub,
    Max,
    Min,
    Nand,
    Or,
    SameSize,
    Sub,
    UMax,
    UMin,
    Xchg,
    Xor,
}
```
```rust
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}
```
```rust
pub enum CodeModel {
    Default,
    JITDefault,
    Kernel,
    Large,
    Medium,
    Small,
}
```
```rust
pub enum ComdatSelectionKind {
    Any,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
```
```rust
pub enum DLLStorageClass {
    Default,
    Export,
    Import,
}
```
```rust
pub enum DWARFEmissionKind {
    Full,
    LineTablesOnly,
    None,
}
```
```rust
pub enum DWARFSourceLanguage {
    Ada2005,
    Ada2012,
    Ada95,
    Ada83,
    BLISS,
    BORLANDDelphi,
    C,
    C11,
    C17,
    C89,
    C99,
    CPlusPlus,
    CPlusPlus03,
    CPlusPlus11,
    CPlusPlus14,
    CPlusPlus17,
    CPlusPlus20,
    Cobol74,
    Cobol85,
    Crystal,
    D,
    Dylan,
    Fortran03,
    Fortran08,
    Fortran18,
    Fortran77,
    Fortran90,
    Fortran95,
    GOOGLERenderScript,
    Go,
    Haskell,
    Java,
    Julia,
    Kotlin,
    MipsAssembler,
    Modula2,
    Modula3,
    Mojo,
    OCaml,
    OpenCL,
    Pascal83,
    PLI,
    Python,
    RenderScript,
    Rust,
    SPIRConstant,
    SPIRGeneric,
    SPIRGlobal,
    SPIRLocal,
    SPIRPrivate,
    SYCLCBuiltin,
    SYCLCGeneric,
    SYCLCGlobal,
    SYCLCLocal,
    SYCLCWorkItemGroupLocal,
    SYCLConstant,
    SYCLGeneric,
    SYCLGlobal,
    SYCLLocal,
    SYCLPrivate,
    SYCLWorkItemGroupLocal,
    Swift,
    UPC,
    Zig,
}
```
```rust
pub enum FileType {
    Assembly,
    Object,
}
```
```rust
pub enum FloatPredicate {
    OEQ,
    OGE,
    OGT,
    OLE,
    OLT,
    ONE,
    ORD,
    PredicateFalse,
    PredicateTrue,
    UEQ,
    UGE,
    UGT,
    ULE,
    ULT,
    UNE,
    UNO,
}
```
```rust
pub enum FunctionLookupError {
    FunctionNotFound,
    JITNotEnabled,
}
```
```rust
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}
```
```rust
pub enum InlineAsmDialect {
    ATT,
    Intel,
}
```
```rust
pub enum IntPredicate {
    EQ,
    NE,
    SGE,
    SGT,
    SLE,
    SLT,
    UGE,
    UGT,
    ULE,
    ULT,
}
```
```rust
pub enum LoadLibraryError {
    LoadingError,
    UnicodeError,
}
```
```rust
pub enum OptimizationLevel {
    Aggressive,
    Default,
    Less,
    None,
}
```
```rust
pub enum RemoveModuleError {
    IncorrectModuleOwner,
    LLVMError(LLVMString),
    ModuleNotOwned,
}
```
```rust
pub enum RelocMode {
    Default,
    DynamicNoPic,
    PIC,
    Static,
}
```
```rust
pub enum StringRadix {
    Alphanumeric,
    Binary,
    Decimal,
    Hexadecimal,
    Octal,
}
```
```rust
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    InitialExecTLSModel,
    LocalDynamicTLSModel,
    LocalExecTLSModel,
}
```
```rust
pub enum UnnamedAddress {
    Global,
    Local,
    None,
}
```
```rust
pub enum LLVMTailCallKind {
    MustTail,
    NoTail,
    None,
    Tail,
}
```
```rust
pub enum PassBuilderOptionsPassBuilderOptionsRefEnum {
    CGSCCAnalysisManager,
    FunctionAnalysisManager,
    FunctionPassManager(PassManager<FunctionValue<'ctx>>),
    LoopAnalysisManager,
    ModuleAnalysisManager,
    PassBuilderOptions(PassBuilderOptions),
}
```
```rust
pub enum VersionRange {
    Infinite,
    VersionRange(Range<Version>),
    VersionRangeInclusive(RangeInclusive<Version>),
    VersionSpecific(Version),
}
```
```rust
pub enum Version {
    Version100,
    Version110,
    Version120,
    Version130,
    Version140,
    Version150,
    Version160,
    Version170,
    Version180,
    Version40,
    Version50,
    Version60,
    Version70,
    Version80,
    Version90,
    VersionLatest,
}
```
```rust
pub enum AddressSpace {
    AMDGPUAddressSpaceFlat = 5,
    AMDGPUCodeObjectConstant = 9,
    AMDGPUConstant = 4,
    AMDGPUDispatchPtr = 6,
    AMDGPUFlatGlobalBuffer = 11,
    AMDGPUGlobal = 1,
    AMDGPUGlobalBuffer = 10,
    AMDGPUKernarg = 4,
    AMDGPULocal = 3,
    AMDGPUParam = 2,
    AMDGPUPrivate = 5,
    AMDGPUQueuePtr = 7,
    AMDGPU तरंग面 = 8,
    CudaConstant = 4,
    CudaGeneric = 256,
    CudaLocal = 5,
    CudaShared = 3,
    Generic = 0,
    NVCPUGeneric = 256,
    NVCPUGlobal = 1,
    OpenCLConstant = 3,
    OpenCLGeneric = 4,
    OpenCLGlobal = 1,
    OpenCLLocal = 2,
    RenderScript = 1,
    SYCLCBuiltin = 6,
    SYCLCGeneric = 0,
    SYCLCGlobal = 1,
    SYCLCLocal = 5,
    SYCLCWorkItemGroupLocal = 2,
    SYCLConstant = 4,
    SYCLGeneric = 0,
    SYCLGlobal = 1,
    SYCLLocal = 5,
    SYCLPrivate = 3,
    SYCLWorkItemGroupLocal = 2,
    SPIRConstant = 4,
    SPIRGeneric = 0,
    SPIRGlobal = 1,
    SPIRLocal = 5,
    SPIRPrivate = 3,
    SPIRWorkgroupLocal = 2,
    推测 = 256,
}
```
```rust
pub enum AtomicOrdering {
    Acquire,
    AcquireRelease,
    Monotonic,
    NotAtomic,
    Release,
    SequentiallyConsistent,
    Unordered,
}
```
```rust
pub enum AtomicRMWBinOp {
    Add,
    And,
    FAdd,
    FMax,
    FMin,
    FSub,
    Max,
    Min,
    Nand,
    Or,
    SameSize,
    Sub,
    UMax,
    UMin,
    Xchg,
    Xor,
}
```
```rust
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}
```
```rust
pub enum CodeModel {
    Default,
    JITDefault,
    Kernel,
    Large,
    Medium,
    Small,
}
```
```rust
pub enum ComdatSelectionKind {
    Any,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
```
```rust
pub enum DLLStorageClass {
    Default,
    Export,
    Import,
}
```
```rust
pub enum DWARFEmissionKind {
    Full,
    LineTablesOnly,
    None,
}
```
```rust
pub enum DWARFSourceLanguage {
    Ada2005,
    Ada2012,
    Ada95,
    Ada83,
    BLISS,
    BORLANDDelphi,
    C,
    C11,
    C17,
    C89,
    C99,
    CPlusPlus,
    CPlusPlus03,
    CPlusPlus11,
    CPlusPlus14,
    CPlusPlus17,
    CPlusPlus20,
    Cobol74,
    Cobol85,
    Crystal,
    D,
    Dylan,
    Fortran03,
    Fortran08,
    Fortran18,
    Fortran77,
    Fortran90,
    Fortran95,
    GOOGLERenderScript,
    Go,
    Haskell,
    Java,
    Julia,
    Kotlin,
    MipsAssembler,
    Modula2,
    Modula3,
    Mojo,
    OCaml,
    OpenCL,
    Pascal83,
    PLI,
    Python,
    RenderScript,
    Rust,
    SPIRConstant,
    SPIRGeneric,
    SPIRGlobal,
    SPIRLocal,
    SPIRPrivate,
    SYCLCBuiltin,
    SYCLCGeneric,
    SYCLCGlobal,
    SYCLCLocal,
    SYCLCWorkItemGroupLocal,
    SYCLConstant,
    SYCLGeneric,
    SYCLGlobal,
    SYCLLocal,
    SYCLPrivate,
    SYCLWorkItemGroupLocal,
    Swift,
    UPC,
    Zig,
}
```
```rust
pub enum FileType {
    Assembly,
    Object,
}
```
```rust
pub enum FloatPredicate {
    OEQ,
    OGE,
    OGT,
    OLE,
    OLT,
    ONE,
    ORD,
    PredicateFalse,
    PredicateTrue,
    UEQ,
    UGE,
    UGT,
    ULE,
    ULT,
    UNE,
    UNO,
}
```
```rust
pub enum FunctionLookupError {
    FunctionNotFound,
    JITNotEnabled,
}
```
```rust
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}
```
```rust
pub enum InlineAsmDialect {
    ATT,
    Intel,
}
```
```rust
pub enum IntPredicate {
    EQ,
    NE,
    SGE,
    SGT,
    SLE,
    SLT,
    UGE,
    UGT,
    ULE,
    ULT,
}
```
```rust
pub enum LoadLibraryError {
    LoadingError,
    UnicodeError,
}
```
```rust
pub enum OptimizationLevel {
    Aggressive,
    Default,
    Less,
    None,
}
```
```rust
pub enum RemoveModuleError {
    IncorrectModuleOwner,
    LLVMError(LLVMString),
    ModuleNotOwned,
}
```
```rust
pub enum RelocMode {
    Default,
    DynamicNoPic,
    PIC,
    Static,
}
```
```rust
pub enum StringRadix {
    Alphanumeric,
    Binary,
    Decimal,
    Hexadecimal,
    Octal,
}
```
```rust
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    InitialExecTLSModel,
    LocalDynamicTLSModel,
    LocalExecTLSModel,
}
```
```rust
pub enum UnnamedAddress {
    Global,
    Local,
    None,
}
```
```rust
pub enum LLVMTailCallKind {
    MustTail,
    NoTail,
    None,
    Tail,
}
```
```rust
pub enum PassBuilderOptionsPassBuilderOptionsRefEnum {
    CGSCCAnalysisManager,
    FunctionAnalysisManager,
    FunctionPassManager(PassManager<FunctionValue<'ctx>>),
    LoopAnalysisManager,
    ModuleAnalysisManager,
    PassBuilderOptions(PassBuilderOptions),
}
```
```rust
pub enum VersionRange {
    Infinite,
    VersionRange(Range<Version>),
    VersionRangeInclusive(RangeInclusive<Version>),
    VersionSpecific(Version),
}
```
```rust
pub enum Version {
    Version100,
    Version110,
    Version120,
    Version130,
    Version140,
    Version150,
    Version160,
    Version170,
    Version180,
    Version40,
    Version50,
    Version60,
    Version70,
    Version80,
    Version90,
    VersionLatest,
}
```
```rust
pub enum AddressSpace {
    AMDGPUAddressSpaceFlat = 5,
    AMDGPUCodeObjectConstant = 9,
    AMDGPUConstant = 4,
    AMDGPUDispatchPtr = 6,
    AMDGPUFlatGlobalBuffer = 11,
    AMDGPUGlobal = 1,
    AMDGPUGlobalBuffer = 10,
    AMDGPUKernarg = 4,
    AMDGPULocal = 3,
    AMDGPUParam = 2,
    AMDGPUPrivate = 5,
    AMDGPUQueuePtr = 7,
    AMDGPU तरंग面 = 8,
    CudaConstant = 4,
    CudaGeneric = 256,
    CudaLocal = 5,
    CudaShared = 3,
    Generic = 0,
    NVCPUGeneric = 256,
    NVCPUGlobal = 1,
    OpenCLConstant = 3,
    OpenCLGeneric = 4,
    OpenCLGlobal = 1,
    OpenCLLocal = 2,
    RenderScript = 1,
    SYCLCBuiltin = 6,
    SYCLCGeneric = 0,
    SYCLCGlobal = 1,
    SYCLCLocal = 5,
    SYCLCWorkItemGroupLocal = 2,
    SYCLConstant = 4,
    SYCLGeneric = 0,
    SYCLGlobal = 1,
    SYCLLocal = 5,
    SYCLPrivate = 3,
    SYCLWorkItemGroupLocal = 2,
    SPIRConstant = 4,
    SPIRGeneric = 0,
    SPIRGlobal = 1,
    SPIRLocal = 5,
    SPIRPrivate = 3,
    SPIRWorkgroupLocal = 2,
    推测 = 256,
}
```
```rust
pub enum AtomicOrdering {
    Acquire,
    AcquireRelease,
    Monotonic,
    NotAtomic,
    Release,
    SequentiallyConsistent,
    Unordered,
}
```
```rust
pub enum AtomicRMWBinOp {
    Add,
    And,
    FAdd,
    FMax,
    FMin,
    FSub,
    Max,
    Min,
    Nand,
    Or,
    SameSize,
    Sub,
    UMax,
    UMin,
    Xchg,
    Xor,
}
```
```rust
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}
```
```rust
pub enum CodeModel {
    Default,
    JITDefault,
    Kernel,
    Large,
    Medium,
    Small,
}
```
```rust
pub enum ComdatSelectionKind {
    Any,
    ExactMatch,
    Largest,
    NoDuplicates,
    SameSize,
}
```
```rust
pub enum DLLStorageClass {
    Default,
    Export,
    Import,
}
```
```rust
pub enum DWARFEmissionKind {
    Full,
    LineTablesOnly,
    None,
}
```
```rust
pub enum DWARFSourceLanguage {
    Ada2005,
    Ada2012,
    Ada95,
    Ada83,
    BLISS,
    BORLANDDelphi,
    C,
    C11,
    C17,
    C89,
    C99,
    CPlusPlus,
    CPlusPlus03,
    CPlusPlus11,
    CPlusPlus14,
    CPlusPlus17,
    CPlusPlus20,
    Cobol74,
    Cobol85,
    Crystal,
    D,
    Dylan,
    Fortran03,
    Fortran08,
    Fortran18,
    Fortran77,
    Fortran90,
    Fortran95,
    GOOGLERenderScript,
    Go,
    Haskell,
    Java,
    Julia,
    Kotlin,
    MipsAssembler,
    Modula2,
    Modula3,
    Mojo,
    OCaml,
    OpenCL,
    Pascal83,
    PLI,
    Python,
    RenderScript,
    Rust,
    SPIRConstant,
    SPIRGeneric,
    SPIRGlobal,
    SPIRLocal,
    SPIRPrivate,
    SYCLCBuiltin,
    SYCLCGeneric,
    SYCLCGlobal,
    SYCLCLocal,
    SYCLCWorkItemGroupLocal,
    SYCLConstant,
    SYCLGeneric,
    SYCLGlobal,
    SYCLLocal,
    SYCLPrivate,
    SYCLWorkItemGroupLocal,
    Swift,
    UPC,
    Zig,
}
```
```rust
pub enum FileType {
    Assembly,
    Object,
}
```
```rust
pub enum FloatPredicate {
    OEQ,
    OGE,
    OGT,
    OLE,
    OLT,
    ONE,
    ORD,
    PredicateFalse,
    PredicateTrue,
    UEQ,
    UGE,
    UGT,
    ULE,
    ULT,
    UNE,
    UNO,
}
```
```rust
pub enum FunctionLookupError {
    FunctionNotFound,
    JITNotEnabled,
}
```
```rust
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}
```
```rust
pub enum InlineAsmDialect {
    ATT,
    Intel,
}
```
```rust
pub enum IntPredicate {
    EQ,
    NE,
    SGE,
    SGT,
    SLE,
    SLT,
    UGE,
    UGT,
    ULE,
    ULT,
}
```
```rust
pub enum LoadLibraryError {
    LoadingError,
    UnicodeError,
}
```
```rust
pub enum OptimizationLevel {
    Aggressive,
    Default,
    Less,
    None,
}
```
```rust
pub enum RemoveModuleError {
    IncorrectModuleOwner,
    LLVMError(LLVMString),
    ModuleNotOwned,
}
```
```rust
pub enum RelocMode {
    Default,
    DynamicNoPic,
    PIC,
    Static,
}
```
```rust
pub enum StringRadix {
    Alphanumeric,
    Binary,
    Decimal,
    Hexadecimal,
    Octal,
}
```
```rust
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    InitialExecTLSModel,
    LocalDynamicTLSModel,
    LocalExecTLSModel,
}
```
```rust
pub enum UnnamedAddress {
    Global,
    Local,
    None,
}
```
```rust
pub enum LLVMTailCallKind {
    MustTail,
    NoTail,
    None,
    Tail,
}
```
```rust
pub enum PassBuilderOptionsPassBuilderOptionsRefEnum {
    CGSCCAnalysisManager,
    FunctionAnalysisManager,
    FunctionPassManager(PassManager<FunctionValue<'ctx>>),
    LoopAnalysisManager,
    ModuleAnalysisManager,
    PassBuilderOptions(PassBuilderOptions),
}
```
```rust
pub enum VersionRange {
    Infinite,
    VersionRange(Range<Version>),
    VersionRangeInclusive(RangeInclusive<Version>),
    VersionSpecific(Version),
}
```
```rust
pub enum Version {
    Version100,
    Version110,
    Version120,
    Version130,
    Version140,
    Version150,
    Version160,
    Version170,
    Version180,
    Version40,
    Version50,
    Version60,
    Version70,
    Version80,
    Version90,
    VersionLatest,
}
```


`src/attributes.rs`
```rust
pub struct Attribute {
    pub(crate) attribute: LLVMAttributeRef,
}

impl std::fmt::Debug for Attribute
pub unsafe fn new(attribute: LLVMAttributeRef) -> Self
pub fn as_mut_ptr(&self) -> LLVMAttributeRef
pub fn is_enum(self) -> bool
pub fn is_string(self) -> bool
pub fn is_type(self) -> bool
fn is_type(self) -> bool
pub fn get_named_enum_kind_id(name: &str) -> u32
#[llvm_versions(..=11)]
pub fn get_enum_kind_id(self) -> u32
#[llvm_versions(12..)]
pub fn get_enum_kind_id(self) -> u32
#[llvm_versions(..=11)]
fn get_enum_kind_id_is_valid(self) -> bool
#[llvm_versions(12..)]
fn get_enum_kind_id_is_valid(self) -> bool
pub fn get_last_enum_kind_id() -> u32
pub fn get_enum_value(self) -> u64
pub fn get_string_kind_id(&self) -> &CStr
pub fn get_string_value(&self) -> &CStr
#[llvm_versions(12..)]
pub fn get_type_value(&self) -> AnyTypeEnum
#[llvm_versions(..12)]
fn get_type_value(&self)
pub(crate) fn get_index(self) -> u32

```
`src/basic_block.rs`
```rust
pub struct BasicBlock<'ctx> {
    pub(crate) basic_block: LLVMBasicBlockRef,
    _marker: PhantomData<&'ctx ()>,
}

impl<'ctx> BasicBlock<'ctx> {
    pub(crate) unsafe fn new(basic_block: LLVMBasicBlockRef) -> Option<Self>
    pub fn as_mut_ptr(&self) -> LLVMBasicBlockRef
    pub fn get_parent(self) -> Option<FunctionValue<'ctx>>
    pub fn get_previous_basic_block(self) -> Option<BasicBlock<'ctx>>
    pub fn get_next_basic_block(self) -> Option<BasicBlock<'ctx>>
    pub fn move_before(self, basic_block: BasicBlock<'ctx>) -> Result<(), ()>
    pub fn move_after(self, basic_block: BasicBlock<'ctx>) -> Result<(), ()>
    pub fn get_first_instruction(self) -> Option<InstructionValue<'ctx>>
    pub fn get_last_instruction(self) -> Option<InstructionValue<'ctx>>
    pub fn get_instruction_with_name(self, name: &str) -> Option<InstructionValue<'ctx>>
    pub fn get_terminator(self) -> Option<InstructionValue<'ctx>>
    pub fn get_instructions(self) -> InstructionIter<'ctx>
    pub fn remove_from_function(self) -> Result<(), ()>
    pub unsafe fn delete(self) -> Result<(), ()>
    pub fn get_context(self) -> ContextRef<'ctx>
    pub fn get_name(&self) -> &CStr
    pub fn set_name(&self, name: &str)
    pub fn replace_all_uses_with(self, other: &BasicBlock<'ctx>)
    pub fn get_first_use(self) -> Option<BasicValueUse<'ctx>>
    pub unsafe fn get_address(self) -> Option<PointerValue<'ctx>>
}

impl fmt::Debug for BasicBlock<'_>
pub struct InstructionIter<'ctx>(Option<InstructionValue<'ctx>>);

impl<'ctx> Iterator for InstructionIter<'ctx>

```
`src/comdat.rs`
```rust
#[llvm_enum(LLVMComdatSelectionKind)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ComdatSelectionKind {
    #[llvm_variant(LLVMAnyComdatSelectionKind)]
    Any,
    #[llvm_variant(LLVMExactMatchComdatSelectionKind)]
    ExactMatch,
    #[llvm_variant(LLVMLargestComdatSelectionKind)]
    Largest,
    #[llvm_variant(LLVMNoDuplicatesComdatSelectionKind)]
    NoDuplicates,
    #[llvm_variant(LLVMSameSizeComdatSelectionKind)]
    SameSize,
}

pub struct Comdat(pub(crate) LLVMComdatRef);

impl Comdat {
    pub unsafe fn new(comdat: LLVMComdatRef) -> Self
    pub fn as_mut_ptr(&self) -> LLVMComdatRef
    pub fn get_selection_kind(self) -> ComdatSelectionKind
    pub fn set_selection_kind(self, kind: ComdatSelectionKind)
}

```
`src/context.rs`
```rust
pub(crate) struct ContextImpl(pub(crate) LLVMContextRef);

impl ContextImpl {
    pub(crate) unsafe fn new(context: LLVMContextRef) -> Self
    fn create_builder<'ctx>(&self) -> Builder<'ctx>
    fn create_module<'ctx>(&self, name: &str) -> Module<'ctx>
    fn create_module_from_ir<'ctx>(&self, memory_buffer: MemoryBuffer) -> Result<Module<'ctx>, LLVMString>
    fn create_inline_asm<'ctx>(
    fn void_type<'ctx>(&self) -> VoidType<'ctx>
    fn bool_type<'ctx>(&self) -> IntType<'ctx>
    fn i8_type<'ctx>(&self) -> IntType<'ctx>
    fn i16_type<'ctx>(&self) -> IntType<'ctx>
    fn i32_type<'ctx>(&self) -> IntType<'ctx>
    fn i64_type<'ctx>(&self) -> IntType<'ctx>
    fn i128_type<'ctx>(&self) -> IntType<'ctx>
    fn custom_width_int_type<'ctx>(&self, bits: u32) -> IntType<'ctx>
    #[llvm_versions(6..)]
    fn metadata_type<'ctx>(&self) -> MetadataType<'ctx>
    fn ptr_sized_int_type<'ctx>(&self, target_data: &TargetData, address_space: Option<AddressSpace>) -> IntType<'ctx>
    fn f16_type<'ctx>(&self) -> FloatType<'ctx>
    fn f32_type<'ctx>(&self) -> FloatType<'ctx>
    fn f64_type<'ctx>(&self) -> FloatType<'ctx>
    fn x86_f80_type<'ctx>(&self) -> FloatType<'ctx>
    fn f128_type<'ctx>(&self) -> FloatType<'ctx>
    fn ppc_f128_type<'ctx>(&self) -> FloatType<'ctx>
    #[llvm_versions(15..)]
    fn ptr_type<'ctx>(&self, address_space: AddressSpace) -> PointerType<'ctx>
    fn struct_type<'ctx>(&self, field_types: &[BasicTypeEnum], packed: bool) -> StructType<'ctx>
    fn opaque_struct_type<'ctx>(&self, name: &str) -> StructType<'ctx>
    #[llvm_versions(12..)]
    fn get_struct_type<'ctx>(&self, name: &str) -> Option<StructType<'ctx>>
    fn const_struct<'ctx>(&self, values: &[BasicValueEnum], packed: bool) -> StructValue<'ctx>
    fn append_basic_block<'ctx>(&self, function: FunctionValue<'ctx>, name: &str) -> BasicBlock<'ctx>
    fn insert_basic_block_after<'ctx>(&self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx>
    fn prepend_basic_block<'ctx>(&self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx>
    #[allow(deprecated)]
    fn metadata_node<'ctx>(&self, values: &[BasicMetadataValueEnum<'ctx>]) -> MetadataValue<'ctx>
    #[allow(deprecated)]
    fn metadata_string<'ctx>(&self, string: &str) -> MetadataValue<'ctx>
    fn get_kind_id(&self, key: &str) -> u32
    fn create_enum_attribute(&self, kind_id: u32, val: u64) -> Attribute
    fn create_string_attribute(&self, key: &str, val: &str) -> Attribute
    #[llvm_versions(12..)]
    fn create_type_attribute(&self, kind_id: u32, type_ref: AnyTypeEnum) -> Attribute
    fn const_string<'ctx>(&self, string: &[u8], null_terminated: bool) -> ArrayValue<'ctx>
    fn set_diagnostic_handler(
}

impl PartialEq<Context> for ContextRef<'_>
impl PartialEq<ContextRef<'_>> for Context
pub struct Context {
    pub(crate) context: ContextImpl,
}

unsafe impl Send for Context {}

impl Context {
    pub fn raw(&self) -> LLVMContextRef
    pub unsafe fn new(context: LLVMContextRef) -> Self
    pub fn create() -> Self
    pub unsafe fn get_global<F, R>(func: F) -> R
    #[inline]
    pub fn create_builder(&self) -> Builder
    #[inline]
    pub fn create_module(&self, name: &str) -> Module
    #[inline]
    pub fn create_module_from_ir(&self, memory_buffer: MemoryBuffer) -> Result<Module, LLVMString>
    #[inline]
    pub fn create_inline_asm<'ctx>(
    #[inline]
    pub fn void_type(&self) -> VoidType
    #[inline]
    pub fn bool_type(&self) -> IntType
    #[inline]
    pub fn i8_type(&self) -> IntType
    #[inline]
    pub fn i16_type(&self) -> IntType
    #[inline]
    pub fn i32_type(&self) -> IntType
    #[inline]
    pub fn i64_type(&self) -> IntType
    #[inline]
    pub fn i128_type(&self) -> IntType
    #[inline]
    pub fn custom_width_int_type(&self, bits: u32) -> IntType
    #[inline]
    #[llvm_versions(6..)]
    pub fn metadata_type(&self) -> MetadataType
    #[inline]
    pub fn ptr_sized_int_type(&self, target_data: &TargetData, address_space: Option<AddressSpace>) -> IntType
    #[inline]
    pub fn f16_type(&self) -> FloatType
    #[inline]
    pub fn f32_type(&self) -> FloatType
    #[inline]
    pub fn f64_type(&self) -> FloatType
    #[inline]
    pub fn x86_f80_type(&self) -> FloatType
    #[inline]
    pub fn f128_type(&self) -> FloatType
    #[inline]
    pub fn ppc_f128_type(&self) -> FloatType
    #[llvm_versions(15..)]
    #[inline]
    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType
    #[inline]
    pub fn struct_type(&self, field_types: &[BasicTypeEnum], packed: bool) -> StructType
    #[inline]
    pub fn opaque_struct_type(&self, name: &str) -> StructType
    #[inline]
    #[llvm_versions(12..)]
    pub fn get_struct_type<'ctx>(&self, name: &str) -> Option<StructType<'ctx>>
    #[inline]
    pub fn const_struct(&self, values: &[BasicValueEnum], packed: bool) -> StructValue
    #[inline]
    pub fn append_basic_block<'ctx>(&'ctx self, function: FunctionValue<'ctx>, name: &str) -> BasicBlock<'ctx>
    #[inline]
    pub fn insert_basic_block_after<'ctx>(&'ctx self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx>
    #[inline]
    pub fn prepend_basic_block<'ctx>(&'ctx self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx>
    #[inline]
    pub fn metadata_node<'ctx>(&'ctx self, values: &[BasicMetadataValueEnum<'ctx>]) -> MetadataValue<'ctx>
    #[inline]
    pub fn metadata_string(&self, string: &str) -> MetadataValue
    #[inline]
    pub fn get_kind_id(&self, key: &str) -> u32
    #[inline]
    pub fn create_enum_attribute(&self, kind_id: u32, val: u64) -> Attribute
    #[inline]
    pub fn create_string_attribute(&self, key: &str, val: &str) -> Attribute
    #[inline]
    #[llvm_versions(12..)]
    pub fn create_type_attribute(&self, kind_id: u32, type_ref: AnyTypeEnum) -> Attribute
    #[inline]
    pub fn const_string(&self, string: &[u8], null_terminated: bool) -> ArrayValue
    #[inline]
    pub(crate) fn set_diagnostic_handler(
}

impl Drop for Context
pub struct ContextRef<'ctx> {
    pub(crate) context: ContextImpl,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> ContextRef<'ctx> {
    pub fn raw(&self) -> LLVMContextRef
    pub unsafe fn new(context: LLVMContextRef) -> Self
    #[inline]
    pub fn create_builder(&self) -> Builder<'ctx>
    #[inline]
    pub fn create_module(&self, name: &str) -> Module<'ctx>
    #[inline]
    pub fn create_module_from_ir(&self, memory_buffer: MemoryBuffer) -> Result<Module<'ctx>, LLVMString>
    #[inline]
    pub fn create_inline_asm(
    #[inline]
    pub fn void_type(&self) -> VoidType<'ctx>
    #[inline]
    pub fn bool_type(&self) -> IntType<'ctx>
    #[inline]
    pub fn i8_type(&self) -> IntType<'ctx>
    #[inline]
    pub fn i16_type(&self) -> IntType<'ctx>
    #[inline]
    pub fn i32_type(&self) -> IntType<'ctx>
    #[inline]
    pub fn i64_type(&self) -> IntType<'ctx>
    #[inline]
    pub fn i128_type(&self) -> IntType<'ctx>
    #[inline]
    pub fn custom_width_int_type(&self, bits: u32) -> IntType<'ctx>
    #[inline]
    #[llvm_versions(6..)]
    pub fn metadata_type(&self) -> MetadataType<'ctx>
    #[inline]
    pub fn ptr_sized_int_type(&self, target_data: &TargetData, address_space: Option<AddressSpace>) -> IntType<'ctx>
    #[inline]
    pub fn f16_type(&self) -> FloatType<'ctx>
    #[inline]
    pub fn f32_type(&self) -> FloatType<'ctx>
    #[inline]
    pub fn f64_type(&self) -> FloatType<'ctx>
    #[inline]
    pub fn x86_f80_type(&self) -> FloatType<'ctx>
    #[inline]
    pub fn f128_type(&self) -> FloatType<'ctx>
    #[inline]
    pub fn ppc_f128_type(&self) -> FloatType<'ctx>
    #[llvm_versions(15..)]
    #[inline]
    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType<'ctx>
    #[inline]
    pub fn struct_type(&self, field_types: &[BasicTypeEnum<'ctx>], packed: bool) -> StructType<'ctx>
    #[inline]
    pub fn opaque_struct_type(&self, name: &str) -> StructType<'ctx>
    #[inline]
    #[llvm_versions(12..)]
    pub fn get_struct_type(&self, name: &str) -> Option<StructType<'ctx>>
    #[inline]
    pub fn const_struct(&self, values: &[BasicValueEnum<'ctx>], packed: bool) -> StructValue<'ctx>
    #[inline]
    pub fn append_basic_block(&self, function: FunctionValue<'ctx>, name: &str) -> BasicBlock<'ctx>
    #[inline]
    pub fn insert_basic_block_after(&self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx>
    #[inline]
    pub fn prepend_basic_block(&self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx>
    #[inline]
    pub fn metadata_node(&self, values: &[BasicMetadataValueEnum<'ctx>]) -> MetadataValue<'ctx>
    #[inline]
    pub fn metadata_string(&self, string: &str) -> MetadataValue<'ctx>
    #[inline]
    pub fn get_kind_id(&self, key: &str) -> u32
    #[inline]
    pub fn create_enum_attribute(&self, kind_id: u32, val: u64) -> Attribute
    #[inline]
    pub fn create_string_attribute(&self, key: &str, val: &str) -> Attribute
    #[inline]
    #[llvm_versions(12..)]
    pub fn create_type_attribute(&self, kind_id: u32, type_ref: AnyTypeEnum) -> Attribute
    #[inline]
    pub fn const_string(&self, string: &[u8], null_terminated: bool) -> ArrayValue<'ctx>
    #[inline]
    pub(crate) fn set_diagnostic_handler(
}

pub unsafe trait AsContextRef<'ctx> {
    fn as_ctx_ref(&self) -> LLVMContextRef;
}

unsafe impl<'ctx> AsContextRef<'ctx> for &'ctx Context
unsafe impl<'ctx> AsContextRef<'ctx> for ContextRef<'ctx>

```
`src/data_layout.rs`
```rust
pub struct DataLayout {
    pub(crate) data_layout: LLVMStringOrRaw,
}

impl DataLayout {
    pub(crate) unsafe fn new_owned(data_layout: *const ::libc::c_char) -> DataLayout
    pub(crate) unsafe fn new_borrowed(data_layout: *const ::libc::c_char) -> DataLayout
    pub fn as_str(&self) -> &CStr
    pub fn as_ptr(&self) -> *const ::libc::c_char
}

impl fmt::Debug for DataLayout

```
`src/debug_info.rs`
```rust
pub fn debug_metadata_version() -> libc::c_uint
#[derive(Debug, PartialEq, Eq)]
pub struct DebugInfoBuilder<'ctx> {
    pub(crate) builder: LLVMDIBuilderRef,
    _marker: PhantomData<&'ctx Context>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DIScope<'ctx> {
    metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DIScope<'ctx> {
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

pub trait AsDIScope<'ctx> {
    #[allow(clippy::wrong_self_convention)]
    fn as_debug_info_scope(self) -> DIScope<'ctx>;
}

impl<'ctx> DebugInfoBuilder<'ctx> {
    pub(crate) fn new(
    pub fn as_mut_ptr(&self) -> LLVMDIBuilderRef
    fn create_compile_unit(
    pub fn create_function(
    pub fn create_lexical_block(
    pub fn create_file(&self, filename: &str, directory: &str) -> DIFile<'ctx>
    pub fn create_debug_location(
    #[llvm_versions(7..)]
    pub fn create_basic_type(
    #[llvm_versions(8..)]
    pub fn create_typedef(
    pub fn create_union_type(
    pub fn create_member_type(
    pub fn create_struct_type(
    pub fn create_subroutine_type(
    pub fn create_pointer_type(
    pub fn create_reference_type(&self, pointee: DIType<'ctx>, tag: u32) -> DIDerivedType<'ctx>
    pub fn create_array_type(
    #[llvm_versions(8..)]
    pub fn create_global_variable_expression(
    #[llvm_versions(8..)]
    pub fn create_constant_expression(&self, value: i64) -> DIExpression<'ctx>
    pub fn create_parameter_variable(
    pub fn create_auto_variable(
    pub fn create_namespace(&self, scope: DIScope<'ctx>, name: &str, export_symbols: bool) -> DINamespace<'ctx>
    pub fn insert_declare_before_instruction(
    pub fn insert_declare_at_end(
    pub fn create_expression(&self, mut address_operations: Vec<i64>) -> DIExpression<'ctx>
    pub fn insert_dbg_value_before(
    pub unsafe fn create_placeholder_derived_type(&self, context: impl AsContextRef<'ctx>) -> DIDerivedType<'ctx>
    pub unsafe fn replace_placeholder_derived_type(
    pub fn finalize(&self)
}

impl<'ctx> Drop for DebugInfoBuilder<'ctx>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIFile<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> AsDIScope<'ctx> for DIFile<'ctx>
impl<'ctx> DIFile<'ctx> {
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DICompileUnit<'ctx> {
    file: DIFile<'ctx>,
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DICompileUnit<'ctx> {
    pub fn get_file(&self) -> DIFile<'ctx>
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

impl<'ctx> AsDIScope<'ctx> for DICompileUnit<'ctx>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DINamespace<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DINamespace<'ctx> {
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

impl<'ctx> AsDIScope<'ctx> for DINamespace<'ctx>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DISubprogram<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    pub(crate) _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> AsDIScope<'ctx> for DISubprogram<'ctx>
impl<'ctx> DISubprogram<'ctx> {
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DIType<'ctx> {
    pub fn get_size_in_bits(&self) -> u64
    pub fn get_align_in_bits(&self) -> u32
    pub fn get_offset_in_bits(&self) -> u64
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

impl<'ctx> AsDIScope<'ctx> for DIType<'ctx>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIDerivedType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DIDerivedType<'ctx> {
    pub fn as_type(&self) -> DIType<'ctx>
}

impl<'ctx> DIDerivedType<'ctx> {
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

impl<'ctx> AsDIScope<'ctx> for DIDerivedType<'ctx>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIBasicType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DIBasicType<'ctx> {
    pub fn as_type(&self) -> DIType<'ctx>
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

impl<'ctx> AsDIScope<'ctx> for DIBasicType<'ctx>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DICompositeType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DICompositeType<'ctx> {
    pub fn as_type(&self) -> DIType<'ctx>
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

impl<'ctx> AsDIScope<'ctx> for DICompositeType<'ctx>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DISubroutineType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DILexicalBlock<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> AsDIScope<'ctx> for DILexicalBlock<'ctx>
impl<'ctx> DILexicalBlock<'ctx> {
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DILocation<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    pub(crate) _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DILocation<'ctx> {
    pub fn get_line(&self) -> u32
    pub fn get_column(&self) -> u32
    pub fn get_scope(&self) -> DIScope<'ctx>
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DILocalVariable<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DILocalVariable<'ctx> {
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIGlobalVariableExpression<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DIGlobalVariableExpression<'ctx> {
    pub fn as_metadata_value(&self, context: impl AsContextRef<'ctx>) -> MetadataValue<'ctx>
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIExpression<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DIExpression<'ctx> {
    pub fn as_mut_ptr(&self) -> LLVMMetadataRef
}

pub use flags::*;
mod flags {
    pub use llvm_sys::debuginfo::LLVMDIFlags as DIFlags;
    use llvm_sys::debuginfo::{LLVMDWARFEmissionKind, LLVMDWARFSourceLanguage};

    pub trait DIFlagsConstants {
        const ZERO: Self;
        const PRIVATE: Self;
        const PROTECTED: Self;
        const PUBLIC: Self;
        const FWD_DECL: Self;
        const APPLE_BLOCK: Self;
        const VIRTUAL: Self;
        const ARTIFICIAL: Self;
        const EXPLICIT: Self;
        const PROTOTYPED: Self;
        const OBJC_CLASS_COMPLETE: Self;
        const OBJECT_POINTER: Self;
        const VECTOR: Self;
        const STATIC_MEMBER: Self;
        const LVALUE_REFERENCE: Self;
        const RVALUE_REFERENCE: Self;
        const RESERVED: Self;
        const SINGLE_INHERITANCE: Self;
        const MULTIPLE_INHERITANCE: Self;
        const VIRTUAL_INHERITANCE: Self;
        const INTRODUCED_VIRTUAL: Self;
        const BIT_FIELD: Self;
        const NO_RETURN: Self;
        const TYPE_PASS_BY_VALUE: Self;
        const TYPE_PASS_BY_REFERENCE: Self;
        const THUNK: Self;
        const INDIRECT_VIRTUAL_BASE: Self;
    }
    impl DIFlagsConstants for DIFlags {
        const ZERO: DIFlags = llvm_sys::debuginfo::LLVMDIFlagZero;
        const PRIVATE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagPrivate;
        const PROTECTED: DIFlags = llvm_sys::debuginfo::LLVMDIFlagProtected;
        const PUBLIC: DIFlags = llvm_sys::debuginfo::LLVMDIFlagPublic;
        const FWD_DECL: DIFlags = llvm_sys::debuginfo::LLVMDIFlagFwdDecl;
        const APPLE_BLOCK: DIFlags = llvm_sys::debuginfo::LLVMDIFlagAppleBlock;
        const VIRTUAL: DIFlags = llvm_sys::debuginfo::LLVMDIFlagVirtual;
        const ARTIFICIAL: DIFlags = llvm_sys::debuginfo::LLVMDIFlagArtificial;
        const EXPLICIT: DIFlags = llvm_sys::debuginfo::LLVMDIFlagExplicit;
        const PROTOTYPED: DIFlags = llvm_sys::debuginfo::LLVMDIFlagPrototyped;
        const OBJC_CLASS_COMPLETE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagObjcClassComplete;
        const OBJECT_POINTER: DIFlags = llvm_sys::debuginfo::LLVMDIFlagObjectPointer;
        const VECTOR: DIFlags = llvm_sys::debuginfo::LLVMDIFlagVector;
        const STATIC_MEMBER: DIFlags = llvm_sys::debuginfo::LLVMDIFlagStaticMember;
        const LVALUE_REFERENCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagLValueReference;
        const RVALUE_REFERENCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagRValueReference;
        const RESERVED: DIFlags = llvm_sys::debuginfo::LLVMDIFlagReserved;
        const SINGLE_INHERITANCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagSingleInheritance;
        const MULTIPLE_INHERITANCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagMultipleInheritance;
        const VIRTUAL_INHERITANCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagVirtualInheritance;
        const INTRODUCED_VIRTUAL: DIFlags = llvm_sys::debuginfo::LLVMDIFlagIntroducedVirtual;
        const BIT_FIELD: DIFlags = llvm_sys::debuginfo::LLVMDIFlagBitField;
        const NO_RETURN: DIFlags = llvm_sys::debuginfo::LLVMDIFlagNoReturn;
        const TYPE_PASS_BY_VALUE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagTypePassByValue;
        const TYPE_PASS_BY_REFERENCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagTypePassByReference;
        const THUNK: DIFlags = llvm_sys::debuginfo::LLVMDIFlagThunk;
        const INDIRECT_VIRTUAL_BASE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagIndirectVirtualBase;
    }

    #[llvm_enum(LLVMDWARFEmissionKind)]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum DWARFEmissionKind {
        #[llvm_variant(LLVMDWARFEmissionKindNone)]
        None,
        #[llvm_variant(LLVMDWARFEmissionKindFull)]
        Full,
        #[llvm_variant(LLVMDWARFEmissionKindLineTablesOnly)]
        LineTablesOnly,
    }

    #[llvm_enum(LLVMDWARFSourceLanguage)]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum DWARFSourceLanguage {
        #[llvm_variant(LLVMDWARFSourceLanguageC89)]
        C89,
        #[llvm_variant(LLVMDWARFSourceLanguageC)]
        C,
        #[llvm_variant(LLVMDWARFSourceLanguageAda83)]
        Ada83,
        #[llvm_variant(LLVMDWARFSourceLanguageC_plus_plus)]
        CPlusPlus,
        #[llvm_variant(LLVMDWARFSourceLanguageCobol74)]
        Cobol74,
        #[llvm_variant(LLVMDWARFSourceLanguageCobol85)]
        Cobol85,
        #[llvm_variant(LLVMDWARFSourceLanguageFortran77)]
        Fortran77,
        #[llvm_variant(LLVMDWARFSourceLanguageFortran90)]
        Fortran90,
        #[llvm_variant(LLVMDWARFSourceLanguagePascal83)]
        Pascal83,
        #[llvm_variant(LLVMDWARFSourceLanguageModula2)]
        Modula2,
        #[llvm_variant(LLVMDWARFSourceLanguageJava)]
        Java,
        #[llvm_variant(LLVMDWARFSourceLanguageC99)]
        C99,
        #[llvm_variant(LLVMDWARFSourceLanguageAda95)]
        Ada95,
        #[llvm_variant(LLVMDWARFSourceLanguageFortran95)]
        Fortran95,
        #[llvm_variant(LLVMDWARFSourceLanguagePLI)]
        PLI,
        #[llvm_variant(LLVMDWARFSourceLanguageObjC)]
        ObjC,
        #[llvm_variant(LLVMDWARFSourceLanguageObjC_plus_plus)]
        ObjCPlusPlus,
        #[llvm_variant(LLVMDWARFSourceLanguageUPC)]
        UPC,
        #[llvm_variant(LLVMDWARFSourceLanguageD)]
        D,
        #[llvm_variant(LLVMDWARFSourceLanguagePython)]
        Python,
        #[llvm_variant(LLVMDWARFSourceLanguageOpenCL)]
        OpenCL,
        #[llvm_variant(LLVMDWARFSourceLanguageGo)]
        Go,
        #[llvm_variant(LLVMDWARFSourceLanguageModula3)]
        Modula3,
        #[llvm_variant(LLVMDWARFSourceLanguageHaskell)]
        Haskell,
        #[llvm_variant(LLVMDWARFSourceLanguageC_plus_plus_03)]
        CPlusPlus03,
        #[llvm_variant(LLVMDWARFSourceLanguageC_plus_plus_11)]
        CPlusPlus11,
        #[llvm_variant(LLVMDWARFSourceLanguageOCaml)]
        OCaml,
        #[llvm_variant(LLVMDWARFSourceLanguageRust)]
        Rust,
        #[llvm_variant(LLVMDWARFSourceLanguageC11)]
        C11,
        #[llvm_variant(LLVMDWARFSourceLanguageSwift)]
        Swift,
        #[llvm_variant(LLVMDWARFSourceLanguageJulia)]
        Julia,
        #[llvm_variant(LLVMDWARFSourceLanguageDylan)]
        Dylan,
        #[llvm_variant(LLVMDWARFSourceLanguageC_plus_plus_14)]
        CPlusPlus14,
        #[llvm_variant(LLVMDWARFSourceLanguageFortran03)]
        Fortran03,
        #[llvm_variant(LLVMDWARFSourceLanguageFortran08)]
        Fortran08,
        #[llvm_variant(LLVMDWARFSourceLanguageRenderScript)]
        RenderScript,
        #[llvm_variant(LLVMDWARFSourceLanguageBLISS)]
        BLISS,
        #[llvm_variant(LLVMDWARFSourceLanguageMips_Assembler)]
        MipsAssembler,
        #[llvm_variant(LLVMDWARFSourceLanguageGOOGLE_RenderScript)]
        GOOGLERenderScript,
        #[llvm_variant(LLVMDWARFSourceLanguageBORLAND_Delphi)]
        BORLANDDelphi,
        #[llvm_versions(16..)]
        #[llvm_variant(LLVMDWARFSourceLanguageKotlin)]
        Kotlin,
        #[llvm_versions(16..)]
        #[llvm_variant(LLVMDWARFSourceLanguageZig)]
        Zig,
        #[llvm_versions(16..)]
        #[llvm_variant(LLVMDWARFSourceLanguageCrystal)]
        Crystal,
        #[llvm_versions(16..)]
        #[llvm_variant(LLVMDWARFSourceLanguageC_plus_plus_17)]
        CPlusPlus17,
        #[llvm_versions(16..)]
        #[llvm_variant(LLVMDWARFSourceLanguageC_plus_plus_20)]
        CPlusPlus20,
        #[llvm_versions(16..)]
        #[llvm_variant(LLVMDWARFSourceLanguageC17)]
        C17,
        #[llvm_versions(16..)]
        #[llvm_variant(LLVMDWARFSourceLanguageFortran18)]
        Fortran18,
        #[llvm_versions(16..)]
        #[llvm_variant(LLVMDWARFSourceLanguageAda2005)]
        Ada2005,
        #[llvm_versions(16..)]
        #[llvm_variant(LLVMDWARFSourceLanguageAda2012)]
        Ada2012,
        #[llvm_versions(17..)]
        #[llvm_variant(LLVMDWARFSourceLanguageMojo)]
        Mojo,
    }
}
    
    ```
    `src/execution_engine.rs`
    ```rust
    pub enum FunctionLookupError {
        JITNotEnabled,
        FunctionNotFound,
    }
    
    impl FunctionLookupError {
        fn as_str(&self) -> &str
    }
    
    impl Display for FunctionLookupError
    pub enum RemoveModuleError {
        ModuleNotOwned,
        IncorrectModuleOwner,
        LLVMError(LLVMString),
    }
    
    impl Error for RemoveModuleError
    impl RemoveModuleError {
        fn as_str(&self) -> &str
    }
    
    impl Display for RemoveModuleError
    pub struct ExecutionEngine<'ctx> {
        execution_engine: Option<ExecEngineInner<'ctx>>,
        target_data: Option<TargetData>,
        jit_mode: bool,
    }
    
    impl<'ctx> ExecutionEngine<'ctx> {
        pub unsafe fn new(execution_engine: Rc<LLVMExecutionEngineRef>, jit_mode: bool) -> Self
        pub fn as_mut_ptr(&self) -> LLVMExecutionEngineRef
        pub(crate) fn execution_engine_rc(&self) -> &Rc<LLVMExecutionEngineRef>
        #[inline]
        pub(crate) fn execution_engine_inner(&self) -> LLVMExecutionEngineRef
        pub fn link_in_mc_jit()
        pub fn link_in_interpreter()
        pub fn add_global_mapping(&self, value: &dyn AnyValue<'ctx>, addr: usize)
        pub fn add_module(&self, module: &Module<'ctx>) -> Result<(), ()>
        pub fn remove_module(&self, module: &Module<'ctx>) -> Result<(), RemoveModuleError>
        pub unsafe fn get_function<F>(&self, fn_name: &str) -> Result<JitFunction<'ctx, F>, FunctionLookupError>
        pub fn get_function_address(&self, fn_name: &str) -> Result<usize, FunctionLookupError>
        pub fn get_target_data(&self) -> &TargetData
        pub fn get_function_value(&self, fn_name: &str) -> Result<FunctionValue<'ctx>, FunctionLookupError>
        pub unsafe fn run_function(
        pub unsafe fn run_function_as_main(&self, function: FunctionValue<'ctx>, args: &[&str]) -> c_int
        pub fn free_fn_machine_code(&self, function: FunctionValue<'ctx>)
        pub fn run_static_constructors(&self)
        pub fn run_static_destructors(&self)
    }
    
    impl Drop for ExecutionEngine<'_>
    impl Clone for ExecutionEngine<'_>
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ExecEngineInner<'ctx>(Rc<LLVMExecutionEngineRef>, PhantomData<&'ctx Context>);
    
    impl Drop for ExecEngineInner<'_>
    impl Deref for ExecEngineInner<'_> {
        type Target = LLVMExecutionEngineRef;
    
        fn deref(&self) -> &Self::Target
    }
    
    #[derive(Clone)]
    pub struct JitFunction<'ctx, F> {
        _execution_engine: ExecEngineInner<'ctx>,
        inner: F,
    }
    
    impl<F> Debug for JitFunction<'_, F>
    pub trait UnsafeFunctionPointer: private::SealedUnsafeFunctionPointer {}
    
    mod private {
        pub trait SealedUnsafeFunctionPointer: Copy {}
    }
    
    impl<F: private::SealedUnsafeFunctionPointer> UnsafeFunctionPointer for F {}
    
    #[cfg(feature = "experimental")]
    pub mod experimental {
        #[derive(Debug)]
        pub struct MangledSymbol(*mut libc::c_char);
    
        impl Deref for MangledSymbol
        #[derive(Debug)]
        pub struct LLVMError(LLVMErrorRef);
    
        impl LLVMError {
            pub fn get_type_id(&self) -> LLVMErrorTypeId
        }
    
        impl Deref for LLVMError
        impl Drop for LLVMError
        #[derive(Debug)]
        pub struct Orc(LLVMOrcJITStackRef);
    
        impl Orc {
            pub fn create(target_machine: TargetMachine) -> Self
            pub fn add_compiled_ir<'ctx>(&self, module: &Module<'ctx>, lazily: bool) -> Result<(), ()>
            pub fn get_error(&self) -> &CStr
            pub fn get_mangled_symbol(&self, symbol: &str) -> MangledSymbol
        }
    
        impl Drop for Orc
        #[test]
        fn test_mangled_str()
    }
    
    ```
    `src/intrinsics.rs`
    ```rust
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct Intrinsic {
        id: u32,
    }
    
    #[llvm_versions(9..)]
    impl Intrinsic {
        pub(crate) unsafe fn new(id: u32) -> Self
        pub fn find(name: &str) -> Option<Self>
        pub fn is_overloaded(&self) -> bool
        pub fn get_declaration<'ctx>(
    
    ```
    `src/lib.rs`
    ```rust
    #[macro_use]
    extern crate inkwell_internals;
    
    #[macro_use]
    pub mod support;
    #[deny(missing_docs)]
    pub mod attributes;
    #[deny(missing_docs)]
    pub mod basic_block;
    pub mod builder;
    #[deny(missing_docs)]
    #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
    pub mod comdat;
    #[deny(missing_docs)]
    pub mod context;
    pub mod data_layout;
    #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
    pub mod debug_info;
    pub mod execution_engine;
    pub mod intrinsics;
    pub mod memory_buffer;
    pub mod memory_manager;
    #[deny(missing_docs)]
    pub mod module;
    pub mod object_file;
    pub mod passes;
    pub mod targets;
    pub mod types;
    pub mod values;
    
    #[cfg(feature = "llvm10-0")]
    pub extern crate llvm_sys_100 as llvm_sys;
    #[cfg(feature = "llvm11-0")]
    pub extern crate llvm_sys_110 as llvm_sys;
    #[cfg(feature = "llvm12-0")]
    pub extern crate llvm_sys_120 as llvm_sys;
    #[cfg(feature = "llvm13-0")]
    pub extern crate llvm_sys_130 as llvm_sys;
    #[cfg(feature = "llvm14-0")]
    pub extern crate llvm_sys_140 as llvm_sys;
    #[cfg(feature = "llvm15-0")]
    pub extern crate llvm_sys_150 as llvm_sys;
    #[cfg(feature = "llvm16-0")]
    pub extern crate llvm_sys_160 as llvm_sys;
    #[cfg(feature = "llvm17-0")]
    pub extern crate llvm_sys_170 as llvm_sys;
    #[cfg(feature = "llvm18-0")]
    pub extern crate llvm_sys_180 as llvm_sys;
    #[cfg(feature = "llvm4-0")]
    pub extern crate llvm_sys_40 as llvm_sys;
    #[cfg(feature = "llvm5-0")]
    pub extern crate llvm_sys_50 as llvm_sys;
    #[cfg(feature = "llvm6-0")]
    pub extern crate llvm_sys_60 as llvm_sys;
    #[cfg(feature = "llvm7-0")]
    pub extern crate llvm_sys_70 as llvm_sys;
    #[cfg(feature = "llvm8-0")]
    pub extern crate llvm_sys_80 as llvm_sys;
    #[cfg(feature = "llvm9-0")]
    pub extern crate llvm_sys_90 as llvm_sys;
    
    #[llvm_versions(7..)]
    use llvm_sys::LLVMInlineAsmDialect;
    
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};
    use std::convert::TryFrom;
    
    macro_rules! assert_unique_features {
        () => {};
        ($first:tt $(,$rest:tt)*) => {
            $(
                #[cfg(all(feature = $first, feature = $rest))]
                compile_error!(concat!("features \"", $first, "\" and \"", $rest, "\" cannot be used together"));
            )*
            assert_unique_features!($($rest),*);
        }
    }
    
    macro_rules! assert_used_features {
        ($($all:tt),*) => {
            #[cfg(not(any($(feature = $all),*)))]
            compile_error!(concat!("One of the LLVM feature flags must be provided: ", $($all, " "),*));
        }
    }
    
    macro_rules! assert_unique_used_features {
        ($($all:tt),*) => {
            assert_unique_features!($($all),*);
            assert_used_features!($($all),*);
        }
    }
    
    assert_unique_used_features! {
        "llvm4-0",
        "llvm5-0",
        "llvm6-0",
        "llvm7-0",
        "llvm8-0",
        "llvm9-0",
        "llvm10-0",
        "llvm11-0",
        "llvm12-0",
        "llvm13-0",
        "llvm14-0",
        "llvm15-0",
        "llvm16-0",
        "llvm17-0",
        "llvm18-0"
    }
    
    pub struct AddressSpace(u32);
    
    impl From<u16> for AddressSpace
    impl TryFrom<u32> for AddressSpace
    #[llvm_enum(LLVMIntPredicate)]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum IntPredicate {
        #[llvm_variant(LLVMIntEQ)]
        EQ,
        #[llvm_variant(LLVMIntNE)]
        NE,
        #[llvm_variant(LLVMIntUGT)]
        UGT,
        #[llvm_variant(LLVMIntUGE)]
        UGE,
        #[llvm_variant(LLVMIntULT)]
        ULT,
        #[llvm_variant(LLVMIntULE)]
        ULE,
        #[llvm_variant(LLVMIntSGT)]
        SGT,
        #[llvm_variant(LLVMIntSGE)]
        SGE,
        #[llvm_variant(LLVMIntSLT)]
        SLT,
        #[llvm_variant(LLVMIntSLE)]
        SLE,
    }
    
    #[llvm_enum(LLVMRealPredicate)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum FloatPredicate {
        #[llvm_variant(LLVMRealOEQ)]
        OEQ,
        #[llvm_variant(LLVMRealOGE)]
        OGE,
        #[llvm_variant(LLVMRealOGT)]
        OGT,
        #[llvm_variant(LLVMRealOLE)]
        OLE,
        #[llvm_variant(LLVMRealOLT)]
        OLT,
        #[llvm_variant(LLVMRealONE)]
        ONE,
        #[llvm_variant(LLVMRealORD)]
        ORD,
        #[llvm_variant(LLVMRealPredicateFalse)]
        PredicateFalse,
        #[llvm_variant(LLVMRealPredicateTrue)]
        PredicateTrue,
        #[llvm_variant(LLVMRealUEQ)]
        UEQ,
        #[llvm_variant(LLVMRealUGE)]
        UGE,
        #[llvm_variant(LLVMRealUGT)]
        UGT,
        #[llvm_variant(LLVMRealULE)]
        ULE,
        #[llvm_variant(LLVMRealULT)]
        ULT,
        #[llvm_variant(LLVMRealUNE)]
        UNE,
        #[llvm_variant(LLVMRealUNO)]
        UNO,
    }
    
    #[llvm_enum(LLVMAtomicOrdering)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum AtomicOrdering {
        #[llvm_variant(LLVMAtomicOrderingNotAtomic)]
        NotAtomic,
        #[llvm_variant(LLVMAtomicOrderingUnordered)]
        Unordered,
        #[llvm_variant(LLVMAtomicOrderingMonotonic)]
        Monotonic,
        #[llvm_variant(LLVMAtomicOrderingAcquire)]
        Acquire,
        #[llvm_variant(LLVMAtomicOrderingRelease)]
        Release,
        #[llvm_variant(LLVMAtomicOrderingAcquireRelease)]
        AcquireRelease,
        #[llvm_variant(LLVMAtomicOrderingSequentiallyConsistent)]
        SequentiallyConsistent,
    }
    
    #[llvm_enum(LLVMAtomicRMWBinOp)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum AtomicRMWBinOp {
        #[llvm_variant(LLVMAtomicRMWBinOpXchg)]
        Xchg,
        #[llvm_variant(LLVMAtomicRMWBinOpAdd)]
        Add,
        #[llvm_variant(LLVMAtomicRMWBinOpSub)]
        Sub,
        #[llvm_variant(LLVMAtomicRMWBinOpAnd)]
        And,
        #[llvm_variant(LLVMAtomicRMWBinOpNand)]
        Nand,
        #[llvm_variant(LLVMAtomicRMWBinOpOr)]
        Or,
        #[llvm_variant(LLVMAtomicRMWBinOpXor)]
        Xor,
        #[llvm_variant(LLVMAtomicRMWBinOpMax)]
        Max,
        #[llvm_variant(LLVMAtomicRMWBinOpMin)]
        Min,
        #[llvm_variant(LLVMAtomicRMWBinOpUMax)]
        UMax,
        #[llvm_variant(LLVMAtomicRMWBinOpUMin)]
        UMin,
        #[llvm_versions(10..)]
        #[llvm_variant(LLVMAtomicRMWBinOpFAdd)]
        FAdd,
        #[llvm_versions(10..)]
        #[llvm_variant(LLVMAtomicRMWBinOpFSub)]
        FSub,
        #[llvm_versions(15..)]
        #[llvm_variant(LLVMAtomicRMWBinOpFMax)]
        FMax,
        #[llvm_versions(15..)]
        #[llvm_variant(LLVMAtomicRMWBinOpFMin)]
        FMin,
    }
    
    #[repr(u32)]
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum OptimizationLevel {
        None = 0,
        Less = 1,
        Default = 2,
        Aggressive = 3,
    }
    
    impl Default for OptimizationLevel {
        fn default() -> Self {
            OptimizationLevel::Default
        }
    }
    
    impl From<OptimizationLevel> for LLVMCodeGenOptLevel {
        fn from(value: OptimizationLevel) -> Self {
            match value {
                OptimizationLevel::None => LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
                OptimizationLevel::Less => LLVMCodeGenOptLevel::LLVMCodeGenLevelLess,
                OptimizationLevel::Default => LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
                OptimizationLevel::Aggressive => LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
            }
        }
    }
    
    #[llvm_enum(LLVMVisibility)]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum GlobalVisibility {
        #[llvm_variant(LLVMDefaultVisibility)]
        Default,
        #[llvm_variant(LLVMHiddenVisibility)]
        Hidden,
        #[llvm_variant(LLVMProtectedVisibility)]
        Protected,
    }
    
    impl Default for GlobalVisibility {
        fn default() -> Self {
            GlobalVisibility::Default
        }
    }
    
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum ThreadLocalMode {
        GeneralDynamicTLSModel,
        LocalDynamicTLSModel,
        InitialExecTLSModel,
        LocalExecTLSModel,
    }
    
    impl ThreadLocalMode {
        pub(crate) fn new(thread_local_mode: LLVMThreadLocalMode) -> Option<Self> {
            match thread_local_mode {
                LLVMThreadLocalMode::LLVMGeneralDynamicTLSModel => Some(ThreadLocalMode::GeneralDynamicTLSModel),
                LLVMThreadLocalMode::LLVMLocalDynamicTLSModel => Some(ThreadLocalMode::LocalDynamicTLSModel),
                LLVMThreadLocalMode::LLVMInitialExecTLSModel => Some(ThreadLocalMode::InitialExecTLSModel),
                LLVMThreadLocalMode::LLVMLocalExecTLSModel => Some(ThreadLocalMode::LocalExecTLSModel),
                LLVMThreadLocalMode::LLVMNotThreadLocal => None,
            }
        }
    
        pub(crate) fn as_llvm_mode(self) -> LLVMThreadLocalMode {
            match self {
                ThreadLocalMode::GeneralDynamicTLSModel => LLVMThreadLocalMode::LLVMGeneralDynamicTLSModel,
                ThreadLocalMode::LocalDynamicTLSModel => LLVMThreadLocalMode::LLVMLocalDynamicTLSModel,
                ThreadLocalMode::InitialExecTLSModel => LLVMThreadLocalMode::LLVMInitialExecTLSModel,
                ThreadLocalMode::LocalExecTLSModel => LLVMThreadLocalMode::LLVMLocalExecTLSModel,
            }
        }
    }
    
    #[llvm_enum(LLVMDLLStorageClass)]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum DLLStorageClass {
        #[llvm_variant(LLVMDefaultStorageClass)]
        Default,
        #[llvm_variant(LLVMDLLImportStorageClass)]
        Import,
        #[llvm_variant(LLVMDLLExportStorageClass)]
        Export,
    }
    
    impl Default for DLLStorageClass {
        fn default() -> Self {
            DLLStorageClass::Default
        }
    }
    
    #[llvm_versions(7..)]
    #[llvm_enum(LLVMInlineAsmDialect)]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum InlineAsmDialect {
        #[llvm_variant(LLVMInlineAsmDialectATT)]
        ATT,
        #[llvm_variant(LLVMInlineAsmDialectIntel)]
        Intel,
    }
    
    ```
    `src/memory_buffer.rs`
    ```rust
    #[derive(Debug)]
    pub struct MemoryBuffer {
        pub(crate) memory_buffer: LLVMMemoryBufferRef,
    }
    
    impl MemoryBuffer {
        pub unsafe fn new(memory_buffer: LLVMMemoryBufferRef) -> Self
        pub fn as_mut_ptr(&self) -> LLVMMemoryBufferRef
        pub fn create_from_file(path: &Path) -> Result<Self, LLVMString>
        pub fn create_from_stdin() -> Result<Self, LLVMString>
        pub fn create_from_memory_range(input: &[u8], name: &str) -> Self
        pub fn create_from_memory_range_copy(input: &[u8], name: &str) -> Self
        pub fn as_slice(&self) -> &[u8]
        pub fn get_size(&self) -> usize
        pub fn create_object_file(self) -> Result<ObjectFile, ()>
    }
    
    impl Drop for MemoryBuffer
    
    ```
    `src/memory_manager.rs`
    ```rust
    pub trait McjitMemoryManager: std::fmt::Debug {
        fn allocate_code_section(
        fn allocate_data_section(
        fn finalize_memory(&mut self) -> Result<(), String>;
        fn destroy(&mut self);
    }
    
    #[derive(Debug)]
    pub struct MemoryManagerAdapter {
        pub memory_manager: Box<dyn McjitMemoryManager>,
    }
    
    pub(crate) extern "C" fn allocate_code_section_adapter(
    pub(crate) extern "C" fn allocate_data_section_adapter(
    pub(crate) extern "C" fn finalize_memory_adapter(
    pub(crate) extern "C" fn destroy_adapter(opaque: *mut libc::c_void)
    unsafe fn c_str_to_str<'a>(ptr: *const libc::c_char) -> &'a str
    
    ```
    `src/module.rs`
    ```rust
    #[llvm_enum(LLVMLinkage)]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum Linkage {
        #[llvm_variant(LLVMAppendingLinkage)]
        Appending,
        #[llvm_variant(LLVMAvailableExternallyLinkage)]
        AvailableExternally,
        #[llvm_variant(LLVMCommonLinkage)]
        Common,
        #[llvm_variant(LLVMDLLExportLinkage)]
        DLLExport,
        #[llvm_variant(LLVMDLLImportLinkage)]
        DLLImport,
        #[llvm_variant(LLVMExternalLinkage)]
        External,
        #[llvm_variant(LLVMExternalWeakLinkage)]
        ExternalWeak,
        #[llvm_variant(LLVMGhostLinkage)]
        Ghost,
        #[llvm_variant(LLVMInternalLinkage)]
        Internal,
        #[llvm_variant(LLVMLinkerPrivateLinkage)]
        LinkerPrivate,
        #[llvm_variant(LLVMLinkerPrivateWeakLinkage)]
        LinkerPrivateWeak,
        #[llvm_variant(LLVMLinkOnceAnyLinkage)]
        LinkOnceAny,
        #[llvm_variant(LLVMLinkOnceODRAutoHideLinkage)]
        LinkOnceODRAutoHide,
        #[llvm_variant(LLVMLinkOnceODRLinkage)]
        LinkOnceODR,
        #[llvm_variant(LLVMPrivateLinkage)]
        Private,
        #[llvm_variant(LLVMWeakAnyLinkage)]
        WeakAny,
        #[llvm_variant(LLVMWeakODRLinkage)]
        WeakODR,
    }
    
    #[derive(Debug, PartialEq, Eq)]
    pub struct Module<'ctx> {
        data_layout: RefCell<Option<DataLayout>>,
        pub(crate) module: Cell<LLVMModuleRef>,
        pub(crate) owned_by_ee: RefCell<Option<ExecutionEngine<'ctx>>>,
        _marker: PhantomData<&'ctx Context>,
    }
    
    impl<'ctx> Module<'ctx> {
        pub unsafe fn new(module: LLVMModuleRef) -> Self
        pub fn as_mut_ptr(&self) -> LLVMModuleRef
        pub fn add_function(&self, name: &str, ty: FunctionType<'ctx>, linkage: Option<Linkage>) -> FunctionValue<'ctx>
        pub fn get_context(&self) -> ContextRef<'ctx>
        pub fn get_first_function(&self) -> Option<FunctionValue<'ctx>>
        pub fn get_last_function(&self) -> Option<FunctionValue<'ctx>>
        pub fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>>
        pub fn get_functions(&self) -> FunctionIterator<'ctx>
        #[llvm_versions(..=11)]
        pub fn get_struct_type(&self, name: &str) -> Option<StructType<'ctx>>
        #[llvm_versions(12..)]
        pub fn get_struct_type(&self, name: &str) -> Option<StructType<'ctx>>
        pub fn set_triple(&self, triple: &TargetTriple)
        pub fn get_triple(&self) -> TargetTriple
        pub fn create_execution_engine(&self) -> Result<ExecutionEngine<'ctx>, LLVMString>
        pub fn create_interpreter_execution_engine(&self) -> Result<ExecutionEngine<'ctx>, LLVMString>
        pub fn create_jit_execution_engine(
        pub fn create_mcjit_execution_engine_with_memory_manager(
        pub fn add_global<T: BasicType<'ctx>>(
        pub fn write_bitcode_to_path(&self, path: impl AsRef<Path>) -> bool
        pub fn write_bitcode_to_file(&self, file: &File, should_close: bool, unbuffered: bool) -> bool
        pub fn write_bitcode_to_memory(&self) -> MemoryBuffer
        pub fn verify(&self) -> Result<(), LLVMString>
        fn get_borrowed_data_layout(module: LLVMModuleRef) -> DataLayout
        pub fn get_data_layout(&self) -> Ref<DataLayout>
        pub fn set_data_layout(&self, data_layout: &DataLayout)
        pub fn print_to_stderr(&self)
        pub fn print_to_string(&self) -> LLVMString
        pub fn print_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LLVMString>
        #[allow(clippy::inherent_to_string)]
        pub fn to_string(&self) -> String
        pub fn set_inline_assembly(&self, asm: &str)
        #[llvm_versions(7..)]
        pub fn add_global_metadata(&self, key: &str, metadata: &MetadataValue<'ctx>) -> Result<(), &'static str>
        pub fn get_global_metadata_size(&self, key: &str) -> u32
        pub fn get_global_metadata(&self, key: &str) -> Vec<MetadataValue<'ctx>>
        pub fn get_first_global(&self) -> Option<GlobalValue<'ctx>>
        pub fn get_last_global(&self) -> Option<GlobalValue<'ctx>>
        pub fn get_global(&self, name: &str) -> Option<GlobalValue<'ctx>>
        pub fn get_globals(&self) -> GlobalIterator<'ctx>
        pub fn parse_bitcode_from_buffer(
        pub fn parse_bitcode_from_path<P: AsRef<Path>>(
        pub fn get_name(&self) -> &CStr
        pub fn set_name(&self, name: &str)
        #[llvm_versions(7..)]
        pub fn get_source_file_name(&self) -> &CStr
        #[llvm_versions(7..)]
        pub fn set_source_file_name(&self, file_name: &str)
        pub fn link_in_module(&self, other: Self) -> Result<(), LLVMString>
        #[llvm_versions(7..)]
        pub fn get_or_insert_comdat(&self, name: &str) -> Comdat
        #[llvm_versions(7..)]
        pub fn get_flag(&self, key: &str) -> Option<MetadataValue<'ctx>>
        #[llvm_versions(7..)]
        pub fn add_metadata_flag(&self, key: &str, behavior: FlagBehavior, flag: MetadataValue<'ctx>)
        #[llvm_versions(7..)]
        pub fn add_basic_value_flag<BV: BasicValue<'ctx>>(&self, key: &str, behavior: FlagBehavior, flag: BV)
        #[llvm_versions(6..)]
        pub fn strip_debug_info(&self) -> bool
        #[llvm_versions(6..)]
        pub fn get_debug_metadata_version(&self) -> libc::c_uint
        #[llvm_versions(7..)]
        pub fn create_debug_info_builder(
        #[llvm_versions(13..)]
        pub fn run_passes(
    }
    
    impl Clone for Module<'_>
    impl Drop for Module<'_>
    #[llvm_versions(7..)]
    #[llvm_enum(LLVMModuleFlagBehavior)]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum FlagBehavior {
        #[llvm_variant(LLVMModuleFlagBehaviorError)]
        Error,
        #[llvm_variant(LLVMModuleFlagBehaviorWarning)]
        Warning,
        #[llvm_variant(LLVMModuleFlagBehaviorRequire)]
        Require,
        #[llvm_variant(LLVMModuleFlagBehaviorOverride)]
        Override,
        #[llvm_variant(LLVMModuleFlagBehaviorAppend)]
        Append,
        #[llvm_variant(LLVMModuleFlagBehaviorAppendUnique)]
        AppendUnique,
    }
    
    #[derive(Debug)]
    pub struct FunctionIterator<'ctx>(Option<FunctionValue<'ctx>>);
    
    impl<'ctx> Iterator for FunctionIterator<'ctx> {
        type Item = FunctionValue<'ctx>;
    
        fn next(&mut self) -> Option<Self::Item>
    }
    
    #[derive(Debug)]
    pub struct GlobalIterator<'ctx>(Option<GlobalValue<'ctx>>);
    
    impl<'ctx> Iterator for GlobalIterator<'ctx> {
        type Item = GlobalValue<'ctx>;
    
        fn next(&mut self) -> Option<Self::Item>
    }
    
    ```
    `src/object_file.rs`
    ```rust
    #[derive(Debug)]
    pub struct ObjectFile {
        object_file: LLVMObjectFileRef,
    }
    
    impl ObjectFile {
        pub unsafe fn new(object_file: LLVMObjectFileRef) -> Self
        pub fn as_mut_ptr(&self) -> LLVMObjectFileRef
        pub fn get_sections(&self) -> SectionIterator
        pub fn get_symbols(&self) -> SymbolIterator
    }
    
    impl Drop for ObjectFile
    #[derive(Debug)]
    pub struct SectionIterator {
        section_iterator: LLVMSectionIteratorRef,
        object_file: LLVMObjectFileRef,
        before_first: bool,
    }
    
    impl SectionIterator {
        pub unsafe fn new(section_iterator: LLVMSectionIteratorRef, object_file: LLVMObjectFileRef) -> Self
        pub fn as_mut_ptr(&self) -> (LLVMSectionIteratorRef, LLVMObjectFileRef)
    }
    
    impl Iterator for SectionIterator {
        type Item = Section;
    
        fn next(&mut self) -> Option<Self::Item>
    }
    
    impl Drop for SectionIterator
    #[derive(Debug)]
    pub struct Section {
        section: LLVMSectionIteratorRef,
        object_file: LLVMObjectFileRef,
    }
    
    impl Section {
        pub unsafe fn new(section: LLVMSectionIterator
Ref, object_file: LLVMObjectFileRef) -> Self
    pub unsafe fn as_mut_ptr(&self) -> (LLVMSectionIteratorRef, LLVMObjectFileRef)
    pub fn get_name(&self) -> Option<&CStr>
    pub fn size(&self) -> u64
    pub fn get_contents(&self) -> &[u8]
    pub fn get_address(&self) -> u64
    pub fn get_relocations(&self) -> RelocationIterator
}

#[derive(Debug)]
pub struct RelocationIterator {
    relocation_iterator: LLVMRelocationIteratorRef,
    section_iterator: LLVMSectionIteratorRef,
    object_file: LLVMObjectFileRef,
    before_first: bool,
}

impl RelocationIterator {
    pub unsafe fn new(
    pub fn as_mut_ptr(&self) -> (LLVMRelocationIteratorRef, LLVMSectionIteratorRef, LLVMObjectFileRef)
}

impl Iterator for RelocationIterator {
    type Item = Relocation;

    fn next(&mut self) -> Option<Self::Item>
}

impl Drop for RelocationIterator
#[derive(Debug)]
pub struct Relocation {
    relocation: LLVMRelocationIteratorRef,
    object_file: LLVMObjectFileRef,
}

impl Relocation {
    pub unsafe fn new(relocation: LLVMRelocationIteratorRef, object_file: LLVMObjectFileRef) -> Self
    pub fn as_mut_ptr(&self) -> (LLVMRelocationIteratorRef, LLVMObjectFileRef)
    pub fn get_offset(&self) -> u64
    pub fn get_symbols(&self) -> SymbolIterator
    pub fn get_type(&self) -> (u64, &CStr)
    pub fn get_value(&self) -> &CStr
}

#[derive(Debug)]
pub struct SymbolIterator {
    symbol_iterator: LLVMSymbolIteratorRef,
    object_file: LLVMObjectFileRef,
    before_first: bool,
}

impl SymbolIterator {
    pub unsafe fn new(symbol_iterator: LLVMSymbolIteratorRef, object_file: LLVMObjectFileRef) -> Self
    pub fn as_mut_ptr(&self) -> (LLVMSymbolIteratorRef, LLVMObjectFileRef)
}

impl Iterator for SymbolIterator {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item>
}

impl Drop for SymbolIterator
#[derive(Debug)]
pub struct Symbol {
    symbol: LLVMSymbolIteratorRef,
}

impl Symbol {
    pub unsafe fn new(symbol: LLVMSymbolIteratorRef) -> Self
    pub fn as_mut_ptr(&self) -> LLVMSymbolIteratorRef
    pub fn get_name(&self) -> Option<&CStr>
    pub fn size(&self) -> u64
    pub fn get_address(&self) -> u64
}

```
`src/passes.rs`
```rust
#[llvm_versions(..=16)]
use llvm_sys::core::LLVMGetGlobalPassRegistry;
#[llvm_versions(..=16)]
pub struct PassManagerBuilder {
    pass_manager_builder: LLVMPassManagerBuilderRef,
}

#[llvm_versions(..=16)]
impl PassManagerBuilder {
    pub unsafe fn new(pass_manager_builder: LLVMPassManagerBuilderRef) -> Self
    pub fn as_mut_ptr(&self) -> LLVMPassManagerBuilderRef
    pub fn create() -> Self
    pub fn set_optimization_level(&self, opt_level: OptimizationLevel)
    pub fn set_size_level(&self, size_level: u32)
    pub fn set_disable_unit_at_a_time(&self, disable: bool)
    pub fn set_disable_unroll_loops(&self, disable: bool)
    pub fn set_disable_simplify_lib_calls(&self, disable: bool)
    pub fn set_inliner_with_threshold(&self, threshold: u32)
    pub fn populate_function_pass_manager(&self, pass_manager: &PassManager<FunctionValue>)
    pub fn populate_module_pass_manager(&self, pass_manager: &PassManager<Module>)
    #[llvm_versions(..=14)]
    pub fn populate_lto_pass_manager(&self, pass_manager: &PassManager<Module>, internalize: bool, run_inliner: bool)
}

#[llvm_versions(..=16)]
impl Drop for PassManagerBuilder
pub trait PassManagerSubType {
    type Input;

    unsafe fn create<I: Borrow<Self::Input>>(input: I) -> LLVMPassManagerRef;
    unsafe fn run_in_pass_manager(&self, pass_manager: &PassManager<Self>) -> bool
    where
        Self: Sized;
}

impl PassManagerSubType for Module<'_>
impl<'ctx> PassManagerSubType for FunctionValue<'ctx>
#[derive(Debug)]
pub struct PassManager<T> {
    pub(crate) pass_manager: LLVMPassManagerRef,
    sub_type: PhantomData<T>,
}

impl PassManager<FunctionValue<'_>> {
    pub fn as_mut_ptr(&self) -> LLVMPassManagerRef
    pub fn initialize(&self) -> bool
    pub fn finalize(&self) -> bool
}

impl<T: PassManagerSubType> PassManager<T> {
    pub unsafe fn new(pass_manager: LLVMPassManagerRef) -> Self
    pub fn create<I: Borrow<T::Input>>(input: I) -> PassManager<T>
    pub fn run_on(&self, input: &T) -> bool
    #[llvm_versions(..=14)]
    pub fn add_argument_promotion_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_constant_merge_pass(&self)
    #[llvm_versions(10..=16)]
    pub fn add_merge_functions_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_dead_arg_elimination_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_function_attrs_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_function_inlining_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_always_inliner_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_global_dce_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_global_optimizer_pass(&self)
    #[llvm_versions(..=11)]
    pub fn add_ip_constant_propagation_pass(&self)
    #[llvm_versions(..=15)]
    pub fn add_prune_eh_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_ipsccp_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_internalize_pass(&self, all_but_main: bool)
    #[llvm_versions(..=16)]
    pub fn add_strip_dead_prototypes_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_strip_symbol_pass(&self)
    #[cfg(feature = "llvm4-0")]
    pub fn add_bb_vectorize_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_loop_vectorize_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_slp_vectorize_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_aggressive_dce_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_bit_tracking_dce_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_alignment_from_assumptions_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_cfg_simplification_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_dead_store_elimination_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_scalarizer_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_merged_load_store_motion_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_gvn_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_new_gvn_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_ind_var_simplify_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_instruction_combining_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_jump_threading_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_licm_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_loop_deletion_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_loop_idiom_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_loop_rotate_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_loop_reroll_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_loop_unroll_pass(&self)
    #[llvm_versions(..=14)]
    pub fn add_loop_unswitch_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_memcpy_optimize_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_partially_inline_lib_calls_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_lower_switch_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_promote_memory_to_register_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_reassociate_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_sccp_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_scalar_repl_aggregates_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_scalar_repl_aggregates_pass_ssa(&self)
    #[llvm_versions(..=16)]
    pub fn add_scalar_repl_aggregates_pass_with_threshold(&self, threshold: i32)
    #[llvm_versions(..=16)]
    pub fn add_simplify_lib_calls_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_tail_call_elimination_pass(&self)
    #[llvm_versions(..=11)]
    pub fn add_constant_propagation_pass(&self)
    #[llvm_versions(12..=16)]
    pub fn add_instruction_simplify_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_demote_memory_to_register_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_verifier_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_correlated_value_propagation_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_early_cse_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_early_cse_mem_ssa_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_lower_expect_intrinsic_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_type_based_alias_analysis_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_scoped_no_alias_aa_pass(&self)
    #[llvm_versions(..=16)]
    pub fn add_basic_alias_analysis_pass(&self)
    #[llvm_versions(7..=15)]
    pub fn add_aggressive_inst_combiner_pass(&self)
    #[llvm_versions(7..=16)]
    pub fn add_loop_unroll_and_jam_pass(&self)
    #[llvm_versions(8..15)]
    pub fn add_coroutine_early_pass(&self)
    #[llvm_versions(8..15)]
    pub fn add_coroutine_split_pass(&self)
    #[llvm_versions(8..15)]
    pub fn add_coroutine_elide_pass(&self)
    #[llvm_versions(8..15)]
    pub fn add_coroutine_cleanup_pass(&self)
}

impl<T> Drop for PassManager<T>
#[llvm_versions(..=16)]
#[derive(Debug)]
pub struct PassRegistry {
    pass_registry: LLVMPassRegistryRef,
}

#[llvm_versions(..=16)]
impl PassRegistry {
    pub unsafe fn new(pass_registry: LLVMPassRegistryRef) -> PassRegistry
    pub fn as_mut_ptr(&self) -> LLVMPassRegistryRef
    pub fn get_global() -> PassRegistry
    pub fn initialize_core(&self)
    pub fn initialize_transform_utils(&self)
    pub fn initialize_scalar_opts(&self)
    #[llvm_versions(..=15)]
    pub fn initialize_obj_carc_opts(&self)
    pub fn initialize_vectorization(&self)
    pub fn initialize_inst_combine(&self)
    pub fn initialize_ipo(&self)
    #[llvm_versions(..=15)]
    pub fn initialize_instrumentation(&self)
    pub fn initialize_analysis(&self)
    pub fn initialize_ipa(&self)
    pub fn initialize_codegen(&self)
    pub fn initialize_target(&self)
    #[llvm_versions(7..=15)]
    pub fn initialize_aggressive_inst_combiner(&self)
}

#[llvm_versions(13..)]
#[derive(Debug)]
pub struct PassBuilderOptions {
    pub(crate) options_ref: LLVMPassBuilderOptionsRef,
}

#[llvm_versions(13..)]
impl PassBuilderOptions {
    pub fn create() -> Self
    pub fn as_mut_ptr(&self) -> LLVMPassBuilderOptionsRef
    pub fn set_verify_each(&self, value: bool)
    pub fn set_debug_logging(&self, value: bool)
    pub fn set_loop_interleaving(&self, value: bool)
    pub fn set_loop_vectorization(&self, value: bool)
    pub fn set_loop_slp_vectorization(&self, value: bool)
    pub fn set_loop_unrolling(&self, value: bool)
    pub fn set_forget_all_scev_in_loop_unroll(&self, value: bool)
    pub fn set_licm_mssa_opt_cap(&self, value: u32)
    pub fn set_licm_mssa_no_acc_for_promotion_cap(&self, value: u32)
    pub fn set_call_graph_profile(&self, value: bool)
    pub fn set_merge_functions(&self, value: bool)
}

#[llvm_versions(13..)]
impl Drop for PassBuilderOptions

```
`src/support/error_handling.rs`
```rust
pub(crate) struct DiagnosticInfo {
    diagnostic_info: LLVMDiagnosticInfoRef,
}

impl DiagnosticInfo {
    pub unsafe fn new(diagnostic_info: LLVMDiagnosticInfoRef) -> Self
    pub(crate) fn get_description(&self) -> *mut ::libc::c_char
    pub(crate) fn severity_is_error(&self) -> bool
    fn severity(&self) -> LLVMDiagnosticSeverity
}

pub(crate) extern "C" fn get_error_str_diagnostic_handler(
pub unsafe fn install_fatal_error_handler(handler: extern "C" fn(*const ::libc::c_char))
pub fn reset_fatal_error_handler()

```
`src/support/mod.rs`
```rust
#[deny(missing_docs)]
pub mod error_handling;

use libc::c_char;
#[llvm_versions(16..)]
use llvm_sys::core::LLVMGetVersion;
use llvm_sys::core::{LLVMCreateMessage, LLVMDisposeMessage};
use llvm_sys::error_handling::LLVMEnablePrettyStackTrace;
use llvm_sys::support::{LLVMLoadLibraryPermanently, LLVMSearchForAddressOfSymbol};

use std::borrow::Cow;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Deref;
use std::path::Path;

#[derive(Eq)]
pub struct LLVMString {
    pub(crate) ptr: *const c_char,
}

impl LLVMString {
    pub(crate) unsafe fn new(ptr: *const c_char) -> Self
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String
    pub(crate) fn create_from_c_str(string: &CStr) -> LLVMString
    pub(crate) fn create_from_str(string: &str) -> LLVMString
}

impl Deref for LLVMString {
    type Target = CStr;

    fn deref(&self) -> &Self::Target
}

impl Debug for LLVMString
impl Display for LLVMString
impl PartialEq for LLVMString
impl Error for LLVMString
impl Drop for LLVMString
#[derive(Eq)]
pub(crate) enum LLVMStringOrRaw {
    Owned(LLVMString),
    Borrowed(*const c_char),
}

impl LLVMStringOrRaw {
    pub fn as_str(&self) -> &CStr
}

impl PartialEq for LLVMStringOrRaw
pub unsafe fn shutdown_llvm()
#[llvm_versions(16..)]
pub fn get_llvm_version() -> (u32, u32, u32)
#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum LoadLibraryError {
    #[error("The given path could not be converted to a `&str`")]
    UnicodeError,
    #[error("The given path could not be loaded as a library")]
    LoadingError,
}

pub fn load_library_permanently(path: &Path) -> Result<(), LoadLibraryError>
pub fn load_visible_symbols()
pub fn search_for_address_of_symbol(symbol: &str) -> Option<usize>
pub fn is_multithreaded() -> bool
pub fn enable_llvm_pretty_stack_trace()
pub(crate) fn to_c_str(mut s: &str) -> Cow<'_, CStr>
#[test]
fn test_to_c_str()

```

`src/types/traits.rs`
```rust
pub unsafe trait AsTypeRef {
    fn as_type_ref(&self) -> LLVMTypeRef;
}

pub unsafe trait AnyType<'ctx>: AsTypeRef + Debug {
    fn as_any_type_enum(&self) -> AnyTypeEnum<'ctx>
    fn print_to_string(&self) -> LLVMString
}

pub unsafe trait BasicType<'ctx>: AnyType<'ctx> {
    fn as_basic_type_enum(&self) -> BasicTypeEnum<'ctx>
    fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx>
    fn is_sized(&self) -> bool
    fn size_of(&self) -> Option<IntValue<'ctx>>
    fn array_type(&self, size: u32) -> ArrayType<'ctx>
    #[cfg_attr(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"), deprecated(note = "Starting from version 15.0, LLVM doesn't differentiate between pointer types. Use Context::ptr_type instead."))]
    fn ptr_type(&self, address_space: AddressSpace) -> PointerType<'ctx>
}

pub unsafe trait IntMathType<'ctx>: BasicType<'ctx> {
    type ValueType: IntMathValue<'ctx>;
    type MathConvType: FloatMathType<'ctx>;
    type PtrConvType: PointerMathType<'ctx>;
    unsafe fn new(value: LLVMValueRef) -> Self;
}

pub unsafe trait FloatMathType<'ctx>: BasicType<'ctx> {
    type ValueType: FloatMathValue<'ctx>;
    type MathConvType: IntMathType<'ctx>;
    unsafe fn new(value: LLVMValueRef) -> Self;
}

pub unsafe trait PointerMathType<'ctx>: BasicType<'ctx> {
    type ValueType: PointerMathValue<'ctx>;
    type PtrConvType: IntMathType<'ctx>;
    unsafe fn new(value: LLVMValueRef) -> Self;
}

pub unsafe trait VectorBaseValue<'ctx>: BasicType<'ctx> {
    unsafe fn new(value: LLVMValueRef) -> Self;
}

```
`src/types/vec_type.rs`
```rust
pub struct VectorType<'ctx> {
    vec_type: Type<'ctx>,
}

impl<'ctx> VectorType<'ctx> {
    pub unsafe fn new(vector_type: LLVMTypeRef) -> Self
    pub fn size_of(self) -> Option<IntValue<'ctx>>
    pub fn get_alignment(self) -> IntValue<'ctx>
    pub fn get_size(self) -> u32
    pub fn const_vector<V: BasicValue<'ctx>>(values: &[V]) -> VectorValue<'ctx>
    pub fn const_zero(self) -> VectorValue<'ctx>
    pub fn print_to_string(self) -> LLVMString
    pub fn get_undef(self) -> VectorValue<'ctx>
    #[llvm_versions(12..)]
    pub fn get_poison(self) -> VectorValue<'ctx>
    pub fn get_element_type(self) -> BasicTypeEnum<'ctx>
    #[cfg_attr(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"), deprecated(note = "Starting from version 15.0, LLVM doesn't differentiate between pointer types. Use Context::ptr_type instead."))]
    pub fn ptr_type(self, address_space: AddressSpace) -> PointerType<'ctx>
    pub fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx>
    pub fn array_type(self, size: u32) -> ArrayType<'ctx>
    pub fn const_array(self, values: &[VectorValue<'ctx>]) -> ArrayValue<'ctx>
    pub fn get_context(self) -> ContextRef<'ctx>
}

unsafe impl AsTypeRef for VectorType<'_>
impl Display for VectorType<'_>

```
`src/types/void_type.rs`
```rust
pub struct VoidType<'ctx> {
    void_type: Type<'ctx>,
}

impl<'ctx> VoidType<'ctx> {
    pub unsafe fn new(void_type: LLVMTypeRef) -> Self
    pub fn is_sized(self) -> bool
    pub fn get_context(self) -> ContextRef<'ctx>
    pub fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx>
    pub fn print_to_string(self) -> LLVMString
}

unsafe impl AsTypeRef for VoidType<'_>
impl Display for VoidType<'_>
