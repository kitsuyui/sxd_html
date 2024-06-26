use std::convert::TryFrom;

use html5ever::{tree_builder::NodeOrText, QualName};
use sxd_document::{
    dom::{ChildOfElement, ChildOfRoot, Document, ParentOfChild},
    QName,
};

use crate::Handle;

pub fn qualname_from_qname(qname: QName) -> QualName {
    QualName::new(
        None,
        qname.namespace_uri().unwrap_or_default().into(),
        qname.local_part().into(),
    )
}

pub fn qualname_as_qname(qualname: &QualName) -> QName {
    // let ns = if qualname.ns.is_empty() {
    //     None
    // } else {
    //     Some(qualname.ns.as_ref())
    // };
    //QName::with_namespace_uri(ns, qualname.local.as_ref())
    QName::with_namespace_uri(None, qualname.local.as_ref())
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

// pub fn deep_clone_element<'d>(elem: &Element<'d>) -> Element<'d> {
//     let document = elem.document();
//     let new_elem = document.create_element(elem.name());

//     for child in elem.children() {
//         let new_child = deep_clone_child_of_element(&child);
//         new_elem.append_child(new_child);
//     }

//     new_elem
// }

// pub fn deep_clone_text<'d>(text: &Text<'d>) -> Text<'d> {
//     let document = text.document();
//     document.create_text(text.text())
// }

// pub fn deep_clone_comment<'d>(comment: &Comment<'d>) -> Comment<'d> {
//     let document = comment.document();
//     document.create_comment(comment.text())
// }

// pub fn deep_clone_processing_instruction<'d>(
//     pi: &ProcessingInstruction<'d>,
// ) -> ProcessingInstruction<'d> {
//     let document = pi.document();
//     document.create_processing_instruction(pi.target(), pi.value())
// }

// pub fn deep_clone_child_of_element<'d>(coe: &ChildOfElement<'d>) -> ChildOfElement<'d> {
//     match coe {
//         ChildOfElement::Element(e) => deep_clone_element(e).into(),
//         ChildOfElement::Text(t) => deep_clone_text(t).into(),
//         ChildOfElement::Comment(c) => deep_clone_comment(c).into(),
//         ChildOfElement::ProcessingInstruction(pi) => deep_clone_processing_instruction(pi).into(),
//     }
// }
