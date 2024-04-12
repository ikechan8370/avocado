#[cfg(test)]
mod tests {
    use std::path::Path;
    use boa_engine::{Context, js_string, JsArgs, JsObject, JsString, JsValue, NativeFunction, Source};
    use boa_engine::builtins::promise::PromiseState;
    use boa_engine::job::NativeJob;
    use boa_engine::object::builtins::JsPromise;
    use boa_engine::property::Attribute;
    use crate::utils::time::format_duration;
    use boa_runtime::Console;



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
                js_string!("test"),
                0,
                NativeFunction::from_copy_closure(move |_this, args, _ctx| {
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
        match promise.state() {
            PromiseState::Pending => {}
            PromiseState::Fulfilled(value) => {
                println!("value: {:?}", value);
            }
            PromiseState::Rejected(_) => {}
        }
    }
}
