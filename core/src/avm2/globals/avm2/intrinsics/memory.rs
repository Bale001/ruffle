use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{Error, TObject};

pub fn casi32<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let address = args.get_i32(activation, 0)?;
    let expected = args.get_i32(activation, 1)?;
    let new_val = args.get_i32(activation, 2)?;
    let caller_domain = activation.caller_domain().domain_memory();
    let mut memory = caller_domain
        .as_bytearray_mut(activation.context.gc_context)
        .unwrap();

    let actual = memory
        .read_at(4, address as usize)
        .map_err(|err| err.to_avm(activation))?;
    if actual == &expected.to_le_bytes() {
        memory.write_at(&new_val.to_le_bytes(), address as usize)?;
    }
    Ok(Value::Undefined)
}
