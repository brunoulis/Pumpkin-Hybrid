use crate::Result;
use cafebabe::{
    MethodAccessFlags,
    attributes::{AttributeData, CodeData},
    bytecode::Opcode,
    constant_pool::MemberRef,
};
use std::io::{Read, Seek};

pub const CLASS: &str = "dev/mrsnowy/teleport_commands/utils/tools";
pub const METHOD: &str = "roundedBlockPos";
pub const DESCRIPTOR: &str = "(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/core/BlockPos;";
const BLOCK_POS: &str = "net/minecraft/core/BlockPos";

pub type ValueId = usize;

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Coordinate(usize),
    Floor(ValueId),
    Round(ValueId),
    DoubleToInt(ValueId),
    LongToInt(ValueId),
}

#[derive(Debug)]
pub struct Function {
    pub operations: Vec<Operation>,
    pub coordinates: [ValueId; 3],
}

#[derive(Clone, Copy)]
enum Value {
    Input,
    Uninitialized,
    BlockPos,
    Double(ValueId),
    Long(ValueId),
    Int(ValueId),
}

pub fn read_class(jar: impl Read + Seek) -> Result<Vec<u8>> {
    let mut archive = zip::ZipArchive::new(jar)?;
    let mut class = archive.by_name(&format!("{CLASS}.class"))?;
    // Bound decompression before giving third-party input to the class file parser.
    const CLASS_LIMIT: u64 = 8 * 1024 * 1024;
    if class.size() > CLASS_LIMIT {
        return Err(format!("{CLASS}.class supera el límite de {CLASS_LIMIT} bytes").into());
    }
    let mut bytes = Vec::with_capacity(class.size() as usize);
    class.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub(super) fn method_code(class: &cafebabe::ClassFile<'_>) -> Result<&CodeData<'_>> {
    if class.this_class.as_str() != CLASS || class.major_version != 69 || class.minor_version != 0 {
        return Err(format!("Se esperaba {CLASS} en formato de clases Java 25 sin preview").into());
    }
    if class.methods.iter().any(|method| method.name == "<clinit>") {
        return Err(format!("{CLASS} requiere inicialización estática").into());
    }
    let mut candidates = class
        .methods
        .iter()
        .filter(|method| method.name == METHOD && method.descriptor.to_string() == DESCRIPTOR);
    let method = candidates
        .next()
        .filter(|method| method.access_flags.contains(MethodAccessFlags::STATIC))
        .ok_or_else(|| format!("Falta el método estático {CLASS}.{METHOD}{DESCRIPTOR}"))?;
    if candidates.next().is_some() {
        return Err(format!("{CLASS}.{METHOD} está duplicado").into());
    }
    let mut code = method
        .attributes
        .iter()
        .filter_map(|attribute| match &attribute.data {
            AttributeData::Code(code) => Some(code),
            _ => None,
        });
    let body = code.next().ok_or("Falta el bytecode del método AOT")?;
    if code.next().is_some() || !body.exception_table.is_empty() || body.max_locals == 0 {
        return Err("El perfil inicial requiere un método con un cuerpo y sin manejadores".into());
    }
    Ok(body)
}

pub fn translate(bytes: &[u8]) -> Result<Function> {
    let class = cafebabe::parse_class(bytes)?;
    let code = method_code(&class)?;
    let instructions = &code
        .bytecode
        .as_ref()
        .ok_or("Falta bytecode decodificado")?
        .opcodes;
    let mut operations = Vec::new();
    let mut stack = Vec::new();
    let mut reserved = false;
    let mut coordinates = None;

    for (offset, opcode) in instructions {
        let fail = || {
            format!(
                "{CLASS}.{METHOD}{DESCRIPTOR}, bytecode @{offset}: {opcode:?} fuera del perfil o pila inválida"
            )
        };
        let mut emit = |operation| {
            let id = operations.len();
            operations.push(operation);
            id
        };
        match opcode {
            Opcode::New(class) if class == BLOCK_POS && !reserved => {
                reserved = true;
                stack.push(Value::Uninitialized);
            }
            Opcode::Dup => {
                let value = *stack.last().ok_or_else(fail)?;
                if matches!(value, Value::Double(_) | Value::Long(_)) {
                    return Err(fail().into());
                }
                stack.push(value);
            }
            Opcode::Aload(0) => stack.push(Value::Input),
            Opcode::Invokevirtual(member) => {
                let axis = match member.name_and_type.name.as_ref() {
                    "x" => 0,
                    "y" => 1,
                    "z" => 2,
                    _ => return Err(fail().into()),
                };
                if !matches_member(member, "net/minecraft/world/phys/Vec3", "()D")
                    || !matches!(stack.pop(), Some(Value::Input))
                {
                    return Err(fail().into());
                }
                stack.push(Value::Double(emit(Operation::Coordinate(axis))));
            }
            Opcode::Invokestatic(member) => {
                let Some(Value::Double(value)) = stack.pop() else {
                    return Err(fail().into());
                };
                let result = match member.name_and_type.name.as_ref() {
                    "floor" if matches_member(member, "java/lang/Math", "(D)D") => {
                        Value::Double(emit(Operation::Floor(value)))
                    }
                    "round" if matches_member(member, "java/lang/Math", "(D)J") => {
                        Value::Long(emit(Operation::Round(value)))
                    }
                    _ => return Err(fail().into()),
                };
                stack.push(result);
            }
            Opcode::D2i => {
                let Some(Value::Double(value)) = stack.pop() else {
                    return Err(fail().into());
                };
                stack.push(Value::Int(emit(Operation::DoubleToInt(value))));
            }
            Opcode::L2i => {
                let Some(Value::Long(value)) = stack.pop() else {
                    return Err(fail().into());
                };
                stack.push(Value::Int(emit(Operation::LongToInt(value))));
            }
            Opcode::Invokespecial(member)
                if member.name_and_type.name == "<init>"
                    && matches_member(member, BLOCK_POS, "(III)V") =>
            {
                let (
                    Some(Value::Int(z)),
                    Some(Value::Int(y)),
                    Some(Value::Int(x)),
                    Some(Value::Uninitialized),
                ) = (stack.pop(), stack.pop(), stack.pop(), stack.pop())
                else {
                    return Err(fail().into());
                };
                for value in &mut stack {
                    if matches!(value, Value::Uninitialized) {
                        *value = Value::BlockPos;
                    }
                }
                coordinates = Some([x, y, z]);
            }
            Opcode::Areturn
                if matches!(stack.as_slice(), [Value::BlockPos])
                    && instructions.last().is_some_and(|(last, _)| last == offset) =>
            {
                return Ok(Function {
                    operations,
                    coordinates: coordinates.ok_or_else(fail)?,
                });
            }
            _ => return Err(fail().into()),
        }
        let words: usize = stack
            .iter()
            .map(|v| {
                if matches!(v, Value::Double(_) | Value::Long(_)) {
                    2
                } else {
                    1
                }
            })
            .sum();
        if words > usize::from(code.max_stack) {
            return Err(format!("{CLASS}.{METHOD}, bytecode @{offset}: max_stack excedido").into());
        }
    }
    Err(format!("{CLASS}.{METHOD} termina sin retorno válido").into())
}

fn matches_member(member: &MemberRef<'_>, class: &str, descriptor: &str) -> bool {
    member.class_name == class && member.name_and_type.descriptor == descriptor
}
