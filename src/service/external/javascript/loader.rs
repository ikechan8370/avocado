use std::collections::HashMap;
use std::future::Future;

use crate::bot::friend::Friend;
use crate::bot::group::Group;
use crate::kritor::r#impl::ContactJsObject;
use crate::kritor::server::kritor_proto::common::element::{Data, ElementType};
use crate::kritor::server::kritor_proto::common::*;
use crate::kritor::server::BOTS;
use crate::service::service::KritorContext;
use boa_engine::object::builtins::{JsArray, JsMap, JsRegExp};
use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::value::TryFromJs;
use boa_engine::{js_string, Context, JsError, JsObject, JsResult, JsValue, NativeFunction};
use boa_runtime::Console;
use log::{debug, error, info, warn};

struct Logger {
    pub debug: NativeFunction,
    pub info: NativeFunction,
    pub warn: NativeFunction,
    pub error: NativeFunction,
}

impl Logger {
    pub fn new(plugin_name: String) -> Self {
        Self {
            debug: NativeFunction::from_copy_closure_with_captures(
                |_this, args, name, _ctx| {
                    debug!("[{}]{:?}", name, args);
                    Ok(JsValue::Undefined)
                },
                plugin_name.clone(),
            ),
            info: NativeFunction::from_copy_closure_with_captures(
                |_this, args, name, _ctx| {
                    info!("[{}]{:?}", name, args);
                    Ok(JsValue::Undefined)
                },
                plugin_name.clone(),
            ),
            warn: NativeFunction::from_copy_closure_with_captures(
                |_this, args, name, _ctx| {
                    warn!("[{}]{:?}", name, args);
                    Ok(JsValue::Undefined)
                },
                plugin_name.clone(),
            ),
            error: NativeFunction::from_copy_closure_with_captures(
                |_this, args, name, _ctx| {
                    error!("[{}]{:?}", name, args);
                    Ok(JsValue::Undefined)
                },
                plugin_name.clone(),
            ),
        }
    }
}

pub fn generate_context(
    groups: &Option<HashMap<u64, Group>>,
    fm: &Option<HashMap<u64, Friend>>,
    uin: u64,
    uid: String,
    nickname: String,
    sender: Option<Sender>,
    contact: Option<Contact>,
    elements: Vec<Element>,
    plugin_name: String,
    kritor_context: &KritorContext,
) -> Context {
    let mut context = Context::default();

    // 注入 console
    let console = Console::init(&mut context);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("the console builtin shouldn't exist");

    // 注入 logger
    let logger = Logger::new(plugin_name);
    let object = ObjectInitializer::new(&mut context)
        .function(logger.debug, js_string!("debug"), 1)
        .function(logger.info, js_string!("info"), 1)
        .function(logger.warn, js_string!("warn"), 1)
        .function(logger.error, js_string!("error"), 1)
        .build();
    context
        .register_global_property(js_string!("logger"), object, Attribute::all())
        .unwrap();

    // 注入check正则函数
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
                debug!("regex: {}", regex);
                // 直接传过来的带有js的/和/
                let regex = regex.trim_start_matches('/').trim_end_matches('/');
                let value = args
                    .get(1)
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .to_std_string_escaped();
                let result = regex::Regex::new(regex).unwrap().is_match(value.as_str());
                Ok(JsValue::from(result))
            }),
        )
        .unwrap();
    // 注入 Bot

    // gl gml
    let gl = JsMap::new(&mut context);
    let gml = JsMap::new(&mut context);
    if let Some(&ref gm) = groups.as_ref() {
        for (k, v) in gm.iter() {
            gl.set(
                js_string!(k.to_string()),
                JsObject::from_proto_and_data(None, (*v).clone().inner),
                &mut context,
            )
            .unwrap();
            let ml = JsMap::new(&mut context);
            v.members.iter().for_each(|(k, v)| {
                ml.set(
                    js_string!(k.to_string()),
                    JsObject::from_proto_and_data(None, (*v).clone()),
                    &mut context,
                )
                .unwrap();
            });
            gml.set(js_string!(k.to_string()), ml, &mut context)
                .unwrap();
        }
    }

    let fl = JsMap::new(&mut context);
    if let Some(&ref fm) = fm.as_ref() {
        for (k, v) in fm.iter() {
            fl.set(
                js_string!(k.to_string()),
                JsObject::from_proto_and_data(None, (*v).clone().inner),
                &mut context,
            )
            .unwrap();
        }
    }

    let bot = ObjectInitializer::new(&mut context)
        .property(js_string!("uin"), uin, Attribute::all())
        .property(js_string!("uid"), js_string!(uid.clone()), Attribute::all())
        .property(js_string!("gl"), gl, Attribute::all())
        .property(js_string!("gml"), gml, Attribute::all())
        .property(js_string!("fl"), fl, Attribute::all())
        .property(
            js_string!("nickname"),
            js_string!(nickname),
            Attribute::all(),
        )
        .function(
            NativeFunction::from_async_fn(send_msg),
            js_string!("sendMessage"),
            3,
        )
        .build();

    context
        .register_global_property(js_string!("Bot"), bot.clone(), Attribute::all())
        .unwrap();

    // e
    let msg = elements
        .iter()
        .filter(|e| e.r#type == i32::from(ElementType::Text))
        .map(|e| match e.data.as_ref().unwrap() {
            Data::Text(t) => t.text.clone(),
            _ => "".to_string(),
        })
        .collect::<Vec<String>>()
        .join("");
    let contact_js = contact.map(ContactJsObject::from).unwrap();
    let is_master = kritor_context.is_master;
    let e = ObjectInitializer::new(&mut context)
        .property(js_string!("msg"), js_string!(msg), Attribute::all())
        .property(
            js_string!("sender"),
            JsObject::from_proto_and_data(None, sender),
            Attribute::all(),
        )
        .property(
            js_string!("contact"),
            JsObject::from_proto_and_data(None, contact_js),
            Attribute::all(),
        )
        .property(js_string!("uin"), uin, Attribute::all())
        .property(js_string!("uid"), js_string!(uid.clone()), Attribute::all())
        .property(js_string!("is_master"), is_master, Attribute::all())
        .property(js_string!("bot"), bot, Attribute::all())
        .function(NativeFunction::from_async_fn(reply), js_string!("reply"), 2)
        .build();

    context
        .register_global_property(js_string!("e"), e, Attribute::all())
        .unwrap();

    context
}

fn elements_from_js(value: JsValue, context: &mut Context) -> JsResult<Vec<Element>> {
    if value.is_string() {
        return Ok(vec![Element {
            r#type: i32::from(ElementType::Text),
            data: Some(Data::Text(TextElement {
                text: value.as_string().unwrap().to_std_string_escaped(),
            })),
        }]);
    }
    let obj = value.as_object().unwrap();
    if !obj.is_array() {
        let elem_type = obj
            .get(js_string!("type"), context)
            .unwrap()
            .as_string()
            .unwrap()
            .to_std_string_escaped();
        let data = &obj.get(js_string!("data"), context).unwrap();
        return Ok(vec![Element {
            r#type: ElementType::from_str_name(elem_type.as_str())
                .unwrap()
                .into(),
            data: Some(
                match ElementType::from_str_name(elem_type.as_str()).unwrap() {
                    ElementType::Text => Data::Text(TextElement::try_from_js(data, context)?),
                    ElementType::At => Data::At(AtElement::try_from_js(data, context)?),
                    ElementType::Face => Data::Face(FaceElement::try_from_js(data, context)?),

                    ElementType::BubbleFace => {
                        Data::BubbleFace(BubbleFaceElement::try_from_js(data, context)?)
                    }
                    ElementType::Reply => Data::Reply(ReplyElement::try_from_js(data, context)?),
                    ElementType::Image => Data::Image(ImageElement::try_from_js(data, context)?),
                    // ElementType::Voice => VoiceElement::try_from_js(data, context), // todo
                    // ElementType::Video => VideoElement::try_from_js(data, context), // todo
                    ElementType::Basketball => {
                        Data::Basketball(BasketballElement::try_from_js(data, context)?)
                    }
                    ElementType::Dice => Data::Dice(DiceElement::try_from_js(data, context)?),
                    ElementType::Rps => Data::Rps(RpsElement::try_from_js(data, context)?),
                    ElementType::Poke => Data::Poke(PokeElement::try_from_js(data, context)?),
                    // ElementType::Music => MusicElement::try_from_js(data, context), // todo
                    ElementType::Weather => {
                        Data::Weather(WeatherElement::try_from_js(data, context)?)
                    }
                    // ElementType::Location => Data::Location(LocationElement::try_from_js(data, context)?),
                    ElementType::Share => Data::Share(ShareElement::try_from_js(data, context)?),
                    ElementType::Gift => Data::Gift(GiftElement::try_from_js(data, context)?),
                    ElementType::MarketFace => {
                        Data::MarketFace(MarketFaceElement::try_from_js(data, context)?)
                    }
                    ElementType::Forward => {
                        Data::Forward(ForwardElement::try_from_js(data, context)?)
                    }
                    ElementType::Contact => {
                        Data::Contact(ContactElement::try_from_js(data, context)?)
                    }
                    ElementType::Json => Data::Json(JsonElement::try_from_js(data, context)?),
                    ElementType::Xml => Data::Xml(XmlElement::try_from_js(data, context)?),
                    ElementType::File => Data::File(FileElement::try_from_js(data, context)?),
                    ElementType::Markdown => {
                        Data::Markdown(MarkdownElement::try_from_js(data, context)?)
                    }
                    // ElementType::Keyboard => Data::Keyboard(KeyboardElement::try_from_js(data, context)?),
                    _ => {
                        error!("unexpected type for arg data");
                        Data::Text(TextElement {
                            text: "error: unexpected type for arg data".to_string(),
                        })
                    }
                },
            ),
        }]);
    }

    let mut elements = Vec::new();
    let array = JsArray::from_object(obj.clone()).unwrap();
    for i in 0..array.length(context)? {
        let v = array.get(i, context)?;
        let obj = v.as_object().unwrap();
        let elem_type = obj
            .get(js_string!("type"), context)
            .unwrap()
            .as_string()
            .unwrap()
            .to_std_string_escaped();
        let data = &obj.get(js_string!("data"), context).unwrap();
        let element = Element {
            r#type: ElementType::from_str_name(elem_type.as_str())
                .unwrap()
                .into(),
            data: Some(
                match ElementType::from_str_name(elem_type.as_str()).unwrap() {
                    ElementType::Text => Data::Text(TextElement::try_from_js(data, context)?),
                    ElementType::At => Data::At(AtElement::try_from_js(data, context)?),
                    ElementType::Face => Data::Face(FaceElement::try_from_js(data, context)?),

                    ElementType::BubbleFace => {
                        Data::BubbleFace(BubbleFaceElement::try_from_js(data, context)?)
                    }
                    ElementType::Reply => Data::Reply(ReplyElement::try_from_js(data, context)?),
                    ElementType::Image => Data::Image(ImageElement::try_from_js(data, context)?),
                    // ElementType::Voice => VoiceElement::try_from_js(data, context), // todo
                    // ElementType::Video => VideoElement::try_from_js(data, context), // todo
                    ElementType::Basketball => {
                        Data::Basketball(BasketballElement::try_from_js(data, context)?)
                    }
                    ElementType::Dice => Data::Dice(DiceElement::try_from_js(data, context)?),
                    ElementType::Rps => Data::Rps(RpsElement::try_from_js(data, context)?),
                    ElementType::Poke => Data::Poke(PokeElement::try_from_js(data, context)?),
                    // ElementType::Music => MusicElement::try_from_js(data, context), // todo
                    ElementType::Weather => {
                        Data::Weather(WeatherElement::try_from_js(data, context)?)
                    }
                    // ElementType::Location => Data::Location(LocationElement::try_from_js(data, context)?),
                    ElementType::Share => Data::Share(ShareElement::try_from_js(data, context)?),
                    ElementType::Gift => Data::Gift(GiftElement::try_from_js(data, context)?),
                    ElementType::MarketFace => {
                        Data::MarketFace(MarketFaceElement::try_from_js(data, context)?)
                    }
                    ElementType::Forward => {
                        Data::Forward(ForwardElement::try_from_js(data, context)?)
                    }
                    ElementType::Contact => {
                        Data::Contact(ContactElement::try_from_js(data, context)?)
                    }
                    ElementType::Json => Data::Json(JsonElement::try_from_js(data, context)?),
                    ElementType::Xml => Data::Xml(XmlElement::try_from_js(data, context)?),
                    ElementType::File => Data::File(FileElement::try_from_js(data, context)?),
                    ElementType::Markdown => {
                        Data::Markdown(MarkdownElement::try_from_js(data, context)?)
                    }
                    // ElementType::Keyboard => Data::Keyboard(KeyboardElement::try_from_js(data, context)?),
                    _ => {
                        error!("unexpected type for arg data");
                        Data::Text(TextElement {
                            text: "error: unexpected type for arg data".to_string(),
                        })
                    }
                },
            ),
        };
        elements.push(element);
    }
    Ok(elements)
}

fn send_msg(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> impl Future<Output = JsResult<JsValue>> {
    let msg = args.get(0).unwrap();
    let contact = args.get(1).unwrap();
    let reply = args.get(2).unwrap().as_boolean().unwrap();
    let bot = this.as_object().unwrap();
    let uid = bot
        .get(js_string!("uid"), context)
        .unwrap()
        .as_string()
        .unwrap()
        .to_std_string_escaped();

    // let self_id = args.get(3).unwrap().as_string().unwrap().to_std_string_escaped();
    let elements = elements_from_js(msg.clone(), context).unwrap();
    let contact = Contact::try_from_js(contact, context).unwrap();
    // todo quote
    async move {
        let bots = BOTS.read().await;
        let bot = bots.get(&uid).unwrap();
        let bot_guard = bot.read().await;
        let response = bot_guard.send_msg(elements, contact).await.unwrap();
        Ok(JsObject::from_proto_and_data(None, response).into())
    }
}

fn reply(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> impl Future<Output = JsResult<JsValue>> {
    // 第一个参数是消息
    let msg = args.get(0).unwrap();
    let elements = elements_from_js(msg.clone(), context).unwrap();
    let e = this.as_object().unwrap();
    let contact = e.get(js_string!("contact"), context).unwrap();
    let contact = contact
        .as_object()
        .unwrap()
        .downcast_ref::<ContactJsObject>()
        .unwrap()
        .clone();
    let contact: Contact = contact.into();
    let uid = e
        .get(js_string!("uid"), context)
        .unwrap()
        .as_string()
        .unwrap()
        .to_std_string_escaped();
    // todo quote
    async move {
        let bots = BOTS.read().await;
        let bot = bots.get(&uid).unwrap();
        let bot_guard = bot.read().await;
        let result = bot_guard.send_msg(elements, contact.clone()).await;
        match result {
            Ok(response) => Ok(JsObject::from_proto_and_data(None, response).into()),
            Err(error) => Err(JsError::from_opaque(JsValue::from(js_string!(
                error.to_string()
            )))),
        }
    }
}
