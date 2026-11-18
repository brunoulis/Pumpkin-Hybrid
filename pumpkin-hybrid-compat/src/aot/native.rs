use super::bytecode::{Function, Operation};
use crate::Result;
use cranelift_codegen::{
    ir::{AbiParam, InstBuilder, MemFlags, UserFuncName, condcodes::FloatCC, types},
    isa::TargetIsa,
    settings::{self, Configurable},
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::{path::Path, sync::Arc};

pub const ABI_VERSION: i32 = 1;
const ABI_SYMBOL: &str = "pumpkin_aot_abi";
const INVOKE_SYMBOL: &str = "pumpkin_aot_block_pos";

pub fn target() -> Result<Arc<dyn TargetIsa>> {
    let mut flags = settings::builder();
    flags.set("is_pic", "true")?;
    flags.set("opt_level", "speed")?;
    let builder = cranelift_native::builder().map_err(|reason| format!("Destino AOT: {reason}"))?;
    Ok(builder.finish(settings::Flags::new(flags))?)
}

pub fn emit(function: &Function, target: Arc<dyn TargetIsa>) -> Result<Vec<u8>> {
    let mut object = ObjectModule::new(ObjectBuilder::new(
        target,
        "pumpkin_aot",
        cranelift_module::default_libcall_names(),
    )?);
    let mut context = object.make_context();
    context
        .func
        .signature
        .returns
        .push(AbiParam::new(types::I32));
    let abi = object.declare_function(ABI_SYMBOL, Linkage::Export, &context.func.signature)?;
    context.func.name = UserFuncName::user(0, abi.as_u32());
    {
        let mut frontend = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut context.func, &mut frontend);
        let block = builder.create_block();
        builder.switch_to_block(block);
        builder.seal_block(block);
        let version = builder.ins().iconst(types::I32, i64::from(ABI_VERSION));
        builder.ins().return_(&[version]);
        builder.finalize();
    }
    object.define_function(abi, &mut context)?;
    object.clear_context(&mut context);

    let pointer = object.target_config().pointer_type();
    context
        .func
        .signature
        .params
        .extend([types::I32, types::F64, types::F64, types::F64, pointer].map(AbiParam::new));
    context
        .func
        .signature
        .returns
        .push(AbiParam::new(types::I32));
    let invoke =
        object.declare_function(INVOKE_SYMBOL, Linkage::Export, &context.func.signature)?;
    context.func.name = UserFuncName::user(0, invoke.as_u32());
    {
        let mut frontend = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut context.func, &mut frontend);
        let entry = builder.create_block();
        let null = builder.create_block();
        let present = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let params = builder.block_params(entry).to_vec();
        builder.ins().brif(params[0], present, &[], null, &[]);

        builder.switch_to_block(null);
        let null_exception = builder.ins().iconst(types::I32, 1);
        builder.ins().return_(&[null_exception]);

        builder.switch_to_block(present);
        let mut values = Vec::with_capacity(function.operations.len());
        for operation in &function.operations {
            let value = match *operation {
                Operation::Coordinate(axis) => params[axis + 1],
                Operation::Floor(value) => builder.ins().floor(values[value]),
                Operation::Round(value) => {
                    let value = values[value];
                    let floor = builder.ins().floor(value);
                    // Adding 0.5 before flooring rounds some inputs just below a half upward.
                    let fraction = builder.ins().fsub(value, floor);
                    let half = builder.ins().f64const(0.5);
                    let up = builder
                        .ins()
                        .fcmp(FloatCC::GreaterThanOrEqual, fraction, half);
                    let one = builder.ins().f64const(1.0);
                    let plus_one = builder.ins().fadd(floor, one);
                    let nearest = builder.ins().select(up, plus_one, floor);
                    builder.ins().fcvt_to_sint_sat(types::I64, nearest)
                }
                Operation::DoubleToInt(value) => {
                    builder.ins().fcvt_to_sint_sat(types::I32, values[value])
                }
                Operation::LongToInt(value) => builder.ins().ireduce(types::I32, values[value]),
            };
            values.push(value);
        }
        for (value, offset) in function.coordinates.into_iter().zip([0, 4, 8]) {
            builder
                .ins()
                .store(MemFlags::new(), values[value], params[4], offset);
        }
        let success = builder.ins().iconst(types::I32, 0);
        builder.ins().return_(&[success]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    object.define_function(invoke, &mut context)?;
    Ok(object.finish()?.emit()?)
}

pub fn invoke(module: &Path, input: Option<[f64; 3]>) -> Result<Option<[i32; 3]>> {
    type Invoke = unsafe extern "C" fn(i32, f64, f64, f64, *mut i32) -> i32;
    let present = i32::from(input.is_some());
    let [x, y, z] = input.unwrap_or_default();
    let mut output = [0; 3];

    // SAFETY: The caller holds the cache lock and verified the generated ELF checksum.
    // Both exports, their signatures and the three output stores are emitted above.
    let status = unsafe {
        let library = libloading::Library::new(module)?;
        let abi = library.get::<unsafe extern "C" fn() -> i32>(ABI_SYMBOL.as_bytes())?;
        if abi() != ABI_VERSION {
            return Err("Versión de ABI AOT incompatible".into());
        }
        let compiled = library.get::<Invoke>(INVOKE_SYMBOL.as_bytes())?;
        compiled(present, x, y, z, output.as_mut_ptr())
    };
    match status {
        0 => Ok(Some(output)),
        1 => Ok(None),
        _ => Err(format!("El módulo devolvió un estado AOT desconocido: {status}").into()),
    }
}
