use std::collections::HashMap;
use async_trait::async_trait;
use bytes::Bytes;
use log::error;
use prost::Message;
use crate::bot::bot::Bot;
use crate::kritor::server::kritor_proto::get_group_member_info_request::Target;
use crate::kritor::server::kritor_proto::*;
use crate::kritor_err;

#[derive(Debug, Clone)]
pub struct Group {
    pub inner: GroupInfo,
    pub members: HashMap<u64, GroupMemberInfo>,
}

impl Default for Group {
    fn default() -> Self {
        Self {
            inner: GroupInfo::default(),
            members: HashMap::new(),
        }
    }
}

impl Group {
    pub fn new(inner: GroupInfo, members: HashMap<u64, GroupMemberInfo>) -> Self {
        Self {
            inner,
            members,
        }
    }
}

#[async_trait]
pub trait GroupAPITrait {

    async fn get_group_list(&self, refresh: bool) -> crate::model::error::Result<GetGroupListResponse>;

    async fn get_group_info(&self, group_id: u64) -> crate::model::error::Result<GetGroupInfoResponse>;

    async fn get_group_member_info_by_uin(&self, group_id: u64, member_uin: u64, refresh: bool) -> crate::model::error::Result<GetGroupMemberInfoResponse>;

    async fn get_group_member_info_by_uid(&self, group_id: u64, member_uid: String, refresh: bool) -> crate::model::error::Result<GetGroupMemberInfoResponse>;

    async fn get_group_member_list(&self, group_id: u64, refresh: bool) -> crate::model::error::Result<GetGroupMemberListResponse>;

    async fn ban_member(&self, group_id: u64, target: ban_member_request::Target, duration: u32) -> crate::model::error::Result<BanMemberResponse>;

    async fn poke_member(&self, group_id: u64, target: poke_member_request::Target) -> crate::model::error::Result<PokeMemberResponse>;

    async fn kick_member(&self, group_id: u64, target: kick_member_request::Target, reject_add_request: bool, kick_reason: Option<String>) -> crate::model::error::Result<KickMemberResponse>;

    async fn leave_group(&self, group_id: u64) -> crate::model::error::Result<LeaveGroupResponse>;

    async fn modify_member_card(&self, group_id: u64, target: modify_member_card_request::Target, card: String) -> crate::model::error::Result<ModifyMemberCardResponse>;

    async fn modify_group_name(&self, group_id: u64, group_name: String) -> crate::model::error::Result<ModifyGroupNameResponse>;

    async fn modify_group_remark(&self, group_id: u64, remark: String) -> crate::model::error::Result<ModifyGroupRemarkResponse>;

    async fn set_group_admin(&self, group_id: u64, target: set_group_admin_request::Target, is_admin: bool) -> crate::model::error::Result<SetGroupAdminResponse>;

    async fn set_group_unique_title(&self, group_id: u64, target: set_group_unique_title_request::Target, unique_title: String) -> crate::model::error::Result<SetGroupUniqueTitleResponse>;

    async fn set_group_whole_ban(&self, group_id: u64, is_ban: bool) -> crate::model::error::Result<SetGroupWholeBanResponse>;

    async fn get_prohibited_user_list(&self, group_id: u64) -> crate::model::error::Result<GetProhibitedUserListResponse>;

    async fn get_remain_count_at_all(&self, group_id: u64) -> crate::model::error::Result<GetRemainCountAtAllResponse>;

    async fn get_not_joined_group_info(&self, group_id: u64) -> crate::model::error::Result<GetNotJoinedGroupInfoResponse>;

    async fn get_group_honor_info(&self, group_id: u64, refresh: bool) -> crate::model::error::Result<GetGroupHonorResponse>;


}

#[async_trait]
impl GroupAPITrait for Bot {
    async fn get_group_list(&self, refresh: bool) -> crate::model::error::Result<GetGroupListResponse> {
        let request = GetGroupListRequest {
            refresh: Some(refresh),
        };
        let response = self.send_request(common::Request {
            cmd: "GroupService.GetGroupList".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        if buf.is_empty() {
            error!("request error: {}", response.msg.as_ref().cloned().unwrap_or("".to_string()));
            return kritor_err!("empty response");
        }
        let response = GetGroupListResponse::decode(buf).unwrap();
        Ok(response)
    }

    async fn get_group_info(&self, group_id: u64) -> crate::model::error::Result<GetGroupInfoResponse> {
        let request = GetGroupInfoRequest {
            group_id,
        };
        let request = common::Request {
            cmd: "GroupService.GetGroupInfo".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let group_info = GetGroupInfoResponse::decode(buf).unwrap();
        Ok(group_info)
    }

    async fn get_group_member_info_by_uin(&self, group_id: u64, member_uin: u64, refresh: bool) -> crate::model::error::Result<GetGroupMemberInfoResponse> {
        let request = GetGroupMemberInfoRequest {
            group_id,
            refresh: Some(refresh),
            target: Some(Target::TargetUin(member_uin)),
        };
        let request = common::Request {
            cmd: "GroupService.GetGroupMemberInfo".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let group_member_info = GetGroupMemberInfoResponse::decode(buf).unwrap();
        Ok(group_member_info)
    }

    async fn get_group_member_info_by_uid(&self, group_id: u64, member_uid: String, refresh: bool) -> crate::model::error::Result<GetGroupMemberInfoResponse> {
        let request = GetGroupMemberInfoRequest {
            group_id,
            refresh: Some(refresh),
            target: Some(Target::TargetUid(member_uid)),
        };
        let request = common::Request {
            cmd: "GroupService.GetGroupMemberInfo".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let group_member_info = GetGroupMemberInfoResponse::decode(buf).unwrap();
        Ok(group_member_info)
    }

    async fn get_group_member_list(&self, group_id: u64, refresh: bool) -> crate::model::error::Result<GetGroupMemberListResponse> {
        let request = GetGroupMemberListRequest {
            group_id,
            refresh: Some(refresh),
        };
        let request = common::Request {
            cmd: "GroupService.GetGroupMemberList".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let group_member_list = GetGroupMemberListResponse::decode(buf).unwrap();
        Ok(group_member_list)
    }

    async fn ban_member(&self, group_id: u64, target: ban_member_request::Target, duration: u32) -> crate::model::error::Result<BanMemberResponse> {
        let request = BanMemberRequest {
            group_id,
            target: Some(target),
            duration,
        };
        let request = common::Request {
            cmd: "GroupService.BanMember".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let ban_member_response = BanMemberResponse::decode(buf).unwrap();
        Ok(ban_member_response)
    }

    async fn poke_member(&self, group_id: u64, target: poke_member_request::Target) -> crate::model::error::Result<PokeMemberResponse> {
        let request = PokeMemberRequest {
            group_id,
            target: Some(target),
        };
        let request = common::Request {
            cmd: "GroupService.PokeMember".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let poke_member_response = PokeMemberResponse::decode(buf).unwrap();
        Ok(poke_member_response)
    }

    async fn kick_member(&self, group_id: u64, target: kick_member_request::Target, reject_add_request: bool, kick_reason: Option<String>) -> crate::model::error::Result<KickMemberResponse> {
        let request = KickMemberRequest {
            group_id,
            target: Some(target),
            reject_add_request: Some(reject_add_request),
            kick_reason,
        };
        let request = common::Request {
            cmd: "GroupService.KickMember".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let kick_member_response = KickMemberResponse::decode(buf).unwrap();
        Ok(kick_member_response)
    }

    async fn leave_group(&self, group_id: u64) -> crate::model::error::Result<LeaveGroupResponse> {
        let request = LeaveGroupRequest {
            group_id,
        };
        let request = common::Request {
            cmd: "GroupService.LeaveGroup".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let leave_group_response = LeaveGroupResponse::decode(buf).unwrap();
        Ok(leave_group_response)
    }

    async fn modify_member_card(&self, group_id: u64, target: modify_member_card_request::Target, card: String) -> crate::model::error::Result<ModifyMemberCardResponse> {
        let request = ModifyMemberCardRequest {
            group_id,
            target: Some(target),
            card,
        };
        let request = common::Request {
            cmd: "GroupService.ModifyMemberCard".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let modify_member_card_response = ModifyMemberCardResponse::decode(buf).unwrap();
        Ok(modify_member_card_response)
    }

    async fn modify_group_name(&self, group_id: u64, group_name: String) -> crate::model::error::Result<ModifyGroupNameResponse> {
        let request = ModifyGroupNameRequest {
            group_id,
            group_name,
        };
        let request = common::Request {
            cmd: "GroupService.ModifyGroupName".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let modify_group_name_response = ModifyGroupNameResponse::decode(buf).unwrap();
        Ok(modify_group_name_response)
    }

    async fn modify_group_remark(&self, group_id: u64, remark: String) -> crate::model::error::Result<ModifyGroupRemarkResponse> {
        let request = ModifyGroupRemarkRequest {
            group_id,
            remark,
        };
        let request = common::Request {
            cmd: "GroupService.ModifyGroupRemark".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let modify_group_remark_response = ModifyGroupRemarkResponse::decode(buf).unwrap();
        Ok(modify_group_remark_response)
    }

    async fn set_group_admin(&self, group_id: u64, target: set_group_admin_request::Target, is_admin: bool) -> crate::model::error::Result<SetGroupAdminResponse> {
        let request = SetGroupAdminRequest {
            group_id,
            target: Some(target),
            is_admin,
        };
        let request = common::Request {
            cmd: "GroupService.SetGroupAdmin".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let set_group_admin_response = SetGroupAdminResponse::decode(buf).unwrap();
        Ok(set_group_admin_response)
    }

    async fn set_group_unique_title(&self, group_id: u64, target: set_group_unique_title_request::Target, unique_title: String) -> crate::model::error::Result<SetGroupUniqueTitleResponse> {
        let request = SetGroupUniqueTitleRequest {
            group_id,
            target: Some(target),
            unique_title,
        };
        let request = common::Request {
            cmd: "GroupService.SetGroupUniqueTitle".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let set_group_unique_title_response = SetGroupUniqueTitleResponse::decode(buf).unwrap();
        Ok(set_group_unique_title_response)
    }

    async fn set_group_whole_ban(&self, group_id: u64, is_ban: bool) -> crate::model::error::Result<SetGroupWholeBanResponse> {
        let request = SetGroupWholeBanRequest {
            group_id,
            is_ban,
        };
        let request = common::Request {
            cmd: "GroupService.SetGroupWholeBan".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let set_group_whole_ban_response = SetGroupWholeBanResponse::decode(buf).unwrap();
        Ok(set_group_whole_ban_response)
    }

    async fn get_prohibited_user_list(&self, group_id: u64) -> crate::model::error::Result<GetProhibitedUserListResponse> {
        let request = GetProhibitedUserListRequest {
            group_id,
        };
        let request = common::Request {
            cmd: "GroupService.GetProhibitedUserList".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let get_prohibited_user_list_response = GetProhibitedUserListResponse::decode(buf).unwrap();
        Ok(get_prohibited_user_list_response)
    }

    async fn get_remain_count_at_all(&self, group_id: u64) -> crate::model::error::Result<GetRemainCountAtAllResponse> {
        let request = GetRemainCountAtAllRequest {
            group_id,
        };
        let request = common::Request {
            cmd: "GroupService.GetRemainCountAtAll".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let get_remain_count_at_all_response = GetRemainCountAtAllResponse::decode(buf).unwrap();
        Ok(get_remain_count_at_all_response)
    }

    async fn get_not_joined_group_info(&self, group_id: u64) -> crate::model::error::Result<GetNotJoinedGroupInfoResponse> {
        let request = GetNotJoinedGroupInfoRequest {
            group_id,
        };
        let request = common::Request {
            cmd: "GroupService.GetNotJoinedGroupInfo".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let get_not_joined_group_info_response = GetNotJoinedGroupInfoResponse::decode(buf).unwrap();
        Ok(get_not_joined_group_info_response)
    }

    async fn get_group_honor_info(&self, group_id: u64, refresh: bool) -> crate::model::error::Result<GetGroupHonorResponse> {
        let request = GetGroupHonorRequest {
            group_id,
            refresh: Some(refresh),
        };
        let request = common::Request {
            cmd: "GroupService.GetGroupHonorInfo".to_string(),
            seq: crate::bot::bot::get_seq(),
            buf: request.encode_to_vec(),
            no_response: false,
        };
        let response = self.send_request(request).await.expect("send error");
        let buf: Bytes = response.buf.clone().into();
        let get_group_honor_response = GetGroupHonorResponse::decode(buf).unwrap();
        Ok(get_group_honor_response)
    }
}
