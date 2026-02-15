# sxd_html

[![crates.io](https://img.shields.io/crates/d/sxd_html)](https://crates.io/crates/sxd_html)
[![tests](https://github.com/kitsuyui/sxd_html/actions/workflows/test.yml/badge.svg)](https://github.com/kitsuyui/sxd_html/actions/workflows/test.yml)
![Coverage](https://raw.githubusercontent.com/kitsuyui/octocov-central/main/badges/kitsuyui/sxd_html/coverage.svg)
[![license](https://img.shields.io/crates/l/sxd_html)](https://github.com/kitsuyui/sxd_html#license)

Uses the `html5ever` to parse html and convert it to a `sxd_document::Package` to use with `sxd_xpath`.
## Example
```rust
use sxd_xpath::{nodeset::Node, Context, Error, Factory, Value};

fn main() -> anyhow::Result<()> {
    let contents = reqwest::blocking::get("https://github.com/trending")?.text()?;
    let package = sxd_html::parse_html(&contents);
    let document = package.as_document();

    let mut trending_repos: Vec<String> = Default::default();
    let repo_as = match evaluate_xpath_node(document.root(), "//article/h1/a")? {
        Value::Nodeset(set) => set,
        _ => panic!("Expected node set"),
    }
    .document_order();

    for repo_a in repo_as {
        let user = evaluate_xpath_node(repo_a, "./span/text()")?.into_string();
        let name = repo_a
            .children()
            .last()
            .unwrap()
            .text()
            .map(|t| t.text())
            .unwrap_or_default();
        trending_repos.push(format!("{}{}", user.trim(), name.trim()));
    }

    println!("Trending Repos :");
    for name in trending_repos {
        println!("\t{}", name);
    }

    Ok(())
}

fn evaluate_xpath_node<'d>(node: impl Into<Node<'d>>, expr: &str) -> Result<Value<'d>, Error> {
    let factory = Factory::new();
    let expression = factory.build(expr)?;
    let expression = expression.ok_or(Error::NoXPath)?;

    let context = Context::new();

    expression
        .evaluate(&context, node.into())
        .map_err(Into::into)
}
```

Output (01/07/2021):
```
Trending Repos :
	GTAmodding /re3
	jina-ai /jina
	JetBrains /kotlin
	freefq /free
	prisma /prisma
	rcmaehl /WhyNotWin11
	bytedance /lightseq
	covidpass-org /covidpass
	pi-apps /pi-platform-docs
	yangshun /front-end-interview-handbook
	amit-davidson /awesome-golang-workshops
	mhadidg /software-architecture-books
	30-seconds /30-seconds-of-code
	GokuMohandas /MadeWithML
	dataease /dataease
	ffffffff0x /Digital-Privacy
	trekhleb /javascript-algorithms
	PaddlePaddle /PaddleX
	SigNoz /signoz
	sudheerj /reactjs-interview-questions
	EdgeSecurityTeam /Vulnerability
	dastergon /awesome-sre
	PowerShell /PowerShell
	CorentinJ /Real-Time-Voice-Cloning
	csseky /cskaoyan
```

## Note

This library uses `html5ever` to parse html. So you need to follow the constraints of `html5ever`.
For example, you need to be careful about how to handle `<table>` tags.

```html
<table>
    <tr>
        <td>1</td>
        <td>2</td>
    </tr>
    <tr>
        <td>3</td>
        <td>4</td>
    </tr>
</table>
```

will be interpreted as

```html
<table>
    <tbody>
        <tr>
            <td>1</td>
            <td>2</td>
        </tr>
        <tr>
            <td>3</td>
            <td>4</td>
        </tr>
    </tbody>
</table>
```

So if you run the query `//table/tr/td`, you will not get the `td` hidden by `tbody`.
You need to run the query `//table/tbody/tr/td`.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
