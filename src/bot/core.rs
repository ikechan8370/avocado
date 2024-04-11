use crate::model::error::Result;

use async_trait::async_trait;
use bytes::Bytes;
use log::error;
use prost::Message;
use crate::bot::bot::Bot;
use crate::kritor::server::kritor_proto::*;
use crate::kritor_err;

#[async_trait]
pub trait CoreAPITrait {
    async fn get_version(&self) -> Result<GetVersionResponse>;
    async fn download_file(&self, url: String) -> Result<DownloadFileResponse>;

    async fn get_current_account(&self) -> Result<GetCurrentAccountResponse>;

    async fn switch_account(&self, account_uid: String, super_ticket: String) -> Result<SwitchAccountResponse>;
}

#[async_trait]
impl CoreAPITrait for Bot {
    async fn get_version(&self) -> Result<GetVersionResponse> {
        let request = GetVersionRequest {};
        let response = self.send_request(common::Request {
            cmd: "CoreService.GetVersion".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        if buf.is_empty() {
            error!("request error: {}", response.msg.unwrap_or("".to_string()));
            return kritor_err!("empty response");
        }
        let response = GetVersionResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn download_file(&self, url: String) -> Result<DownloadFileResponse> {
        let request = DownloadFileRequest {
            url: Some(url),
            base64: None,
            root_path: None,
            file_name: None,
            thread_cnt: None,
            headers: None,
        };
        let response = self.send_request(common::Request {
            cmd: "CoreService.DownloadFile".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        if buf.is_empty() {
            error!("request error: {}", response.msg.unwrap_or("".to_string()));
            return kritor_err!("empty response");
        }
        let response = DownloadFileResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn get_current_account(&self) -> Result<GetCurrentAccountResponse> {
        let request = GetCurrentAccountRequest {};
        let response = self.send_request(common::Request {
            cmd: "CoreService.GetCurrentAccount".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        if buf.is_empty() {
            error!("request error: {}", response.msg.unwrap_or("".to_string()));
            return kritor_err!("empty response");
        }
        let response = GetCurrentAccountResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn switch_account(&self, account_uid: String, super_ticket: String) -> Result<SwitchAccountResponse> {
        let request = SwitchAccountRequest {
            account: Some(switch_account_request::Account::AccountUid(account_uid)),
            super_ticket,
        };
        let response = self.send_request(common::Request {
            cmd: "CoreService.SwitchAccount".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        if buf.is_empty() {
            error!("request error: {}", response.msg.unwrap_or("".to_string()));
            return kritor_err!("empty response");
        }
        let response = SwitchAccountResponse::decode(buf).unwrap();
        Ok(response)
    }
}