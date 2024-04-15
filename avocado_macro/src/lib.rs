use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, ItemStruct, LitStr, Path};

// 定义一个属性宏
#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut name: Option<LitStr> = None;
    let mut events: Vec<Path> = vec![];
    let mut regex: Option<LitStr> = None;

    let tea_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            name = meta.value()?.parse()?;
            Ok(())
        } else if meta.path.is_ident("events") {
            meta.parse_nested_meta(|meta| {
                events.push(meta.path);
                Ok(())
            })
        } else if meta.path.is_ident("pattern") {
            regex = meta.value()?.parse()?;
            Ok(())
        } else {
            Err(meta.error("unsupported tea property"))
        }
    });
    parse_macro_input!(args with tea_parser);

    let input_parsed = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_parsed.ident;

    let name = name.unwrap().value();
    let events_tokens = quote! { vec![#(#events),*] };
    if regex.is_some() {
        let regex = regex.unwrap().value();
        let expanded = quote! {
            #input_parsed

            #[async_trait]
            impl crate::service::service::Matchable for #struct_name {
                fn matches(&self, context: crate::service::service::KritorContext) -> bool {
                    let re = regex::Regex::new(#regex).unwrap();
                    if let Some(message) = context.message {
                        if let Some(elements) = message.elements.get_text_elements() {
                            return elements.iter().any(|ele| re.is_match(ele.text.as_str()));
                        }
                    }
                    false
                }
            }

            use ctor::ctor;

            #[ctor]
            fn auto_init() {
                let service = std::sync::Arc::new(#struct_name::default());
                let events = #events_tokens;
                let name = String::from(#name);
                crate::service::register::register_service(service, events, name);
            }
        };
        return TokenStream::from(expanded);
    }

    let expanded = quote! {
        #input_parsed

        use ctor::ctor;

        #[ctor]
        fn auto_init() {
            let service = std::sync::Arc::new(#struct_name::default());
            let events = #events_tokens;
            let name = String::from(#name);
            crate::service::register::register_service(service, events, name);
        }
    };

    // 返回生成的代码
    TokenStream::from(expanded)
}
