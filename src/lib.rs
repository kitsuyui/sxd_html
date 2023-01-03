mod error;
mod handle;
mod util;
use html5ever::driver::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;

pub use error::Error;
pub(crate) use handle::Handle;

use html5ever::{
    tendril::Tendril,
    tree_builder::{NodeOrText, TreeSink},
    LocalName, Namespace, QualName,
};
use sxd_document::{
    dom::{ChildOfElement, Document},
    Package,
};

#[derive(Debug)]
struct DocHtmlSink<'d> {
    document: Document<'d>,
    document_handle: Handle<'d>,
    errors: Vec<Error>,
    current_line: u64,
}

impl<'d> DocHtmlSink<'d> {
    fn new(document: Document<'d>) -> Self {
        let document_handle = Handle::Document(document.root());

        Self {
            document,
            document_handle,
            errors: Default::default(),
            current_line: 0,
        }
    }
}

impl<'d> TreeSink for DocHtmlSink<'d> {
    type Handle = Handle<'d>;
    type Output = Vec<Error>;

    fn set_current_line(&mut self, line_number: u64) {
        self.current_line = line_number;
    }

    fn finish(self) -> Self::Output {
        self.errors
    }

    fn parse_error(&mut self, msg: std::borrow::Cow<'static, str>) {
        self.errors.push(Error::new(self.current_line, msg));
    }

    fn get_document(&mut self) -> Self::Handle {
        self.document_handle.clone()
    }

    // this is only called on elements
    fn elem_name<'h>(&'h self, target: &'h Self::Handle) -> html5ever::ExpandedName<'h> {
        match target {
            Handle::Element(_, qualname, _) => qualname.expanded(),
            _ => panic!("not an element"),
        }
    }

    fn create_element(
        &mut self,
        name: html5ever::QualName,
        attrs: Vec<html5ever::Attribute>,
        flags: html5ever::tree_builder::ElementFlags,
    ) -> Self::Handle {
        let qname = util::qualname_as_qname(&name);
        let elem = self.document.create_element(qname);

        for attr in attrs {
            let qname = util::qualname_as_qname(&attr.name);
            elem.set_attribute_value(qname, attr.value.as_ref());
        }

        Handle::Element(elem, name, flags.template)
    }

    fn create_comment(&mut self, text: html5ever::tendril::StrTendril) -> Self::Handle {
        let comment = self.document.create_comment(text.as_ref());
        Handle::Comment(comment)
    }

    fn create_pi(
        &mut self,
        target: html5ever::tendril::StrTendril,
        data: html5ever::tendril::StrTendril,
    ) -> Self::Handle {
        let data = if data.is_empty() {
            None
        } else {
            Some(data.as_ref())
        };

        let pi = self
            .document
            .create_processing_instruction(target.as_ref(), data);

        Handle::ProcessingInstruction(pi)
    }

    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        match parent {
            Handle::Document(root) => {
                // text cant be appended to root so no need to concatenate it
                let child = util::node_or_text_into_child_of_root(child);
                root.append_child(child);
            }
            Handle::Element(elem, _, _) => {
                let last = elem.children().into_iter().last();

                match (last, child) {
                    (Some(ChildOfElement::Text(x)), NodeOrText::AppendText(y)) => {
                        let mut new_text = x.text().to_string();
                        new_text.push_str(y.as_ref());
                        x.set_text(&new_text);
                    }
                    (_, child) => {
                        let document = elem.document();
                        let child = util::node_or_text_into_child_of_element(&document, child);
                        elem.append_child(child);
                    }
                }
            }
            _ => panic!("Can only appent into document or element"),
        }
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        // https://github.com/servo/html5ever/blob/master/rcdom/lib.rs#L348

        let parent = element.parent();
        let has_parent = parent.is_some();

        if has_parent {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &mut self,
        _name: html5ever::tendril::StrTendril,
        _public_id: html5ever::tendril::StrTendril,
        _system_id: html5ever::tendril::StrTendril,
    ) {
        // ignored, cant seem to find a way to add doctype using sxd_document
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        // this template stuff is probably not well done but seems to work
        match target {
            Handle::Element(_, _, true) => target.clone(),
            _ => panic!("not a template element"),
        }
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&mut self, _mode: html5ever::tree_builder::QuirksMode) {
        // ignored
    }

    fn append_before_sibling(
        &mut self,
        sibling: &Self::Handle,
        new_node: NodeOrText<Self::Handle>,
    ) {
        let parent = sibling.parent().expect("must have a parent");

        let children = {
            let mut v = vec![ChildOfElement::from(sibling.clone())];
            v.extend(sibling.following_siblings().into_iter());
            v
        };

        for child in children.iter() {
            util::child_of_element_remove_from_parent(child);
        }

        util::parent_of_child_append_node_or_text(&parent, new_node);
        let parent_handle = Handle::from(parent);
        for child in children {
            let node_or_text = match child {
                ChildOfElement::Text(t) => NodeOrText::AppendText(Tendril::from(t.text())),
                coe => NodeOrText::AppendNode(Handle::from(coe)),
            };
            self.append(&parent_handle, node_or_text);
        }
    }

    // this is only called on elements
    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        let elem = target.element_ref();
        for attr in attrs {
            let qname = util::qualname_as_qname(&attr.name);
            if elem.attribute_value(qname).is_some() {
                continue;
            }

            elem.set_attribute_value(qname, attr.value.as_ref());
        }
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        target.remove_from_parent();
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        let node = node.element_ref();
        let new_parent = new_parent.element_ref();

        let children = node.children();
        node.clear_children();
        new_parent.append_children(children);
    }
}

pub fn parse_html(contents: &str) -> Package {
    parse_html_with_errors(contents).0
}

pub fn parse_html_fragment(contents: &str) -> Package {
    parse_html_fragment_with_errors(contents).0
}

pub fn parse_html_with_errors(contents: &str) -> (Package, Vec<Error>) {
    let package = Package::new();
    let document = package.as_document();
    let sink = DocHtmlSink::new(document);

    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            exact_errors: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let parser = html5ever::parse_document(sink, opts);
    let errors = parser.one(contents);

    (package, errors)
}

pub fn parse_html_fragment_with_errors(contents: &str) -> (Package, Vec<Error>) {
    let package = Package::new();
    let document = package.as_document();
    let sink = DocHtmlSink::new(document);

    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            exact_errors: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let parser = html5ever::parse_fragment(
        sink,
        opts,
        QualName::new(None, Namespace::default(), LocalName::from("")),
        Default::default(),
    );
    let errors = parser.one(contents);

    (package, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Error;

    #[cfg(test)]
    fn get_html_result(html: &str, xpath: &str) -> Result<String, Error> {
        let package = parse_html(html);
        let root = package.as_document().root();
        let value = evaluate_xpath_node(root, xpath).map_err(|_| Error)?;
        Ok(value.string())
    }

    #[cfg(test)]
    fn get_html_fragment_result(html: &str, xpath: &str) -> Result<String, Error> {
        let package = parse_html_fragment(html);
        let root = package.as_document().root();
        let value = evaluate_xpath_node(root, xpath).map_err(|_| Error)?;
        Ok(value.string())
    }

    #[cfg(test)]
    fn get_xml_result(xml: &str, xpath: &str) -> Result<String, Error> {
        let xml = sxd_document::parser::parse(xml).map_err(|_| Error)?;
        let value = evaluate_xpath_node(xml.as_document().root(), xpath).map_err(|_| Error)?;
        Ok(value.string())
    }

    #[cfg(test)]
    fn evaluate_xpath_node<'d>(
        node: impl Into<sxd_xpath::nodeset::Node<'d>>,
        expr: &str,
    ) -> Result<sxd_xpath::Value<'d>, sxd_xpath::Error> {
        let factory = sxd_xpath::Factory::new();
        let expression = factory.build(expr)?;
        let expression = expression.ok_or(sxd_xpath::Error::NoXPath)?;
        let context = sxd_xpath::Context::new();
        expression
            .evaluate(&context, node.into())
            .map_err(Into::into)
    }

    #[test]
    fn test_parse_html() {
        let html = r#"<!DOCTYPE html>
<html>
  <head>
    <title>Test</title>
  </head>
  <body>
    <div id="test">Hello World</div>
  </body>
</html>"#;
        let result = get_html_result(html, "/html/body/div/text()").unwrap();
        assert_eq!(result, "Hello World");
        let result = get_html_result(html, "/html/head/title/text()").unwrap();
        assert_eq!(result, "Test");
    }

    #[test]
    fn test_comparison_with_xml() {
        // This test is to ensure that the html parser is working as expected
        // sxd_document is well tested and is used to parse the xml
        // so if this test fails then it is likely that the html parser is not working
        let result_xml =
            get_xml_result("<root><child>text</child></root>", "//root/child/text()").unwrap();
        let result_html =
            get_html_result("<root><child>text</child></root>", "//root/child/text()").unwrap();
        assert_eq!(result_xml, "text");
        assert_eq!(result_html, "text");

        let result_xml =
            get_xml_result("<root><child>text</child></root>", "//root/child").unwrap();
        let result_html =
            get_html_result("<root><child>text</child></root>", "//root/child").unwrap();
        assert_eq!(result_xml, "text");
        assert_eq!(result_html, "text");

        let result_xml =
            get_xml_result("<table><tr><td>text</td></tr></table>", "//table//tr//td").unwrap();
        let result_html =
            get_html_result("<table><tr><td>text</td></tr></table>", "//table//tr//td").unwrap();
        assert_eq!(result_xml, "text");
        assert_eq!(result_html, "text");

        let result_xml =
            get_xml_result("<table><tr><td>text</td></tr></table>", "//table/tr/td").unwrap();
        let result_html =
            get_html_result("<table><tr><td>text</td></tr></table>", "//table/tr/td").unwrap();
        let result_html2 = get_html_result(
            "<table><tr><td>text</td></tr></table>",
            "//table/tbody/tr/td",
        )
        .unwrap();
        assert_eq!(result_xml, "text");
        assert_eq!(result_html, "");
        assert_eq!(result_html2, "text"); // tbody is added by html5ever

        let result_xml =
            get_xml_result("<table><tr><td>text</td></tr></table>", "//table").unwrap();
        let result_html =
            get_html_result("<table><tr><td>text</td></tr></table>", "//table").unwrap();
        assert_eq!(result_xml, "text");
        assert_eq!(result_html, "text");

        let x1 = get_xml_result("<tr><td>text</td></tr>", "//tr").unwrap();
        let x2 = get_html_result("<tr><td>text</td></tr>", "//tr").unwrap(); // html5ever vanishes the tr because it is not in a table
        let x3 = get_html_fragment_result("<tr><td>text</td></tr>", "//tr").unwrap(); // fragment mode does not vanish the tr
        assert_eq!(x1, "text");
        assert_eq!(x2, "");
        assert_eq!(x3, "text");
    }

    #[test]
    fn test_comparison_with_xml_nested_xpath() {
        let base_html = "<root><child>text</child></root>";
        let node = sxd_document::parser::parse(base_html).unwrap();
        let result_xml = evaluate_xpath_node(node.as_document().root(), "//root");
        let mut results = vec![];
        match result_xml {
            Ok(sxd_xpath::Value::Nodeset(set)) => {
                assert_eq!(set.size(), 1);
                for elm in set.document_order() {
                    let result = evaluate_xpath_node(elm, "//child/text()").unwrap();
                    results.push(result.string());
                }
            }
            _ => panic!("Error"),
        }
        assert_eq!(results, vec!["text"]);

        let node = parse_html(base_html);
        let result_html = evaluate_xpath_node(node.as_document().root(), "//root");
        let mut results = vec![];
        match result_html {
            Ok(sxd_xpath::Value::Nodeset(set)) => {
                assert_eq!(set.size(), 1);
                for elm in set.document_order() {
                    let result = evaluate_xpath_node(elm, "//child/text()").unwrap();
                    results.push(result.string());
                }
            }
            _ => panic!("Error"),
        }
        assert_eq!(results, vec!["text"]);

        let base_html2 = "<root><child>text1</child><child>text2</child></root>";
        let node = sxd_document::parser::parse(base_html2).unwrap();
        let result_xml = evaluate_xpath_node(node.as_document().root(), "//root/*");
        let mut results = vec![];
        match result_xml {
            Ok(sxd_xpath::Value::Nodeset(set)) => {
                assert_eq!(set.size(), 2);
                for elm in set.document_order().iter() {
                    let result = evaluate_xpath_node(*elm, "./text()").unwrap();
                    results.push(result.string());
                }
            }
            _ => panic!("Error"),
        }
        assert_eq!(results, vec!["text1", "text2"]);

        let node = parse_html(base_html2);
        let result_html = evaluate_xpath_node(node.as_document().root(), "//root/*");
        let mut results = vec![];
        match result_html {
            Ok(sxd_xpath::Value::Nodeset(set)) => {
                assert_eq!(set.size(), 2);
                for elm in set.document_order().iter() {
                    let result = evaluate_xpath_node(*elm, "./text()").unwrap();
                    results.push(result.string());
                }
            }
            _ => panic!("Error"),
        }
        assert_eq!(results, vec!["text1", "text2"]);

        let base_html3 = r#"<root>
            <mytable>
                <mytr>
                    <mytd>text1</mytd>
                    <mytd>text2</mytd>
                </mytr>
                <mytr>
                    <mytd>text3</mytd>
                    <mytd>text4</mytd>
                </mytr>
            </mytable>
        </root>"#;
        let node = sxd_document::parser::parse(base_html3).unwrap();
        let result_xml = evaluate_xpath_node(node.as_document().root(), "//root/mytable/mytr");
        let mut results = vec![];
        match result_xml {
            Ok(sxd_xpath::Value::Nodeset(set)) => {
                assert_eq!(set.size(), 2);
                for elm in set.document_order().iter() {
                    match evaluate_xpath_node(*elm, "./mytd/text()").unwrap() {
                        sxd_xpath::Value::Nodeset(set2) => {
                            for elm in set2.document_order().iter() {
                                results.push(elm.string_value());
                            }
                        }
                        _ => panic!("Error"),
                    }
                }
            }
            _ => panic!("Error"),
        }
        assert_eq!(results, vec!["text1", "text2", "text3", "text4"]);

        let node = parse_html(base_html3);
        let result_html = evaluate_xpath_node(node.as_document().root(), "//root/mytable/mytr");
        let mut results = vec![];
        match result_html {
            Ok(sxd_xpath::Value::Nodeset(set)) => {
                assert_eq!(set.size(), 2);
                for elm in set.document_order().iter() {
                    match evaluate_xpath_node(*elm, "./mytd/text()").unwrap() {
                        sxd_xpath::Value::Nodeset(set2) => {
                            for elm in set2.document_order().iter() {
                                results.push(elm.string_value());
                            }
                        }
                        _ => panic!("Error"),
                    }
                }
            }
            _ => panic!("Error"),
        }
        assert_eq!(results, vec!["text1", "text2", "text3", "text4"]);
    }

    #[test]
    fn test_comparison_with_xml_nested_xpath_table() {
        let base_html3 = r#"<root>
            <table>
                <tr>
                    <td>text1</td>
                    <td>text2</td>
                </tr>
                <tr>
                    <td>text3</td>
                    <td>text4</td>
                </tr>
            </table>
        </root>"#;
        let node = sxd_document::parser::parse(base_html3).unwrap();
        let result_xml = evaluate_xpath_node(node.as_document().root(), "//root/table/tr");
        let mut results = vec![];
        match result_xml {
            Ok(sxd_xpath::Value::Nodeset(set)) => {
                assert_eq!(set.size(), 2);
                for elm in set.document_order().iter() {
                    match evaluate_xpath_node(*elm, "./td/text()").unwrap() {
                        sxd_xpath::Value::Nodeset(set2) => {
                            for elm in set2.document_order().iter() {
                                results.push(elm.string_value());
                            }
                        }
                        _ => panic!("Error"),
                    }
                }
            }
            _ => panic!("Error"),
        }
        assert_eq!(results, vec!["text1", "text2", "text3", "text4"]);

        let node = parse_html(base_html3);
        let result_html = evaluate_xpath_node(node.as_document().root(), "//root/table//tr");
        let mut results = vec![];
        match result_html {
            Ok(sxd_xpath::Value::Nodeset(set)) => {
                assert_eq!(set.size(), 2);
                for elm in set.document_order().iter() {
                    match evaluate_xpath_node(*elm, "./td/text()").unwrap() {
                        sxd_xpath::Value::Nodeset(set2) => {
                            for elm in set2.document_order().iter() {
                                results.push(elm.string_value());
                            }
                        }
                        _ => panic!("Error"),
                    }
                }
            }
            _ => panic!("Error"),
        }
        assert_eq!(results, vec!["text1", "text2", "text3", "text4"]);
    }
}
