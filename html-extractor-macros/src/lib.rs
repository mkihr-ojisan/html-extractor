use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Delimiter, TokenStream, TokenTree, TokenTree::*};
use proc_macro_error::*;
use quote::{quote, ToTokens};

#[proc_macro_error]
#[proc_macro]
pub fn html_extractor(input: TokenStream1) -> TokenStream1 {
    let mut input_iter: TokenStreamIter = TokenStream::from(input).into_iter().peekable();

    let mut structs = Vec::new();
    while !input_iter.is_finished() {
        structs.push(Struct::parse(&mut input_iter));
    }

    quote!(#(#structs)*).into()
}

lazy_static::lazy_static! {
    static ref CRATE: String = proc_macro_crate::crate_name("html-extractor").unwrap();
}

type TokenStreamIter = std::iter::Peekable<<TokenStream as IntoIterator>::IntoIter>;
trait TokenStreamIterExt {
    fn is_finished(&mut self) -> bool;
    fn peek_ex(&mut self, expected: &str) -> &TokenTree;
    fn peek_ex_str(&mut self, expected: &str) -> String;
    fn next_ex(&mut self, expected: &str) -> TokenTree;
    fn next_ex_str(&mut self, expected: &str) -> String;
    fn expect(&mut self, expect: &str);
    fn expect_or_none(&mut self, expect: &str);
    fn advance(&mut self, advance: usize);
}
impl TokenStreamIterExt for TokenStreamIter {
    fn is_finished(&mut self) -> bool {
        self.peek().is_none()
    }
    fn peek_ex(&mut self, expected: &str) -> &TokenTree {
        self.peek()
            .unwrap_or_else(|| panic!("expected {}", expected))
    }
    fn peek_ex_str(&mut self, expected: &str) -> String {
        self.peek()
            .unwrap_or_else(|| panic!("expected {}", expected))
            .to_string()
    }
    fn next_ex(&mut self, expected: &str) -> TokenTree {
        self.next()
            .unwrap_or_else(|| panic!("expected {}", expected))
    }
    fn next_ex_str(&mut self, expected: &str) -> String {
        self.next()
            .unwrap_or_else(|| panic!("expected {}", expected))
            .to_string()
    }
    fn expect(&mut self, expect: &str) {
        let next = self
            .next()
            .unwrap_or_else(|| panic!("expected `{}`", expect));
        if next.to_string() != expect {
            abort!(next, "expected `{}`, found `{}`", expect, next);
        }
    }
    fn expect_or_none(&mut self, expect: &str) {
        let next = match self.next() {
            Some(n) => n,
            None => return,
        };
        if next.to_string() != expect {
            abort!(next, "expected `{}`, found `{}`", expect, next);
        }
    }
    fn advance(&mut self, advance: usize) {
        for _ in 0..advance {
            self.next();
        }
    }
}

enum Visibility {
    Private,
    Public,
    PublicIn(TokenStream),
}
impl Visibility {
    fn parse(ts: &mut TokenStreamIter) -> Visibility {
        let iter_advance;
        let vis = match &*ts.peek_ex_str("`pub` or identifier") {
            "pub" => {
                ts.next();
                match ts.peek_ex("`(crate)`, `(super)`, `(in SimplePath)` or identifier") {
                    Group(g) if g.delimiter() == Delimiter::Parenthesis => {
                        iter_advance = 1;
                        Visibility::PublicIn(g.stream())
                    }
                    _ => {
                        iter_advance = 0;
                        Visibility::Public
                    }
                }
            }
            _ => {
                iter_advance = 0;
                Visibility::Private
            }
        };
        ts.advance(iter_advance);
        vis
    }
}
impl ToTokens for Visibility {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Visibility::Private => quote!(),
            Visibility::Public => quote!(pub),
            Visibility::PublicIn(s) => quote!(pub (#s)),
        });
    }
}

struct Attributes {
    tokens: Vec<TokenTree>,
}
impl Attributes {
    fn parse(ts: &mut TokenStreamIter) -> Attributes {
        let mut tokens = Vec::new();
        while ts.peek_ex_str("attribute, visibility or identifier") == "#" {
            tokens.push(ts.next_ex("`#`"));
            tokens.push(ts.next_ex("`[..]`"));
        }
        Attributes { tokens }
    }
}
impl ToTokens for Attributes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.tokens.clone());
    }
}

struct Struct {
    attr: Attributes,
    vis: Visibility,
    name: TokenTree,
    fields: Vec<Field>,
}
impl Struct {
    fn parse(ts: &mut TokenStreamIter) -> Struct {
        let attr = Attributes::parse(ts);
        let vis = Visibility::parse(ts);
        let name = ts.next_ex("identifier");

        let mut fields = Vec::new();
        match ts.next_ex("{{..}}") {
            Group(g) if g.delimiter() == Delimiter::Brace => {
                let mut body_ts = g.stream().into_iter().peekable();
                while !body_ts.is_finished() {
                    fields.push(Field::parse(&mut body_ts));
                    body_ts.expect_or_none(",");
                }
            }
            tt => abort!(tt, "expected {{..}}, found `{}`", tt),
        }

        Struct {
            attr,
            vis,
            name,
            fields,
        }
    }
}
impl ToTokens for Struct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attr = &self.attr;
        let vis = &self.vis;
        let name = &self.name;

        let field_def = self.fields.iter().map(|f| f.def_tokens());
        let field_extract = self.fields.iter().map(|f| f.extract_tokens(&self.name));
        let field_init = self.fields.iter().map(|f| f.init_tokens());

        let _crate = CRATE.parse::<TokenStream>().unwrap();

        tokens.extend(quote!(
            #attr
            #vis struct #name {
                #(#field_def)*
            }
            impl #_crate::HtmlExtractor for #name {
                fn extract(__elem: &#_crate::scraper::ElementRef) -> Result<Self, #_crate::Error> {
                    #(#field_extract)*
                    Ok(Self {
                        #(#field_init)*
                    })
                }
            }
        ));
    }
}

enum Field {
    Single {
        field: SingleField,
        extractor: Extractor,
    },
    Tuple {
        fields: Vec<SingleField>,
        extractor: Extractor,
    },
}
impl Field {
    fn parse(ts: &mut TokenStreamIter) -> Field {
        match ts.peek_ex("(..), visibility or identifier") {
            Group(g) if g.delimiter() == Delimiter::Parenthesis => {
                //Tuple
                let mut fields_ts = g.stream().into_iter().peekable();

                let mut fields = Vec::new();
                while !fields_ts.is_finished() {
                    fields.push(SingleField::parse(&mut fields_ts));
                    fields_ts.expect_or_none(",");
                }
                ts.next();

                ts.expect("=");

                let extractor = Extractor::parse(ts);

                if extractor.capture.is_none() {
                    abort!(
                        fields[0].name,
                        "parsing to tuple fields requires capturing with regex"
                    );
                }

                Field::Tuple { fields, extractor }
            }
            _ => {
                //Single
                let field = SingleField::parse(ts);

                ts.expect("=");

                let extractor = Extractor::parse(ts);

                Field::Single { field, extractor }
            }
        }
    }

    fn def_tokens(&self) -> TokenStream {
        let mut ts = TokenStream::new();
        match self {
            Field::Single { field, .. } => {
                let attr = &field.attr;
                let vis = &field.vis;
                let name = &field.name;
                let ty = &field.ty;
                ts.extend(quote!(
                    #attr
                    #vis #name: #(#ty)*,
                ));
            }
            Field::Tuple { fields, .. } => {
                for field in fields {
                    let attr = &field.attr;
                    let vis = &field.vis;
                    let name = &field.name;
                    let ty = &field.ty;
                    ts.extend(quote!(
                        #attr
                        #vis #name: #(#ty)*,
                    ));
                }
            }
        }
        ts
    }
    fn extract_tokens(&self, struct_name: &TokenTree) -> TokenStream {
        match self {
            Field::Single { field, extractor } => {
                let name = &field.name;
                let extractor_ts = extractor.to_tokens(struct_name, &field.name);
                quote!(
                    let #name = #extractor_ts;
                )
            }
            Field::Tuple { fields, extractor } => {
                let names = fields.iter().map(|f| &f.name);
                let extractor_ts = extractor.to_tokens(struct_name, &fields[0].name);
                quote!(
                    let (#(#names,)*) = #extractor_ts;
                )
            }
        }
    }
    fn init_tokens(&self) -> TokenStream {
        match self {
            Field::Single { field, .. } => {
                let name = &field.name;
                quote!(
                    #name,
                )
            }
            Field::Tuple { fields, .. } => {
                let names = fields.iter().map(|f| &f.name);
                quote!(
                    #(#names,)*
                )
            }
        }
    }
}
struct SingleField {
    attr: Attributes,
    vis: Visibility,
    name: TokenTree,
    ty: Vec<TokenTree>,
}
impl SingleField {
    fn parse(ts: &mut TokenStreamIter) -> Self {
        let attr = Attributes::parse(ts);
        let vis = Visibility::parse(ts);
        let name = ts.next_ex("identifier");

        ts.expect(":");

        let mut ty = Vec::<TokenTree>::new();
        while !ts.is_finished() && {
            let peek = ts.peek_ex_str("`,` or `=`");
            peek != "," && peek != "="
        } {
            ty.push(ts.next_ex(","));
        }

        Self {
            attr,
            vis,
            name,
            ty,
        }
    }
}

struct Extractor {
    target: ExtractTarget,
    capture: Option<TokenTree>,
    collect: bool,
}
impl Extractor {
    fn parse(ts: &mut TokenStreamIter) -> Self {
        let extractor_tt = ts.next_ex("`(..)`");
        let mut extractor_ts: TokenStreamIter = match &extractor_tt {
            Group(g) if g.delimiter() == Delimiter::Parenthesis => {
                g.stream().into_iter().peekable()
            }
            tt => abort!(tt, "expect `(..)`, found `{}`", tt),
        };

        let mut target = None;
        let mut capture = None;
        let mut collect = false;

        while !extractor_ts.is_finished() {
            match &*extractor_ts.next_ex_str("`elem`, `attr`, `text`, `capture` or `collect`") {
                "elem" => {
                    extractor_ts.expect("of");
                    let selector = extractor_ts.next_ex("literal string").clone();
                    target = Some(ExtractTarget::Element(selector));
                }
                "attr" => {
                    let attr = match extractor_ts.next_ex("`[..]`") {
                        Group(g) if g.delimiter() == Delimiter::Bracket => {
                            g.stream().into_iter().peekable().next_ex("literal string")
                        }
                        tt => abort!(tt, "expected `[..]`, found {}", tt),
                    };
                    extractor_ts.expect("of");
                    let selector = extractor_ts.next_ex("literal string").clone();
                    target = Some(ExtractTarget::Attribute(attr, selector));
                }
                "text" => {
                    let nth = match extractor_ts.next_ex("`[..]` or `of`") {
                        Group(g) if g.delimiter() == Delimiter::Bracket => {
                            extractor_ts.expect("of");
                            g.stream()
                        }
                        tt if tt.to_string() == "of" => "0".parse().unwrap(),
                        tt => abort!(tt, "expected `[..]` or `of`, found {}", tt),
                    };

                    let selector = extractor_ts.next_ex("literal string").clone();
                    target = Some(ExtractTarget::TextNode(nth, selector));
                }
                "capture" => {
                    extractor_ts.expect("with");
                    let regex = extractor_ts.next_ex("literal string").clone();
                    capture = Some(regex);
                }
                "collect" => {
                    collect = true;
                }
                tt => abort!(
                    tt,
                    "expected `elem`, `attr`, `text`, `capture` or `collect`, found `{}`",
                    tt
                ),
            }
            extractor_ts.expect_or_none(",");
        }

        let target = match target {
            Some(t) => t,
            None => abort!(extractor_tt, "target is not specified"),
        };

        if let ExtractTarget::Element(_) = &target {
            if capture.is_some() {
                abort!(
                    extractor_tt,
                    "`elem of ..` and `capture with ..` cannot be used for the same field"
                );
            }
        }

        Extractor {
            target,
            capture,
            collect,
        }
    }
    fn to_tokens(&self, struct_name: &TokenTree, field_name: &TokenTree) -> TokenStream {
        let _crate = CRATE.parse::<TokenStream>().unwrap();

        let selector = self.target.selector();
        if let Err(err) = scraper::Selector::parse(&get_literal_str_value(selector)) {
            abort!(selector, "cannot parse the selector: {:?}", err);
        }

        let mut regex_captures_len = None;

        let lazy_static_ts = match &self.capture {
            Some(regex) => {
                match regex::Regex::new(&get_literal_str_value(regex)) {
                    Ok(regex) => regex_captures_len = Some(regex.captures_len()),
                    Err(err) => abort!(regex, "cannot parse the regex: {:?}", err),
                };
                quote! {
                    #_crate::lazy_static::lazy_static! {
                        static ref SELECTOR: #_crate::scraper::Selector = #_crate::scraper::Selector::parse(#selector).unwrap();
                        static ref REGEX: #_crate::regex::Regex = #_crate::regex::Regex::new(#regex).unwrap();
                    }
                }
            }
            None => quote! {
                #_crate::lazy_static::lazy_static! {
                    static ref SELECTOR: #_crate::scraper::Selector = #_crate::scraper::Selector::parse(#selector).unwrap();
                }
            },
        };

        let extract_data_from_elem_ts = match &self.target {
            ExtractTarget::Element(_) => quote! {
                let data = target_elem;
            },
            ExtractTarget::Attribute(attr, _) => quote! {
                let data = target_elem.value().attr(#attr).ok_or(
                    #_crate::error::ErrorKind::InvalidInput(
                        ::std::borrow::Cow::Borrowed(::std::concat!(
                            "extracting the data of field `",
                            ::std::stringify!(#field_name),
                            "` in struct `",
                            ::std::stringify!(#struct_name),
                            "`, attribute `",
                            #attr,
                            "` is not found"
                        ))
                    )
                )?;
            },
            ExtractTarget::TextNode(nth, _) => quote! {
                let data = target_elem.text().nth(#nth).ok_or(
                    #_crate::error::ErrorKind::InvalidInput(
                        ::std::borrow::Cow::Borrowed(::std::concat!(
                            "extracting the data of field `",
                            ::std::stringify!(#field_name),
                            "` in struct `",
                            ::std::stringify!(#struct_name),
                            "`, ",
                            ::std::stringify!(#nth),
                            "th text node is not found"
                        ))
                    )
                )?;
            },
        };

        let parse_data_ts = match &self.capture {
            Some(_) => {
                let mut captures = Vec::new();
                for i in 1..regex_captures_len.unwrap() {
                    captures.push(quote! {
                        caps.get(#i).unwrap().as_str().parse().or_else(|e| Err(
                            #_crate::error::ErrorKind::InvalidInput(
                                ::std::borrow::Cow::Owned(::std::format!(::std::concat!(
                                    "extracting the data of field `",
                                    ::std::stringify!(#field_name),
                                    "` in struct `",
                                    ::std::stringify!(#struct_name),
                                    "`, cannot parse for the ",
                                    ::std::stringify!(#i),
                                    "th field: {}"
                                ), e))
                            )
                        ))?
                    });
                }
                quote! {
                    let caps = REGEX.captures(data).ok_or(
                        #_crate::error::ErrorKind::InvalidInput(
                            ::std::borrow::Cow::Borrowed(::std::concat!(
                                "extracting the data of field `",
                                ::std::stringify!(#field_name),
                                "` in struct `",
                                ::std::stringify!(#struct_name),
                                "`, nothing is captured with regex"
                            ))
                        )
                    )?;
                    (
                        #(#captures,)*
                    )
                }
            }
            None => match &self.target {
                ExtractTarget::Element(_) => quote! {
                    #_crate::HtmlExtractor::extract(&data)?
                },
                _ => quote! {
                    data.parse().or_else(|e| Err(#_crate::error::ErrorKind::InvalidInput(
                            ::std::borrow::Cow::Owned(format!(::std::concat!(
                                "extracting the data of field `",
                                ::std::stringify!(#field_name),
                                "` in struct `",
                                ::std::stringify!(#struct_name),
                                "`, cannot parse `{}`: {}",
                            ), data, e))
                        )
                    ))?
                },
            },
        };

        let extract_data_ts = if self.collect {
            quote! {
                let mut items = Vec::new();
                for target_elem in __elem.select(&*SELECTOR) {
                    let item = {
                        #extract_data_from_elem_ts
                        #parse_data_ts
                    };
                    items.push(item);
                }
                items.into_iter().collect()
            }
        } else {
            quote! {
                let target_elem = __elem.select(&*SELECTOR).next().ok_or(
                    #_crate::error::ErrorKind::InvalidInput(
                        ::std::borrow::Cow::Borrowed(::std::concat!(
                            "extracting the data of field `",
                            ::std::stringify!(#field_name),
                            "` in struct `",
                            ::std::stringify!(#struct_name),
                            "`, no element matched the selector"
                        ))
                    )
                )?;
                #extract_data_from_elem_ts
                #parse_data_ts
            }
        };

        quote! {{
            #lazy_static_ts
            #extract_data_ts
        }}
    }
}
enum ExtractTarget {
    //0 = selector
    Element(TokenTree),
    //0 = attribute, 1 = selector
    Attribute(TokenTree, TokenTree),
    //0 = nth, 1 = selector
    TextNode(TokenStream, TokenTree),
}
impl ExtractTarget {
    fn selector(&self) -> &TokenTree {
        match self {
            ExtractTarget::Element(s) => s,
            ExtractTarget::Attribute(_, s) => s,
            ExtractTarget::TextNode(_, s) => s,
        }
    }
}

fn get_literal_str_value(tt: &TokenTree) -> String {
    let ts = quote!(#tt);
    let lit_str: syn::LitStr =
        syn::parse2(ts).unwrap_or_else(|_| panic!("expected literal string, found `{}`", tt));
    lit_str.value()
}
