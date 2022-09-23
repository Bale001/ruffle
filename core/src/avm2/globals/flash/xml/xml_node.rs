//! `flash.xml.XMLNode` builtin/prototype

use crate::avm2::object::TObject;
use crate::avm2::{Activation, Error, Object, QName, Value};

pub use crate::avm2::object::xml_allocator as xml_node_allocator;

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(xml) = this.and_then(|t| t.as_xml()).map(|xml| xml.as_node()) {
        let node_type = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let tag_name = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;

        xml.set_name(activation.context.gc_context, QName::dynamic_name(tag_name));
        xml.set_type(activation.context.gc_context, node_type as u8);
    }
    Ok(Value::Undefined)
}
