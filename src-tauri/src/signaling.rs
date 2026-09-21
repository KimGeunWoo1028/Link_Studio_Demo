use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Director,
    Camera,
    Program,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CameraSlot {
    Cam1,
    Cam2,
}

impl CameraSlot {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cam1 => "cam1",
            Self::Cam2 => "cam2",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "cam1" => Some(Self::Cam1),
            "cam2" => Some(Self::Cam2),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PgmState {
    pub active_slot: Option<CameraSlot>,
    pub pip_enabled: bool,
    pub pip_slot: Option<CameraSlot>,
    pub caption_text: String,
    pub caption_visible: bool,
}

impl Default for PgmState {
    fn default() -> Self {
        Self {
            active_slot: None,
            pip_enabled: false,
            pip_slot: None,
            caption_text: String::new(),
            caption_visible: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfo {
    pub client_id: String,
    pub role: Role,
    pub slot: Option<CameraSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HubError {
    UnknownSession,
    InvalidSlot,
    SlotTaken { slot: CameraSlot },
    UnknownPeer,
    NotAllowed,
}

impl HubError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownSession => "unknown-session",
            Self::InvalidSlot => "invalid-slot",
            Self::SlotTaken { .. } => "slot-taken",
            Self::UnknownPeer => "unknown-peer",
            Self::NotAllowed => "not-allowed",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::UnknownSession => "Session does not exist.".into(),
            Self::InvalidSlot => "Camera slot must be cam1 or cam2.".into(),
            Self::SlotTaken { slot } => format!("{} is already connected.", slot.as_str()),
            Self::UnknownPeer => "Target peer is not connected.".into(),
            Self::NotAllowed => "This signaling message is not allowed.".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outgoing {
    To { client_id: String, payload: ServerEvent },
    Close { client_id: String, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ServerEvent {
    Registered {
        client_id: String,
        peers: Vec<PeerInfo>,
        pgm: PgmState,
    },
    PeerJoined {
        peer: PeerInfo,
    },
    PeerLeft {
        client_id: String,
        role: Role,
        slot: Option<CameraSlot>,
    },
    MakeOffer {
        to: String,
    },
    Offer {
        from: String,
        sdp: String,
    },
    Answer {
        from: String,
        sdp: String,
    },
    Ice {
        from: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },
    PgmState {
        pgm: PgmState,
    },
    CameraOrientation {
        from: String,
        slot: Option<CameraSlot>,
        orientation: String,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone)]
struct Client {
    session_id: String,
    role: Role,
    slot: Option<CameraSlot>,
}

#[derive(Debug, Default)]
struct Room {
    director: Option<String>,
    cameras: HashMap<CameraSlot, String>,
    programs: HashSet<String>,
    pgm: PgmState,
}

impl Room {
    fn peers(&self) -> Vec<PeerInfo> {
        let mut peers = Vec::new();
        if let Some(id) = &self.director {
            peers.push(PeerInfo {
                client_id: id.clone(),
                role: Role::Director,
                slot: None,
            });
        }
        for (slot, id) in &self.cameras {
            peers.push(PeerInfo {
                client_id: id.clone(),
                role: Role::Camera,
                slot: Some(*slot),
            });
        }
        for id in &self.programs {
            peers.push(PeerInfo {
                client_id: id.clone(),
                role: Role::Program,
                slot: None,
            });
        }
        peers
    }

    fn receivers(&self) -> Vec<String> {
        let mut ids = Vec::new();
        if let Some(id) = &self.director {
            ids.push(id.clone());
        }
        ids.extend(self.programs.iter().cloned());
        ids
    }

    fn remove_client(&mut self, client_id: &str) -> Option<(Role, Option<CameraSlot>)> {
        if self.director.as_deref() == Some(client_id) {
            self.director = None;
            return Some((Role::Director, None));
        }
        let slot = self
            .cameras
            .iter()
            .find_map(|(slot, id)| if id == client_id { Some(*slot) } else { None });
        if let Some(slot) = slot {
            self.cameras.remove(&slot);
            if self.pgm.active_slot == Some(slot) {
                self.pgm.active_slot = self.cameras.keys().next().copied();
            }
            if self.pgm.pip_slot == Some(slot) {
                self.pgm.pip_slot = None;
                self.pgm.pip_enabled = false;
            }
            return Some((Role::Camera, Some(slot)));
        }
        if self.programs.remove(client_id) {
            return Some((Role::Program, None));
        }
        None
    }
}

#[derive(Debug, Default)]
pub struct Hub {
    valid_sessions: HashSet<String>,
    rooms: HashMap<String, Room>,
    clients: HashMap<String, Client>,
}

#[derive(Debug, Clone)]
pub struct RegisterOk {
    pub outgoing: Vec<Outgoing>,
}

impl Hub {
    pub fn open_session(&mut self, session_id: impl Into<String>) {
        self.valid_sessions.insert(session_id.into());
    }

    pub fn session_exists(&self, session_id: &str) -> bool {
        self.valid_sessions.contains(session_id)
    }

    pub fn register(
        &mut self,
        session_id: &str,
        client_id: String,
        role: Role,
        slot: Option<CameraSlot>,
    ) -> Result<RegisterOk, HubError> {
        if !self.valid_sessions.contains(session_id) {
            return Err(HubError::UnknownSession);
        }

        if role == Role::Camera && slot.is_none() {
            return Err(HubError::InvalidSlot);
        }

        let room = self
            .rooms
            .entry(session_id.to_string())
            .or_default();

        match role {
            Role::Camera => {
                let slot = slot.ok_or(HubError::InvalidSlot)?;
                let previous = room.cameras.insert(slot, client_id.clone());
                if room.pgm.active_slot.is_none() {
                    room.pgm.active_slot = Some(slot);
                }
                if let Some(previous) = previous {
                    self.clients.remove(&previous);
                    let mut outgoing = self.finish_register(session_id, client_id, role, Some(slot))?;
                    outgoing.outgoing.insert(
                        0,
                        Outgoing::Close {
                            client_id: previous,
                            reason: "slot-replaced".into(),
                        },
                    );
                    return Ok(outgoing);
                }
            }
            Role::Director => {
                if let Some(previous) = room.director.replace(client_id.clone()) {
                    self.clients.remove(&previous);
                    // previous connection is closed below
                    room.director = Some(client_id.clone());
                    let mut outgoing = self.finish_register(session_id, client_id, role, slot)?;
                    outgoing.outgoing.insert(
                        0,
                        Outgoing::Close {
                            client_id: previous,
                            reason: "director-replaced".into(),
                        },
                    );
                    return Ok(outgoing);
                }
            }
            Role::Program => {
                room.programs.insert(client_id.clone());
            }
        }

        self.finish_register(session_id, client_id, role, slot)
    }

    fn finish_register(
        &mut self,
        session_id: &str,
        client_id: String,
        role: Role,
        slot: Option<CameraSlot>,
    ) -> Result<RegisterOk, HubError> {
        self.clients.insert(
            client_id.clone(),
            Client {
                session_id: session_id.to_string(),
                role,
                slot,
            },
        );

        let room = self.rooms.get(session_id).ok_or(HubError::UnknownSession)?;
        let peers = room.peers();
        let pgm = room.pgm.clone();
        let new_peer = PeerInfo {
            client_id: client_id.clone(),
            role,
            slot,
        };

        let mut outgoing = vec![Outgoing::To {
            client_id: client_id.clone(),
            payload: ServerEvent::Registered {
                client_id: client_id.clone(),
                peers: peers.clone(),
                pgm,
            },
        }];

        for peer in peers.iter().filter(|peer| peer.client_id != client_id) {
            outgoing.push(Outgoing::To {
                client_id: peer.client_id.clone(),
                payload: ServerEvent::PeerJoined {
                    peer: new_peer.clone(),
                },
            });
        }

        if role == Role::Camera {
            for receiver in room.receivers() {
                outgoing.push(Outgoing::To {
                    client_id: client_id.clone(),
                    payload: ServerEvent::MakeOffer { to: receiver },
                });
            }
            for peer in room.peers() {
                outgoing.push(Outgoing::To {
                    client_id: peer.client_id,
                    payload: ServerEvent::PgmState { pgm: room.pgm.clone() },
                });
            }
        } else {
            for camera_id in room.cameras.values() {
                outgoing.push(Outgoing::To {
                    client_id: camera_id.clone(),
                    payload: ServerEvent::MakeOffer {
                        to: client_id.clone(),
                    },
                });
            }
        }

        Ok(RegisterOk { outgoing })
    }

    pub fn disconnect(&mut self, client_id: &str) -> Vec<Outgoing> {
        let Some(client) = self.clients.remove(client_id) else {
            return Vec::new();
        };
        let Some(room) = self.rooms.get_mut(&client.session_id) else {
            return Vec::new();
        };
        let Some((role, slot)) = room.remove_client(client_id) else {
            return Vec::new();
        };
        let pgm = room.pgm.clone();
        let others = room.peers();
        let mut outgoing = Vec::new();
        for peer in others {
            outgoing.push(Outgoing::To {
                client_id: peer.client_id.clone(),
                payload: ServerEvent::PeerLeft {
                    client_id: client_id.to_string(),
                    role,
                    slot,
                },
            });
            outgoing.push(Outgoing::To {
                client_id: peer.client_id,
                payload: ServerEvent::PgmState { pgm: pgm.clone() },
            });
        }
        outgoing
    }

    pub fn set_pgm(&mut self, from: &str, pgm: PgmState) -> Result<Vec<Outgoing>, HubError> {
        let client = self.clients.get(from).ok_or(HubError::UnknownPeer)?;
        if client.role != Role::Director {
            return Err(HubError::NotAllowed);
        }
        let session_id = client.session_id.clone();
        let room = self.rooms.get_mut(&session_id).ok_or(HubError::UnknownSession)?;
        room.pgm = pgm.clone();
        let mut outgoing = Vec::new();
        for peer in room.peers() {
            outgoing.push(Outgoing::To {
                client_id: peer.client_id,
                payload: ServerEvent::PgmState { pgm: pgm.clone() },
            });
        }
        Ok(outgoing)
    }

    pub fn relay_camera_orientation(
        &self,
        from: &str,
        orientation: String,
    ) -> Result<Vec<Outgoing>, HubError> {
        if orientation != "landscape" && orientation != "portrait" {
            return Err(HubError::NotAllowed);
        }
        let client = self.clients.get(from).ok_or(HubError::UnknownPeer)?;
        if client.role != Role::Camera {
            return Err(HubError::NotAllowed);
        }
        let session_id = client.session_id.clone();
        let slot = client.slot;
        let room = self.rooms.get(&session_id).ok_or(HubError::UnknownSession)?;
        let mut outgoing = Vec::new();
        for peer in room.peers() {
            if peer.client_id == from {
                continue;
            }
            if matches!(peer.role, Role::Director | Role::Program) {
                outgoing.push(Outgoing::To {
                    client_id: peer.client_id,
                    payload: ServerEvent::CameraOrientation {
                        from: from.to_string(),
                        slot,
                        orientation: orientation.clone(),
                    },
                });
            }
        }
        Ok(outgoing)
    }

    pub fn relay_offer(&self, from: &str, to: &str, sdp: String) -> Result<Outgoing, HubError> {
        self.ensure_same_session_signal(from, to, true)?;
        Ok(Outgoing::To {
            client_id: to.to_string(),
            payload: ServerEvent::Offer { from: from.to_string(), sdp },
        })
    }

    pub fn relay_answer(&self, from: &str, to: &str, sdp: String) -> Result<Outgoing, HubError> {
        self.ensure_same_session_signal(from, to, false)?;
        Ok(Outgoing::To {
            client_id: to.to_string(),
            payload: ServerEvent::Answer { from: from.to_string(), sdp },
        })
    }

    pub fn relay_ice(
        &self,
        from: &str,
        to: &str,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    ) -> Result<Outgoing, HubError> {
        self.ensure_peer_pair(from, to)?;
        Ok(Outgoing::To {
            client_id: to.to_string(),
            payload: ServerEvent::Ice {
                from: from.to_string(),
                candidate,
                sdp_mid,
                sdp_m_line_index,
            },
        })
    }

    fn ensure_peer_pair(&self, from: &str, to: &str) -> Result<(), HubError> {
        let a = self.clients.get(from).ok_or(HubError::UnknownPeer)?;
        let b = self.clients.get(to).ok_or(HubError::UnknownPeer)?;
        if a.session_id != b.session_id {
            return Err(HubError::NotAllowed);
        }
        Ok(())
    }

    fn ensure_same_session_signal(&self, from: &str, to: &str, offer_from_camera: bool) -> Result<(), HubError> {
        self.ensure_peer_pair(from, to)?;
        let a = &self.clients[from];
        let b = &self.clients[to];
        let camera_to_receiver = a.role == Role::Camera && matches!(b.role, Role::Director | Role::Program);
        let receiver_to_camera = b.role == Role::Camera && matches!(a.role, Role::Director | Role::Program);
        if offer_from_camera && !camera_to_receiver {
            return Err(HubError::NotAllowed);
        }
        if !offer_from_camera && !receiver_to_camera {
            return Err(HubError::NotAllowed);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payloads_for(outgoing: &[Outgoing], client_id: &str) -> Vec<ServerEvent> {
        outgoing
            .iter()
            .filter_map(|item| match item {
                Outgoing::To {
                    client_id: target,
                    payload,
                } if target == client_id => Some(payload.clone()),
                _ => None,
            })
            .collect()
    }

    fn hub_with_session() -> Hub {
        let mut hub = Hub::default();
        hub.open_session("sess1");
        hub
    }

    #[test]
    fn unknown_session_is_rejected() {
        let mut hub = Hub::default();
        let err = hub
            .register("missing", "d1".into(), Role::Director, None)
            .unwrap_err();
        assert_eq!(err, HubError::UnknownSession);
    }

    #[test]
    fn camera_without_slot_is_rejected() {
        let mut hub = hub_with_session();
        let err = hub
            .register("sess1", "c1".into(), Role::Camera, None)
            .unwrap_err();
        assert_eq!(err, HubError::InvalidSlot);
    }

    #[test]
    fn duplicate_camera_slot_replaces_previous() {
        let mut hub = hub_with_session();
        hub.register("sess1", "c1".into(), Role::Camera, Some(CameraSlot::Cam1))
            .unwrap();
        let result = hub
            .register("sess1", "c2".into(), Role::Camera, Some(CameraSlot::Cam1))
            .unwrap();
        assert!(result.outgoing.iter().any(|item| matches!(
            item,
            Outgoing::Close {
                client_id,
                reason
            } if client_id == "c1" && reason == "slot-replaced"
        )));
    }

    #[test]
    fn camera_is_told_to_offer_existing_director() {
        let mut hub = hub_with_session();
        hub.register("sess1", "d1".into(), Role::Director, None)
            .unwrap();
        let result = hub
            .register("sess1", "c1".into(), Role::Camera, Some(CameraSlot::Cam1))
            .unwrap();
        let events = payloads_for(&result.outgoing, "c1");
        assert!(events.iter().any(|event| matches!(event, ServerEvent::MakeOffer { to } if to == "d1")));
        assert!(events.iter().any(|event| matches!(event, ServerEvent::Registered { .. })));
    }

    #[test]
    fn director_join_asks_existing_cameras_to_offer() {
        let mut hub = hub_with_session();
        hub.register("sess1", "c1".into(), Role::Camera, Some(CameraSlot::Cam1))
            .unwrap();
        let result = hub
            .register("sess1", "d1".into(), Role::Director, None)
            .unwrap();
        let events = payloads_for(&result.outgoing, "c1");
        assert!(events.iter().any(|event| matches!(event, ServerEvent::MakeOffer { to } if to == "d1")));
    }

    #[test]
    fn offer_is_relayed_to_director() {
        let mut hub = hub_with_session();
        hub.register("sess1", "d1".into(), Role::Director, None)
            .unwrap();
        hub.register("sess1", "c1".into(), Role::Camera, Some(CameraSlot::Cam1))
            .unwrap();
        let out = hub.relay_offer("c1", "d1", "sdp-offer".into()).unwrap();
        assert_eq!(
            out,
            Outgoing::To {
                client_id: "d1".into(),
                payload: ServerEvent::Offer {
                    from: "c1".into(),
                    sdp: "sdp-offer".into()
                }
            }
        );
    }

    #[test]
    fn offer_camera_to_camera_is_rejected() {
        let mut hub = hub_with_session();
        hub.register("sess1", "c1".into(), Role::Camera, Some(CameraSlot::Cam1))
            .unwrap();
        hub.register("sess1", "c2".into(), Role::Camera, Some(CameraSlot::Cam2))
            .unwrap();
        let err = hub.relay_offer("c1", "c2", "sdp".into()).unwrap_err();
        assert_eq!(err, HubError::NotAllowed);
    }

    #[test]
    fn camera_orientation_is_relayed_to_director() {
        let mut hub = hub_with_session();
        hub.register("sess1", "d1".into(), Role::Director, None)
            .unwrap();
        hub.register("sess1", "c1".into(), Role::Camera, Some(CameraSlot::Cam1))
            .unwrap();
        let outgoing = hub
            .relay_camera_orientation("c1", "landscape".into())
            .unwrap();
        assert_eq!(
            payloads_for(&outgoing, "d1"),
            vec![ServerEvent::CameraOrientation {
                from: "c1".into(),
                slot: Some(CameraSlot::Cam1),
                orientation: "landscape".into(),
            }]
        );
    }

    #[test]
    fn disconnect_notifies_peers() {
        let mut hub = hub_with_session();
        hub.register("sess1", "d1".into(), Role::Director, None)
            .unwrap();
        hub.register("sess1", "c1".into(), Role::Camera, Some(CameraSlot::Cam1))
            .unwrap();
        let outgoing = hub.disconnect("c1");
        let events = payloads_for(&outgoing, "d1");
        assert!(events.iter().any(|event| matches!(
            event,
            ServerEvent::PeerLeft {
                client_id,
                role: Role::Camera,
                slot: Some(CameraSlot::Cam1)
            } if client_id == "c1"
        )));
    }

    #[test]
    fn only_director_can_set_pgm() {
        let mut hub = hub_with_session();
        hub.register("sess1", "c1".into(), Role::Camera, Some(CameraSlot::Cam1))
            .unwrap();
        let err = hub
            .set_pgm(
                "c1",
                PgmState {
                    active_slot: Some(CameraSlot::Cam2),
                    ..PgmState::default()
                },
            )
            .unwrap_err();
        assert_eq!(err, HubError::NotAllowed);
    }

    #[test]
    fn two_cameras_and_program_viewer_get_make_offer() {
        let mut hub = hub_with_session();
        hub.register("sess1", "c1".into(), Role::Camera, Some(CameraSlot::Cam1))
            .unwrap();
        hub.register("sess1", "c2".into(), Role::Camera, Some(CameraSlot::Cam2))
            .unwrap();
        let result = hub
            .register("sess1", "p1".into(), Role::Program, None)
            .unwrap();
        assert!(payloads_for(&result.outgoing, "c1")
            .iter()
            .any(|event| matches!(event, ServerEvent::MakeOffer { to } if to == "p1")));
        assert!(payloads_for(&result.outgoing, "c2")
            .iter()
            .any(|event| matches!(event, ServerEvent::MakeOffer { to } if to == "p1")));
    }

    #[test]
    fn server_event_json_uses_camel_case_fields() {
        let json = serde_json::to_value(&ServerEvent::Registered {
            client_id: "c1".into(),
            peers: vec![],
            pgm: PgmState::default(),
        })
        .unwrap();
        assert_eq!(json["type"], "registered");
        assert_eq!(json["clientId"], "c1");
        assert!(json.get("client_id").is_none());

        let ice = serde_json::to_value(&ServerEvent::Ice {
            from: "c1".into(),
            candidate: "cand".into(),
            sdp_mid: Some("0".into()),
            sdp_m_line_index: Some(0),
        })
        .unwrap();
        assert_eq!(ice["sdpMid"], "0");
        assert_eq!(ice["sdpMLineIndex"], 0);
        assert!(ice.get("sdp_mid").is_none());
    }
}
