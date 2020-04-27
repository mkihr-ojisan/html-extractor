# html-extractor

A Rust crate for extracting data from HTML.

[crates.io](https://crates.io/crates/html-extractor)  
[github](https://github.com/mkihr-ojisan/html-extractor)  
[documentation](https://docs.rs/html-extractor/0.1.0/html_extractor/)

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
