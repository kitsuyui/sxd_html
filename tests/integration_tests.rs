#[cfg(test)]
mod tests {
    use sxd_document::dom::{ChildOfElement, Element};
    use sxd_xpath::{Context, Factory, Value};

    const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

    fn child_kinds(element: Element<'_>) -> Vec<String> {
        element
            .children()
            .iter()
            .map(|child| match child {
                ChildOfElement::Element(element) => {
                    format!("element:{}", element.name().local_part())
                }
                ChildOfElement::Text(text) => format!("text:{}", text.text()),
                ChildOfElement::Comment(comment) => format!("comment:{}", comment.text()),
                ChildOfElement::ProcessingInstruction(pi) => {
                    format!("pi:{}", pi.target())
                }
            })
            .collect()
    }

    #[test]
    fn parse_simple() {
        let contents = "<!DOCTYPE html><html><div>hello<br>bye</div></html>";
        let (package, errors) = sxd_html::parse_html_with_errors(contents);
        assert_eq!(0, errors.len());

        let root = package.as_document().root();
        let root = root.children()[0]
            .element()
            .expect("html should be root element");

        assert_eq!("html", root.name().local_part());
        assert!(root.name().namespace_uri().is_none());

        let children = root.children();
        // head and body are added if not present
        assert_eq!(2, children.len());

        let head = children[0].element().unwrap();
        let body = children[1].element().unwrap();

        assert_eq!("head", head.name().local_part());
        assert!(head.name().namespace_uri().is_none());
        assert_eq!(0, head.children().len());

        let children = body.children();
        assert_eq!("body", body.name().local_part());
        assert!(body.name().namespace_uri().is_none());
        assert_eq!(1, children.len());

        let c0 = children[0].element().unwrap();
        let children = c0.children();
        assert_eq!("div", c0.name().local_part());
        assert!(c0.name().namespace_uri().is_none());
        assert_eq!(3, children.len());

        let c0 = children[0].text().unwrap();
        let c1 = children[1].element().unwrap();
        let c2 = children[2].text().unwrap();

        assert_eq!("hello", c0.text());

        assert_eq!("br", c1.name().local_part());
        assert_eq!(0, c1.children().len());

        assert_eq!("bye", c2.text());
    }

    #[test]
    fn preserves_svg_namespace_for_xpath() {
        let contents = concat!(
            "<!DOCTYPE html><html><body>",
            r#"<svg width="10"><circle id="dot"></circle></svg>"#,
            "</body></html>",
        );
        let (package, errors) = sxd_html::parse_html_with_errors(contents);
        assert_eq!(0, errors.len());

        let factory = Factory::new();
        let expression = factory.build("//svg:circle").unwrap().unwrap();
        let root = package.as_document().root();
        let mut context = Context::new();
        context.set_namespace("svg", SVG_NAMESPACE);

        let value = expression.evaluate(&context, root).unwrap();
        let nodes = match value {
            Value::Nodeset(nodes) => nodes,
            _ => panic!("expected node set"),
        };

        assert_eq!(1, nodes.size());
    }

    #[test]
    fn parse_errors_are_inspectable() {
        let contents = "<!DOCTYPE html>\n<html><body><p></div></body></html>";
        let (_package, errors) = sxd_html::parse_html_with_errors(contents);

        let error = errors
            .first()
            .expect("mismatched tags should produce a parse error");
        assert!(error.line() > 0);
        assert!(!error.message().is_empty());
    }

    #[test]
    fn preserves_foster_parented_siblings_around_table() {
        let contents = concat!(
            "<!DOCTYPE html><html><body><div>",
            "<!--before--><table><tr><td>cell</td></tr>alpha<span>tail</span></table>",
            "<p>after</p>",
            "</div></body></html>",
        );
        let (package, errors) = sxd_html::parse_html_with_errors(contents);
        assert!(
            !errors.is_empty(),
            "malformed table content should still be observable through parse_html"
        );

        let root = package.as_document().root();
        let html = root.children()[0]
            .element()
            .expect("html should be root element");
        let body = html.children()[1]
            .element()
            .expect("body should be inserted");
        let div = body.children()[0]
            .element()
            .expect("div should be preserved");

        assert_eq!(
            vec![
                "comment:before".to_string(),
                "text:alpha".to_string(),
                "element:span".to_string(),
                "element:table".to_string(),
                "element:p".to_string(),
            ],
            child_kinds(div),
        );

        let table = div.children()[3]
            .element()
            .expect("table should remain after foster-parented siblings");
        assert_eq!(vec!["element:tbody".to_string()], child_kinds(table));

        let tbody = table.children()[0]
            .element()
            .expect("tbody should be inserted by html5ever");
        assert_eq!(vec!["element:tr".to_string()], child_kinds(tbody));

        let paragraph = div.children()[4]
            .element()
            .expect("paragraph after the table should remain last");
        assert_eq!("after", paragraph.children()[0].text().unwrap().text());
    }
}
