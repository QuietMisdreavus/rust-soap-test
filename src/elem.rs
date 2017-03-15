use std::io::Write;

use xml::writer::{self, EventWriter, XmlEvent};

pub struct Element {
    pub name: Name,
    pub content: ElemContent,
}

pub struct Name {
    pub local_name: String,
    pub namespace: Option<String>,
    pub prefix: Option<String>,
}

pub enum ElemContent {
    Text(String),
    Children(Vec<Element>),
}

pub trait ToXml {
    fn to_xml(&self) -> Element;
}

pub trait FromXml: Sized {
    fn from_xml(Element) -> Self;
}

impl Element {
    pub fn serialize<W: Write>(&self, sink: &mut EventWriter<W>) -> writer::Result<()> {
        match (&self.name.namespace, &self.name.prefix) {
            (&Some(ref ns), &Some(ref prefix)) => {
                sink.write(XmlEvent::start_element(&self.name.local_name[..]).ns(&prefix[..], &ns[..]))?;
            },
            (&Some(ref ns), &None) => {
                sink.write(XmlEvent::start_element(&self.name.local_name[..]).default_ns(&ns[..]))?;
            },
            _ => {
                sink.write(XmlEvent::start_element(&self.name.local_name[..]))?;
            }
        }

        match &self.content {
            &ElemContent::Text(ref text) => {
                sink.write(&text[..])?;
            },
            &ElemContent::Children(ref children) => {
                for child in children {
                    child.serialize(sink)?;
                }
            },
        }

        sink.write(XmlEvent::end_element())?;

        Ok(())
    }
}