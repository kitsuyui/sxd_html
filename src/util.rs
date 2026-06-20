use std::convert::TryFrom;

use html5ever::{tree_builder::NodeOrText, QualName};
use sxd_document::{
    dom::{ChildOfElement, ChildOfRoot, Document, ParentOfChild},
    QName,
};

use crate::Handle;

const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

pub fn qualname_as_qname<'a>(qualname: &'a QualName) -> QName<'a> {
    let namespace_uri = match qualname.ns.as_ref() {
        "" | HTML_NAMESPACE => None,
        namespace_uri => Some(namespace_uri),
    };

    QName::with_namespace_uri(namespace_uri, qualname.local.as_ref())
}

pub fn node_or_text_into_child_of_root(node_or_text: NodeOrText<Handle>) -> ChildOfRoot {
    match node_or_text {
        NodeOrText::AppendNode(handle) =>
        {
            #[allow(clippy::expect_used)]
            ChildOfRoot::try_from(handle).expect("Cannot convert to ChildOfRoot")
        }
        NodeOrText::AppendText(_) => panic!("Text cannot be made into ChildOfRoot"),
    }
}

pub fn node_or_text_into_child_of_element<'d>(
    document: &Document<'d>,
    node_or_text: NodeOrText<Handle<'d>>,
) -> ChildOfElement<'d> {
    match node_or_text {
        NodeOrText::AppendNode(handle) =>
        {
            #[allow(clippy::expect_used)]
            ChildOfElement::try_from(handle).expect("Cannot convert to ChildOfElement")
        }
        NodeOrText::AppendText(text) => ChildOfElement::from(document.create_text(text.as_ref())),
    }
}

pub fn child_of_element_remove_from_parent(coe: &ChildOfElement) {
    match coe {
        ChildOfElement::Element(x) => x.remove_from_parent(),
        ChildOfElement::Text(x) => x.remove_from_parent(),
        ChildOfElement::Comment(x) => x.remove_from_parent(),
        ChildOfElement::ProcessingInstruction(x) => x.remove_from_parent(),
    }
}

pub fn parent_of_child_append_node_or_text(poc: &ParentOfChild, noe: NodeOrText<Handle>) {
    match poc {
        ParentOfChild::Root(r) => r.append_child(node_or_text_into_child_of_root(noe)),
        ParentOfChild::Element(e) => {
            e.append_child(node_or_text_into_child_of_element(&e.document(), noe))
        }
    }
}
