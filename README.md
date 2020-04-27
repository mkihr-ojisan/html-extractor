# html-extractor

![Rust](https://github.com/mkihr-ojisan/html-extractor/workflows/Rust/badge.svg)
[![html-extractor at crates.io](https://img.shields.io/crates/v/html-extractor.svg)](https://crates.io/crates/html-extractor)
[![html-extractor at docs.rs](https://docs.rs/html-extractor/badge.svg)](https://docs.rs/html-extractor)

A Rust crate for extracting data from HTML.

## Examples

### Extracting a simple value from HTML

```rust
use html_extractor::{html_extractor, HtmlExtractor};
html_extractor! {
    #[derive(Debug, PartialEq)]
    Foo {
        foo: usize = (text of "#foo"),
    }
}

fn main() {
    let input = r#"
        <div id="foo">1</div>
    "#;
    let foo = Foo::extract_from_str(input).unwrap();
    assert_eq!(foo, Foo { foo: 1 });
}
```

### Extracting a collection from HTML

```rust
use html_extractor::{html_extractor, HtmlExtractor};
html_extractor! {
    #[derive(Debug, PartialEq)]
    Foo {
        foo: Vec<usize> = (text of ".foo", collect),
    }
}

fn main() {
    let input = r#"
        <div class="foo">1</div>
        <div class="foo">2</div>
        <div class="foo">3</div>
        <div class="foo">4</div>
    "#;
    let foo = Foo::extract_from_str(input).unwrap();
    assert_eq!(foo, Foo { foo: vec![1, 2, 3, 4] });
}
```

### Extracting with regex

```rust
use html_extractor::{html_extractor, HtmlExtractor};
html_extractor! {
    #[derive(Debug, PartialEq)]
    Foo {
        (foo: usize,) = (text of "#foo", capture with "^foo=(.*)$"),
    }
}

fn main() {
    let input = r#"
        <div id="foo">foo=1</div>
    "#;
    let foo = Foo::extract_from_str(input).unwrap();
    assert_eq!(foo, Foo { foo: 1 });
}
```

## Changelog

### v0.1.1

- Fix the links in the documentation
