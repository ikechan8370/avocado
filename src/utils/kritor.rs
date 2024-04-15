use crate::kritor::server::kritor_proto::common::{Contact, Sender};

pub fn same_contact_and_sender(cs1: (&Contact, &Sender), cs2: (&Contact, &Sender)) -> bool {
    if cs1.0.scene != cs2.0.scene {
        return false;
    }
    if cs1.0.peer != cs2.0.peer {
        return false;
    }
    if cs1.0.sub_peer.as_ref().unwrap_or(&String::default())
        != cs2.0.sub_peer.as_ref().unwrap_or(&String::default())
    {
        return false;
    }
    if cs1.1.uid != cs2.1.uid {
        return false;
    }
    if cs1.1.uin.unwrap_or_default() != cs2.1.uin.unwrap_or_default() {
        return false;
    }
    true
}
