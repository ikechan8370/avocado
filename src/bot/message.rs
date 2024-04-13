use async_trait::async_trait;
use bytes::Bytes;
use prost::Message;
use crate::bot::bot::Bot;
use crate::kritor::server::kritor_proto::*;
use crate::kritor::server::kritor_proto::common::{Contact, Element, ForwardMessageBody};
use crate::model::error::Result;

#[async_trait]
pub trait MessageAPITrait {

    async fn send_msg(&self, segments: Vec<Element>, contact: Contact) -> Result<SendMessageResponse>;

    async fn send_msg_by_res_id(&self, res_id: String, contact: Contact) -> Result<SendMessageByResIdResponse>;

    async fn set_message_readed(&self, contact: Contact) -> Result<SetMessageReadResponse>;

    async fn recall_message(&self, contact: Contact, message_id: String) -> Result<RecallMessageResponse>;

    async fn react_message_with_emoji(&self, contact: Contact, message_id: String, face_id: u32, is_set: bool) -> Result<ReactMessageWithEmojiResponse>;

    async fn get_message(&self, contact: Contact, message_id: String) -> Result<GetMessageResponse>;

    async fn get_message_by_seq(&self, contact: Contact, message_seq: u64) -> Result<GetMessageBySeqResponse>;

    async fn get_history_message(&self, contact: Contact, start_message_id: Option<String>, count: Option<u32>) -> Result<GetHistoryMessageResponse>;

    async fn get_history_message_by_seq(&self, contact: Contact, start_message_seq: Option<u64>, count: Option<u32>) -> Result<GetHistoryMessageBySeqResponse>;

    async fn upload_forward_message(&self, contact: Contact, messages: Vec<ForwardMessageBody>) -> Result<UploadForwardMessageResponse>;

    async fn download_forward_message(&self, res_id: String) -> Result<DownloadForwardMessageResponse>;

    async fn get_essence_message_list(&self, group_id: u64, page: u32, page_size: u32) -> Result<GetEssenceMessageListResponse>;

    async fn set_essence_message(&self, group_id: u64, message_id: String) -> Result<SetEssenceMessageResponse>;

    async fn delete_essence_message(&self, group_id: u64, message_id: String) -> Result<DeleteEssenceMessageResponse>;

}

#[async_trait]
impl MessageAPITrait for Bot {
    async fn send_msg(&self, segments: Vec<Element>, contact: Contact) -> Result<SendMessageResponse> {
        let request = SendMessageRequest {
            contact: Some(contact),
            elements: segments,
            retry_count: None,
            message_id: None,
            notice_id: None,
            request_id: None,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.SendMessageRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = SendMessageResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn send_msg_by_res_id(&self, res_id: String, contact: Contact) -> Result<SendMessageByResIdResponse> {
        let request = SendMessageByResIdRequest {
            contact: Some(contact),
            res_id: res_id,
            retry_count: None,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.SendMessageByResIdRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = SendMessageByResIdResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn set_message_readed(&self, contact: Contact) -> Result<SetMessageReadResponse> {
        let request = SetMessageReadRequest {
            contact: Some(contact),
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.SetMessageReadRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = SetMessageReadResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn recall_message(&self, contact: Contact, message_id: String) -> Result<RecallMessageResponse> {
        let request = RecallMessageRequest {
            contact: Some(contact),
            message_id: message_id,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.RecallMessageRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = RecallMessageResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn react_message_with_emoji(&self, contact: Contact, message_id: String, face_id: u32, is_set: bool) -> Result<ReactMessageWithEmojiResponse> {
        let request = ReactMessageWithEmojiRequest {
            contact: Some(contact),
            message_id: message_id,
            face_id: face_id,
            is_set: is_set,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.ReactMessageWithEmojiRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = ReactMessageWithEmojiResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn get_message(&self, contact: Contact, message_id: String) -> Result<GetMessageResponse> {
        let request = GetMessageRequest {
            contact: Some(contact),
            message_id: message_id,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.GetMessageRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = GetMessageResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn get_message_by_seq(&self, contact: Contact, message_seq: u64) -> Result<GetMessageBySeqResponse> {
        let request = GetMessageBySeqRequest {
            contact: Some(contact),
            message_seq: message_seq,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.GetMessageBySeqRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = GetMessageBySeqResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn get_history_message(&self, contact: Contact, start_message_id: Option<String>, count: Option<u32>) -> Result<GetHistoryMessageResponse> {
        let request = GetHistoryMessageRequest {
            contact: Some(contact),
            start_message_id: start_message_id,
            count: count,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.GetHistoryMessageRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = GetHistoryMessageResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn get_history_message_by_seq(&self, contact: Contact, start_message_seq: Option<u64>, count: Option<u32>) -> Result<GetHistoryMessageBySeqResponse> {
        let request = GetHistoryMessageBySeqRequest {
            contact: Some(contact),
            start_message_seq: start_message_seq,
            count: count,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.GetHistoryMessageBySeqRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = GetHistoryMessageBySeqResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn upload_forward_message(&self, contact: Contact, messages: Vec<ForwardMessageBody>) -> Result<UploadForwardMessageResponse> {
        let request = UploadForwardMessageRequest {
            contact: Some(contact),
            messages: messages,
            retry_count: None,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.UploadForwardMessageRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = UploadForwardMessageResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn download_forward_message(&self, res_id: String) -> Result<DownloadForwardMessageResponse> {
        let request = DownloadForwardMessageRequest {
            res_id: res_id,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.DownloadForwardMessageRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = DownloadForwardMessageResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn get_essence_message_list(&self, group_id: u64, page: u32, page_size: u32) -> Result<GetEssenceMessageListResponse> {
        let request = GetEssenceMessageListRequest {
            group_id: group_id,
            page: page,
            page_size: page_size,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.GetEssenceMessageListRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = GetEssenceMessageListResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn set_essence_message(&self, group_id: u64, message_id: String) -> Result<SetEssenceMessageResponse> {
        let request = SetEssenceMessageRequest {
            group_id: group_id,
            message_id: message_id,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.SetEssenceMessageRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = SetEssenceMessageResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn delete_essence_message(&self, group_id: u64, message_id: String) -> Result<DeleteEssenceMessageResponse> {
        let request = DeleteEssenceMessageRequest {
            group_id: group_id,
            message_id: message_id,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.DeleteEssenceMessageRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let response = DeleteEssenceMessageResponse::decode(buf).unwrap();
        Ok(response)
    }

}