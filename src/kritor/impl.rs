use boa_engine::{Context, js_string, JsError, JsResult, JsValue};
use boa_engine::object::builtins::JsArrayBuffer;
use boa_engine::value::TryFromJs;
use crate::kritor::server::kritor_proto::common::{Contact, image_element, ImageElement, Scene};
use crate::kritor::server::kritor_proto::common::element::Data;

impl TryFromJs for ImageElement {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let object = value.as_object().expect("value is not an object");
        let file_md5 = object.get("file_md5", context)?.as_string().map(|s| s.to_std_string_escaped());
        let sub_type = object.get("sub_type", context)?.as_number().map(|n| n as u32);
        let type_str = object.get("type", context)?.as_string().map(|s| s.to_std_string_escaped());
        let r#type = type_str.map(|s| image_element::ImageType::from_str_name(&s.to_uppercase())).unwrap().map(|s| s.into());
        let data = if let Ok(file) = object.get("file", context).as_ref() {
            let file = JsArrayBuffer::from_object(file.as_object().unwrap().clone())?;
            let file = file.data().expect("file is detached").to_vec();
            Some(image_element::Data::File(file))
        } else if let Ok(url) = object.get("url", context).as_ref() {
            let url = url.as_string().unwrap().to_std_string_escaped();
            Some(image_element::Data::FileUrl(url))
        } else if let Ok(file_path) = object.get("file_path", context).as_ref() {
            let file_path = file_path.as_string().unwrap().to_std_string_escaped();
            Some(image_element::Data::FilePath(file_path))
        } else if let Ok(file_name) = object.get("file_name", context).as_ref() {
            let file_name = file_name.as_string().unwrap().to_std_string_escaped();
            Some(image_element::Data::FileName(file_name))
        } else {
            None
        };
        Ok(Self {
            file_md5,
            sub_type,
            r#type,
            data,
        })
    }
}

impl TryFromJs for Contact {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let object = value.as_object().expect("value is not an object");
        let peer = object.get("peer", context)?.as_string().map(|s| s.to_std_string_escaped()).unwrap();
        let sub_peer = object.get("sub_peer", context)?.as_string().map(|s| s.to_std_string_escaped());

        let scene_str = object.get("scene", context)?.as_string().map(|s| s.to_std_string_escaped()).unwrap();
        let scene = Scene::from_str_name(&scene_str.to_uppercase()).unwrap();
        Ok(Self {
            scene: scene.into(),
            peer,
            sub_peer,
        })
    }
}