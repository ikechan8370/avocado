use proc_macro::TokenStream;

use quote::quote;
use syn::{ItemStruct, LitStr, parse_macro_input, Path};

// 定义一个属性宏
#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut name: Option<LitStr> = None;
    let mut events: Vec<Path> = vec![];

    let tea_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            name = meta.value()?.parse()?;
            Ok(())
        } else if meta.path.is_ident("events") {
            meta.parse_nested_meta(|meta| {
                events.push(meta.path);
                Ok(())
            })
        } else {
            Err(meta.error("unsupported tea property"))
        }
    });
    parse_macro_input!(args with tea_parser);

    let input_parsed = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_parsed.ident;

    let name = name.unwrap().value();
    let events_tokens = quote! { vec![#(#events),*] };
    let expanded  = quote! {
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
