use crate::ast::{AstType, Expression};
use crate::codegen::llvm::functions::calls as function_calls;
use crate::codegen::llvm::{LLVMCompiler, Type, VariableInfo};
use crate::error::CompileError;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::BasicValueEnum;
use std::sync::atomic::{AtomicUsize, Ordering};

static CLOSURE_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn compile_function_call<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    match expr {
        Expression::FunctionCall { name, args, .. } => {
            function_calls::compile_function_call(compiler, name, args)
        }
        _ => Err(CompileError::InternalError(
            format!("Expected FunctionCall, got {:?}", expr),
            compiler.get_current_span(),
        )),
    }
}

pub fn compile_method_call<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    match expr {
        Expression::MethodCall {
            object,
            method,
            type_args,
            args,
            ..
        } => compiler.compile_method_call_with_type_args(object, method, type_args, args),
        _ => Err(CompileError::InternalError(
            format!("Expected MethodCall, got {:?}", expr),
            compiler.get_current_span(),
        )),
    }
}

pub fn compile_closure<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    expr: &Expression,
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    match expr {
        Expression::Closure {
            params,
            body,
            return_type,
        } => {
            let closure_id = CLOSURE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let closure_name = format!("__closure_{}", closure_id);

            let ret_type = return_type
                .clone()
                .unwrap_or_else(|| compiler.infer_expression_type(body).unwrap_or(AstType::I32));

            let llvm_ret_type = compiler.to_llvm_type(&ret_type)?;

            let param_types: Vec<AstType> = params
                .iter()
                .map(|(_, opt_type)| opt_type.clone().unwrap_or(AstType::I32))
                .collect();

            // Convert param types to LLVM metadata types
            let mut param_metadata: Vec<BasicMetadataTypeEnum> =
                Vec::with_capacity(param_types.len());
            for t in &param_types {
                let llvm_ty = compiler.to_llvm_type(t)?;
                let basic = compiler.expect_basic_type(llvm_ty)?;
                param_metadata.push(basic.into());
            }

            // Build function type using shared helper
            let function_type = match llvm_ret_type {
                Type::Function(f) => f,
                _ => function_calls::build_fn_type_from_ret(
                    compiler,
                    llvm_ret_type,
                    &param_metadata,
                )?,
            };

            let closure_fn = compiler
                .module
                .add_function(&closure_name, function_type, None);

            let saved_function = compiler.current_function;
            let saved_block = compiler.builder.get_insert_block();
            let saved_variables = compiler.variables.clone();
            compiler.current_function = Some(closure_fn);

            let entry = compiler.context.append_basic_block(closure_fn, "entry");
            compiler.builder.position_at_end(entry);

            compiler.symbols.enter_scope();
            for (i, (param_name, param_type_opt)) in params.iter().enumerate() {
                let param_type = param_type_opt.clone().unwrap_or(AstType::I32);
                let llvm_param_type = compiler.to_llvm_type(&param_type)?;
                let basic_type = compiler.expect_basic_type(llvm_param_type)?;
                let alloca = compiler.builder.build_alloca(basic_type, param_name)?;
                let param_value = closure_fn.get_nth_param(i as u32).ok_or_else(|| {
                    CompileError::InternalError(
                        format!("Missing parameter {}", i),
                        compiler.get_current_span(),
                    )
                })?;
                compiler.builder.build_store(alloca, param_value)?;
                compiler.variables.insert(
                    param_name.clone(),
                    VariableInfo {
                        pointer: alloca,
                        ast_type: param_type,
                        is_mutable: true,
                        is_initialized: true,
                        definition_span: compiler.get_current_span(),
                    },
                );
            }

            let result = compiler.compile_expression(body)?;

            if compiler.current_block()?.get_terminator().is_none() {
                if matches!(ret_type, AstType::Void) {
                    compiler.builder.build_return(None)?;
                } else {
                    compiler.builder.build_return(Some(&result))?;
                }
            }

            compiler.symbols.exit_scope();
            compiler.variables = saved_variables;
            compiler.current_function = saved_function;
            if let Some(block) = saved_block {
                compiler.builder.position_at_end(block);
            }

            compiler.functions.insert(closure_name.clone(), closure_fn);
            compiler.function_types.insert(closure_name, ret_type);

            Ok(closure_fn.as_global_value().as_pointer_value().into())
        }
        _ => Err(CompileError::InternalError(
            format!("Expected Closure, got {:?}", expr),
            compiler.get_current_span(),
        )),
    }
}
