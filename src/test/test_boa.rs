#[cfg(test)]
mod tests {
    use crate::utils::time::format_duration;
    use boa_engine::builtins::promise::PromiseState;
    use boa_engine::job::NativeJob;
    use boa_engine::object::builtins::{JsPromise, JsRegExp};
    use boa_engine::object::ObjectInitializer;
    use boa_engine::property::Attribute;
    use boa_engine::{
        js_string, Context, JsArgs, JsError, JsObject, JsString, JsValue, NativeFunction, Source,
    };
    use boa_runtime::Console;
    use log::{debug, error, info, warn};
    use std::path::Path;

    struct Logger {
        pub debug: NativeFunction,
        pub info: NativeFunction,
        pub warn: NativeFunction,
        pub error: NativeFunction,
    }

    impl Logger {
        pub fn new() -> Self {
            Self {
                debug: NativeFunction::from_copy_closure(|_this, args, _ctx| {
                    debug!("{:?}", args);
                    Ok(JsValue::Undefined)
                }),
                info: NativeFunction::from_copy_closure(|_this, args, _ctx| {
                    info!("{:?}", args);
                    Ok(JsValue::Undefined)
                }),
                warn: NativeFunction::from_copy_closure(|_this, args, _ctx| {
                    warn!("{:?}", args);
                    Ok(JsValue::Undefined)
                }),
                error: NativeFunction::from_copy_closure(|_this, args, _ctx| {
                    error!("{:?}", args);
                    Ok(JsValue::Undefined)
                }),
            }
        }
    }

    #[tokio::test]
    async fn test_boa_call_and_bind() {
        fn test(str: String) -> String {
            format!("{}, test", str)
        }

        let mut context = Context::default();
        let console = Console::init(&mut context);
        context
            .register_global_property(js_string!(Console::NAME), console, Attribute::all())
            .expect("the console builtin shouldn't exist");
        context
            .register_global_callable(
                js_string!("check"),
                2,
                NativeFunction::from_copy_closure(move |_this, args, _ctx| {
                    let regex = match args.get(0).unwrap() {
                        JsValue::String(r) => r.to_std_string_escaped(),
                        JsValue::Object(r) => {
                            let r = JsRegExp::from_object(r.clone()).unwrap();
                            r.to_string(_ctx).unwrap()
                        }
                        _ => {
                            return Err(JsError::from_opaque(JsValue::from(js_string!(
                                "unexpected type for arg regex"
                            ))));
                        }
                    };
                    let value = args
                        .get(1)
                        .unwrap()
                        .as_string()
                        .unwrap()
                        .to_std_string_escaped();
                    let result = regex::Regex::new(regex.as_str())
                        .unwrap()
                        .is_match(value.as_str());
                    Ok(JsValue::from(result))
                }),
            )
            .unwrap();

        let logger = Logger::new();
        let object = ObjectInitializer::new(&mut context)
            .function(logger.debug, js_string!("debug"), 1)
            .function(logger.info, js_string!("info"), 1)
            .function(logger.warn, js_string!("warn"), 1)
            .function(logger.error, js_string!("error"), 1)
            .build();

        context
            .register_global_property(js_string!("logger"), object, Attribute::all())
            .unwrap();

        context
            .register_global_callable(
                js_string!("avocado"),
                1,
                NativeFunction::from_copy_closure(move |_this, args, _ctx| {
                    println!("args: {:?}", args);
                    args.get_or_undefined(0).to_string(_ctx).map(|s| {
                        let result = test(s.to_std_string_escaped());
                        JsValue::from(JsString::from(result.as_str()))
                    })
                }),
            )
            .unwrap();
        let source = Source::from_filepath(Path::new("src/test/test_boa.js")).unwrap();
        let value = context.eval(source).unwrap();
        let obj = value.as_object().cloned().unwrap();
        let promise = JsPromise::from_object(obj).unwrap();
        context.run_jobs();
        println!("promise: {:?}", promise.state());
        match promise.state() {
            PromiseState::Pending => {}
            PromiseState::Fulfilled(value) => {
                println!("Fulfilled: {:?}", value);
            }
            PromiseState::Rejected(err) => {
                println!("Rejected: {:?}", value.as_object().cloned().unwrap());
            }
        }
    }
}
