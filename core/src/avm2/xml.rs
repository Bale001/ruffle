use super::{property_map::PropertyMap, qname::QName, string::AvmString};
use gc_arena::{Collect, GcCell, MutationContext};

pub const ELEMENT_NODE: u8 = 1;
#[allow(dead_code)]
pub const TEXT_NODE: u8 = 3;

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
struct XmlData<'gc> {
    parent: Option<Xml<'gc>>,

    node_type: u8,

    tag_name: QName<'gc>,

    attributes: PropertyMap<'gc, AvmString<'gc>>,

    children: Vec<Xml<'gc>>,
}

#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct Xml<'gc>(GcCell<'gc, XmlData<'gc>>);

impl<'gc> Xml<'gc> {
    pub fn new(mc: MutationContext<'gc, '_>, tag_name: QName<'gc>, node_type: u8) -> Self {
        Self(GcCell::allocate(
            mc,
            XmlData {
                tag_name,
                node_type,
                parent: None,
                attributes: PropertyMap::new(),
                children: Vec::new(),
            },
        ))
    }

    pub fn set_type(&self, mc: MutationContext<'gc, '_>, node_type: u8) {
        self.0.write(mc).node_type = node_type;
    }

    pub fn set_name(&self, mc: MutationContext<'gc, '_>, tag_name: QName<'gc>) {
        self.0.write(mc).tag_name = tag_name;
    }
}
