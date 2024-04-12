use std::collections::HashMap;
use std::sync::{Arc};
use boa_engine::{Context, js_string, JsError, JsObject, JsResult, JsValue, NativeFunction};
use boa_engine::object::builtins::{JsArray, JsRegExp};
use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::value::TryFromJs;
use boa_runtime::Console;
use log::{debug, error, info, warn};
use tokio::sync::RwLock;
use crate::bot::bot::Bot;
use crate::kritor::server::kritor_proto::{FriendInfo, GroupInfo, GroupMemberInfo};
use boa_gc::{Finalize, GcRefCell, Trace};
use serde_json::Value;
use crate::kritor::server::kritor_proto::common::*;
use crate::kritor::server::kritor_proto::common::element::ElementType;
use crate::service::service::Elements;

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
pub async fn generate_context(bot: Arc<RwLock<Bot>>) -> Context {
    let mut context = Context::default();
    let console = Console::init(&mut context);

    // 注入 console
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("the console builtin shouldn't exist");

    // 注入 logger
    let logger = Logger::new();
    let object = ObjectInitializer::new(&mut context)
        .function(logger.debug, js_string!("debug"), 1)
        .function(logger.info, js_string!("info"), 1)
        .function(logger.warn, js_string!("warn"), 1)
        .function(logger.error, js_string!("error"), 1)
        .build();
    context.register_global_property(js_string!("logger"), object, Attribute::all()).unwrap();

    // 注入check正则函数
    context
        .register_global_callable(
            js_string!("check"),
            2,
            NativeFunction::from_copy_closure(move |_this, args, _ctx| {
                let regex = match args.get(0).unwrap() {
                    JsValue::String(r) => {
                        r.to_std_string_escaped()
                    }
                    JsValue::Object(r) => {
                        let r = JsRegExp::from_object(r.clone()).unwrap();
                        r.to_string(_ctx).unwrap()
                    }
                    _ => {
                        return Err(JsError::from_opaque(JsValue::from(js_string!("unexpected type for arg regex"))));
                    }
                };
                let value = args.get(1).unwrap().as_string().unwrap().to_std_string_escaped();
                let result = regex::Regex::new(regex.as_str()).unwrap().is_match(value.as_str());
                Ok(JsValue::from(result))
            }),
        )
        .unwrap();
    // 注入 Bot
    let bot_guard = bot.read().await;
    // uin
    let uin = bot_guard.get_uin().unwrap_or_default();
    // uid
    let uid = bot_guard.get_uid().unwrap_or_default();
    // gl gml
    let gm = bot_guard.get_groups().read().await;
    let mut gl: HashMap<String, GroupInfo> = HashMap::new();
    let mut gml: HashMap<String, HashMap<String, GroupMemberInfo>> = HashMap::new();
    if let Some(&ref gm) = gm.as_ref() {
        for (k, v) in gm.iter() {
            gl.insert(k.to_string(), (*v).clone());
            gml.insert(k.to_string(), v.members.clone());
        }
    }

    let fm = bot_guard.get_friends().read().await;
    let mut fl: HashMap<String, FriendInfo> = HashMap::new();
    if let Some(&ref fm) = fm.as_ref() {
        for (k, v) in fm.iter() {
            fl.insert(k.to_string(), (*v).clone());
        }
    }

    let bot = ObjectInitializer::new(&mut context)
        .property(js_string!("uin"), uin, Attribute::all())
        .property(js_string!("uid"), uid.as_str(), Attribute::all())
        .property(js_string!("gl"), gl, Attribute::all())
        .property(js_string!("gml"), gml, Attribute::all())
        .property(js_string!("fl"), fl, Attribute::all())
        .function(NativeFunction::from_async_fn(|_this, args, _ctx| {
            let msg = args.get(0).unwrap();

            let contact = args.get(1).unwrap().as_object().unwrap();
            let reply = args.get(2).unwrap().as_boolean().unwrap();
            async move {
                let bot_guard = bot.read().await;
                let elements = elements_from_js(msg.clone(), &mut context).unwrap();
                let contact = Contact::try_from_js(contact, &mut context).unwrap();
                let response = bot_guard.send_msg(elements, contact).await.unwrap();
                // todo how to return custom object?
                Ok(JsValue::Undefined)
            }
        }), js_string!("sendMessage"), 3)
        .build();

    context
        .register_global_property(
            js_string!("Bot"),
            bot,
            Attribute::all()).unwrap();

    context
}


fn elements_from_js(value: JsValue, context: &mut Context) -> JsResult<Vec<Element>> {
    if value.is_string() {
        return vec![Element {
            r#type: i32::from(ElementType::Text),
            data: TextElement {
                text: value.as_string().unwrap().to_string(),
            },
        }];
    }
    let obj = value.as_object().unwrap();
    if !obj.is_array() {
        let elem_type = obj.get(js_string!("type"), context).unwrap().as_string().unwrap().to_std_string_escaped();
        let data = obj.get(js_string!("data"), context).unwrap().as_object().unwrap();
        return vec![Element {
            r#type: ElementType::from_str_name(elem_type.as_str()).unwrap().into(),
            data: match ElementType::from_str_name(elem_type.as_str()).unwrap() {
                ElementType::Text => TextElement::try_from_js(data, context),
                ElementType::At => AtElement::try_from_js(data, context),
                ElementType::Face => FaceElement::try_from_js(data, context),
                ElementType::BubbleFace => BubbleFaceElement::try_from_js(data, context),
                ElementType::Reply => ReplyElement::try_from_js(data, context),
                ElementType::Image => ImageElement::try_from_js(data, context),
                // ElementType::Voice => VoiceElement::try_from_js(data, context), // todo
                // ElementType::Video => VideoElement::try_from_js(data, context), // todo
                ElementType::Basketball => BasketballElement::try_from_js(data, context),
                ElementType::Dice => DiceElement::try_from_js(data, context),
                ElementType::Rps => RpsElement::try_from_js(data, context),
                ElementType::Poke => PokeElement::try_from_js(data, context),
                // ElementType::Music => MusicElement::try_from_js(data, context), // todo
                ElementType::Weather => WeatherElement::try_from_js(data, context),
                ElementType::Location => LocationElement::try_from_js(data, context),
                ElementType::Share => ShareElement::try_from_js(data, context),
                ElementType::Gift => GiftElement::try_from_js(data, context),
                ElementType::MarketFace => MarketFaceElement::try_from_js(data, context),
                ElementType::Forward => ForwardElement::try_from_js(data, context),
                ElementType::Contact => ContactElement::try_from_js(data, context),
                ElementType::Json => JsonElement::try_from_js(data, context),
                ElementType::Xml => XmlElement::try_from_js(data, context),
                ElementType::File => FileElement::try_from_js(data, context),
                ElementType::Markdown => MarkdownElement::try_from_js(data, context),
                ElementType::Keyboard => KeyboardElement::try_from_js(data, context),
                _ => None
            }.unwrap(),
        }];
    }

    let mut elements = Vec::new();
    let array = JsArray::from_object(obj.clone()).unwrap();
    for i in 0..array.length(context)? {
        let v = array.get(i, context)?;
        let obj = v.as_object().unwrap();
        let elem_type = obj.get(js_string!("type"), context).unwrap().as_string().unwrap().to_std_string_escaped();
        let data = obj.get(js_string!("data"), context).unwrap().as_object().unwrap();
        let element = Element {
            r#type: ElementType::from_str_name(elem_type.as_str()).unwrap().into(),
            data: match ElementType::from_str_name(elem_type.as_str()).unwrap() {
                ElementType::Text => TextElement::try_from_js(data, context),
                ElementType::At => AtElement::try_from_js(data, context),
                ElementType::Face => FaceElement::try_from_js(data, context),
                ElementType::BubbleFace => BubbleFaceElement::try_from_js(data, context),
                ElementType::Reply => ReplyElement::try_from_js(data, context),
                ElementType::Image => ImageElement::try_from_js(data, context),
                // ElementType::Voice => VoiceElement::try_from_js(data, context), // todo
                // ElementType::Video => VideoElement::try_from_js(data, context), // todo
                ElementType::Basketball => BasketballElement::try_from_js(data, context),
                ElementType::Dice => DiceElement::try_from_js(data, context),
                ElementType::Rps => RpsElement::try_from_js(data, context),
                ElementType::Poke => PokeElement::try_from_js(data, context),
                // ElementType::Music => MusicElement::try_from_js(data, context), // todo
                ElementType::Weather => WeatherElement::try_from_js(data, context),
                ElementType::Location => LocationElement::try_from_js(data, context),
                ElementType::Share => ShareElement::try_from_js(data, context),
                ElementType::Gift => GiftElement::try_from_js(data, context),
                ElementType::MarketFace => MarketFaceElement::try_from_js(data, context),
                ElementType::Forward => ForwardElement::try_from_js(data, context),
                ElementType::Contact => ContactElement::try_from_js(data, context),
                ElementType::Json => JsonElement::try_from_js(data, context),
                ElementType::Xml => XmlElement::try_from_js(data, context),
                ElementType::File => FileElement::try_from_js(data, context),
                ElementType::Markdown => MarkdownElement::try_from_js(data, context),
                ElementType::Keyboard => KeyboardElement::try_from_js(data, context),
                _ => None
            }.unwrap(),
        };
        elements.push(element);
    }
    Ok(elements)

}
