use async_trait::async_trait;
use bytes::Bytes;
use prost::Message;
use crate::bot::bot::Bot;
use crate::kritor::server::kritor_proto::*;
use crate::model::error::Result;
#[derive(Debug)]
pub struct Friend {
    pub inner: crate::kritor::server::kritor_proto::FriendInfo,
}

#[async_trait]
pub trait FriendAPITrait {
    async fn get_friend_list(&self, refresh: bool) -> Result<GetFriendListResponse>;
    async fn get_friend_profile_card(&self, uins: Vec<u64>, uids: Vec<String>) -> Result<GetFriendProfileCardResponse>;

    async fn get_stranger_profile_card(&self, uins: Vec<u64>, uids: Vec<String>) -> Result<GetStrangerProfileCardResponse>;

    async fn set_profile_card(&self, nickname: Option<String>, company: Option<String>, email: Option<String>, college: Option<String>, personal_note: Option<String>, birthday: Option<u32>, age: Option<u32>) -> Result<SetProfileCardResponse>;

    async fn is_black_list_user(&self, target: is_black_list_user_request::Target) -> Result<IsBlackListUserResponse>;

    async fn vote_user(&self, target: vote_user_request::Target, vote_count: u32) -> Result<VoteUserResponse>;

    async fn get_uid_by_uin(&self, uins: Vec<u64>) -> Result<GetUidByUinResponse>;

    async fn get_uin_by_uid(&self, uids: Vec<String>) -> Result<GetUinByUidResponse>;
}

#[async_trait]
impl FriendAPITrait for Bot {
    async fn get_friend_list(&self, refresh: bool) -> Result<GetFriendListResponse> {
        let request = GetFriendListRequest {
            refresh: Some(refresh),
        };
        let response = self.send_request(common::Request {
            cmd: "FriendService.GetFriendListRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        let response = GetFriendListResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn get_friend_profile_card(&self, uins: Vec<u64>, uids: Vec<String>) -> Result<GetFriendProfileCardResponse> {
        let request = GetFriendProfileCardRequest {
            target_uins: uins,
            target_uids: uids,
        };
        let response = self.send_request(common::Request {
            cmd: "FriendService.GetFriendProfileCardRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        let response = GetFriendProfileCardResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn get_stranger_profile_card(&self, uins: Vec<u64>, uids: Vec<String>) -> Result<GetStrangerProfileCardResponse> {
        let request = GetStrangerProfileCardRequest {
            target_uins: uins,
            target_uids: uids,
        };
        let response = self.send_request(common::Request {
            cmd: "FriendService.GetStrangerProfileCardRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        let response = GetStrangerProfileCardResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn set_profile_card(&self, nickname: Option<String>, company: Option<String>, email: Option<String>, college: Option<String>, personal_note: Option<String>, birthday: Option<u32>, age: Option<u32>) -> Result<SetProfileCardResponse> {
        let request = SetProfileCardRequest {
            nick_name: nickname,
            company,
            email,
            college,
            personal_note,
            birthday,
            age,
        };
        let response = self.send_request(common::Request {
            cmd: "FriendService.SetProfileCardRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        let response = SetProfileCardResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn is_black_list_user(&self, target: is_black_list_user_request::Target) -> Result<IsBlackListUserResponse> {
        let request = IsBlackListUserRequest {
            target: Some(target),
        };
        let response = self.send_request(common::Request {
            cmd: "FriendService.IsBlackListUserRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        let response = IsBlackListUserResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn vote_user(&self, target: vote_user_request::Target, vote_count: u32) -> Result<VoteUserResponse> {
        let request = VoteUserRequest {
            vote_count,
            target: Some(target)
        };
        let response = self.send_request(common::Request {
            cmd: "FriendService.VoteUserRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        let response = VoteUserResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn get_uid_by_uin(&self, uins: Vec<u64>) -> Result<GetUidByUinResponse> {
        let request = GetUidByUinRequest {
            target_uins: uins,
        };
        let response = self.send_request(common::Request {
            cmd: "FriendService.GetUidByUinRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        let response = GetUidByUinResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn get_uin_by_uid(&self, uids: Vec<String>) -> Result<GetUinByUidResponse> {
        let request = GetUinByUidRequest {
            target_uids: uids,
        };
        let response = self.send_request(common::Request {
            cmd: "FriendService.GetUinByUidRequest".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        let response = GetUinByUidResponse::decode(buf).unwrap();
        Ok(response)
    }
}