use crate::mvd::MvdTarget;
use crate::protocol::types::*;
#[cfg(feature = "ascii_strings")]
use crate::utils::ascii_converter::AsciiConverter;
use crate::utils::userinfo::Userinfo;
use serde::Serialize;
use std::collections::HashMap;

pub mod interpolated;
pub use interpolated::InterpolatedState;

pub type Stat = [i32; 32];

static mut PACKET_ENT_COUNT: u32 = 0;

#[derive(Serialize, Clone, Debug, Default)]
pub struct Player {
    pub frags: i16,
    pub ping: u16,
    pub pl: u8,
    pub entertime: f32,
    pub uid: u32,
    pub userinfo: Userinfo,
    pub name: StringByte,
    pub team: StringByte,
    pub spectator: bool,
    pub top_color: StringByte,
    pub bottom_color: StringByte,
    pub origin: CoordinateVector,
    pub angle: AngleVector,
    pub model: u8,
    pub skinnum: u8,
    pub effects: u8,
    pub weaponframe: u8,
    pub stats: Stat,
}

impl Player {
    /// updates the userinfo of this [`Player`].
    #[cfg(feature = "ascii_strings")]
    fn update_userinfo(&mut self) {
        for (k, v) in self.userinfo.values.iter() {
            if k.string == "team" {
                self.team = v.clone();
            }
            if k.string == "name" {
                self.name = v.clone();
            }
        }
    }

    /// the userinfo isnt read into the [`Player`] when ascii_strings is disabled.
    #[cfg(not(feature = "ascii_strings"))]
    fn update_userinfo(&mut self) {}
}

#[derive(Serialize, Copy, Clone, Debug, Default)]
pub struct Entity {
    pub index: u16,
    pub model: u16,
    pub frame: u8,
    pub colormap: u8,
    pub skinnum: u8,
    pub effects: u8,
    pub origin: CoordinateVector,
    pub angle: AngleVector,
}

impl Entity {
    pub fn reverse_delta(&mut self, delta: &Packetentity) -> Packetentity {
        let mut p = Packetentity::default();

        if let Some(v) = delta.model {
            p.model = Some(self.model.clone());
        }
        if let Some(v) = delta.frame {
            p.frame = Some(self.frame.clone());
        }
        if let Some(v) = delta.colormap {
            p.colormap = Some(self.colormap.clone());
        }
        if let Some(v) = delta.skin {
            p.skin = Some(self.skinnum.clone());
        }
        if let Some(v) = delta.effects {
            p.effects = Some(self.effects.clone());
        }
        if let Some(v) = delta.origin {
            let mut r = CoordinateVectorOption::default();
            if v.x.is_some() {
                r.x = Some(self.origin.x.clone());
            }
            if v.y.is_some() {
                r.y = Some(self.origin.y.clone());
            }
            if v.z.is_some() {
                r.z = Some(self.origin.z.clone());
            }
            p.origin = Some(r);
        }
        if let Some(v) = delta.angle {
            let mut r = AngleVectorOption::default();
            if v.x.is_some() {
                r.x = Some(self.angle.x.clone());
            }
            if v.y.is_some() {
                r.y = Some(self.angle.y.clone());
            }
            if v.z.is_some() {
                r.z = Some(self.angle.z.clone());
            }
            p.angle = Some(r);
        }
        p
    }
    /// apply [`ServerMessage::Packetentity`] as a deltapacket_entity to this [`Entity`]
    pub fn apply_delta(&mut self, delta: &Packetentity) {
        if let Some(v) = delta.model {
            self.model = v;
        }
        if let Some(v) = delta.frame {
            self.frame = v;
        }
        if let Some(v) = delta.colormap {
            self.frame = v;
        }
        if let Some(v) = delta.skin {
            self.frame = v;
        }
        if let Some(v) = delta.effects {
            self.frame = v;
        }
        if let Some(v) = delta.origin {
            v.apply_to(&mut self.origin);
        }
        if let Some(v) = delta.angle {
            v.apply_to(&mut self.angle);
        }
    }

    /// create [`Entity`] from [`ServerMessage::Spawnbaseline`]
    pub fn from_baseline(baseline: &Spawnbaseline) -> Entity {
        Entity {
            model: baseline.model_index as u16,
            frame: baseline.model_frame,
            colormap: baseline.colormap,
            skinnum: baseline.skinnum,
            origin: baseline.origin,
            angle: baseline.angle,
            ..Default::default()
        }
    }

    /// create [`Entity`] from [`ServerMessage::Spawnstatic`]
    pub fn from_static(static_ent: &Spawnstatic) -> Entity {
        Entity {
            index: static_ent.model_index as u16,
            frame: static_ent.model_frame,
            colormap: static_ent.colormap,
            skinnum: static_ent.skinnum,
            origin: static_ent.origin,
            angle: static_ent.angle,
            ..Default::default()
        }
    }

    /// create [`Entity`] from [`ServerMessage::Packetentity`]
    pub fn from_packetentity(packet_entity: &Packetentity) -> Entity {
        let mut angle = AngleVector {
            ..Default::default()
        };
        if let Some(pe_angle) = packet_entity.angle {
            pe_angle.apply_to(&mut angle);
        }
        let mut origin = CoordinateVector {
            ..Default::default()
        };
        if let Some(pe_o) = packet_entity.origin {
            pe_o.apply_to(&mut origin);
        }
        Entity {
            index: packet_entity.entity_index,
            frame: packet_entity.frame.unwrap_or(0),
            model: packet_entity.model.unwrap_or(0),
            colormap: packet_entity.colormap.unwrap_or(0),
            skinnum: packet_entity.skin.unwrap_or(0),
            effects: packet_entity.effects.unwrap_or(0),
            origin,
            angle,
        }
    }
}

/*
#[derive(Serialize, Copy, Clone, Debug)]
pub struct Sound {
    pub channel: u16,
    pub entity: u16,
    pub index: u8,
    pub volume: u8,
    pub attenuation: u8,
    pub origin: CoordinateVector
}

impl Sound {
    pub fn from_static(static_sound: &Spawnstaticsound) -> Sound {
        return Sound {
        }
    }
}
*/

#[derive(Serialize, Clone, Default, Debug)]
pub struct State {
    #[cfg(feature = "ascii_strings")]
    ascii_converter: AsciiConverter,
    pub serverdata: Serverdata,
    pub players: HashMap<u16, Player>,
    pub sounds: Vec<StringByte>,
    pub models: Vec<StringByte>,
    pub baseline_entities: HashMap<u16, Entity>,
    pub static_entities: Vec<Spawnstatic>,
    pub entities: HashMap<u16, Entity>,
    pub temp_entities: HashMap<u16, Tempentity>,
    pub static_sounds: Vec<Spawnstaticsound>,
}

impl State {
    pub fn new() -> State {
        State {
            models: vec![StringByte {
                bytes: vec![0],
                string: "".to_string(),
            }],
            ..Default::default()
        }
    }

    #[cfg(feature = "ascii_strings")]
    pub fn new_with_ascii_conveter(ascii_converter: AsciiConverter) -> State {
        State {
            models: vec![StringByte {
                bytes: vec![0],
                string: "".to_string(),
            }],
            ascii_converter,
            ..Default::default()
        }
    }

    fn update_player(&mut self, player_index: u16, message: &ServerMessage) {
        let p = self.players.get_mut(&player_index);
        let player = match p {
            Some(player) => player,
            None => {
                self.players.insert(
                    player_index,
                    Player {
                        ..Default::default()
                    },
                );
                self.players.get_mut(&player_index).unwrap()
            }
        };
        match message {
            ServerMessage::Updatefrags(data) => {
                player.frags = data.frags;
            }
            ServerMessage::Updateping(data) => {
                player.ping = data.ping;
            }
            ServerMessage::Updatepl(data) => {
                player.pl = data.pl;
            }
            ServerMessage::Updateentertime(data) => {
                player.entertime = data.entertime;
            }
            ServerMessage::Updateuserinfo(data) => {
                player.uid = data.uid;
                player.userinfo.update(&data.userinfo);
                player.update_userinfo();
            }
            ServerMessage::Playerinfo(data) => match data {
                Playerinfo::PlayerinfoMvdT(playerinfo_mvd) => {
                    if let Some(origin) = playerinfo_mvd.origin {
                        origin.apply_to(&mut player.origin);
                    };
                    if let Some(angle) = playerinfo_mvd.angle {
                        angle.apply_to(&mut player.angle);
                    };
                }
                Playerinfo::PlayerinfoConnectionT(playerinfo_connection) => todo!(),
            },
            ServerMessage::Updatestatlong(data) => {
                player.stats[data.stat as usize] = data.value;
            }
            ServerMessage::Updatestat(data) => {
                player.stats[data.stat as usize] = data.value as i32;
            }
            ServerMessage::Setinfo(data) => {
                player.userinfo.update_key_value(&data.key, &data.value);
                player.update_userinfo();
            }
            ServerMessage::Setangle(data) => {
                player.angle = data.angle;
            }
            _ => {
                panic!("{:?}, is not applicable to player", message)
            }
        }
    }

    fn packet_entities(&mut self, packet_entities: &Packetentities) {
        unsafe {
            // println!("packet_entities: run({})", PACKET_ENT_COUNT);
            PACKET_ENT_COUNT += 1;
        }
        for (index, ent) in packet_entities.entities.iter().enumerate() {
            let baseline = match self.baseline_entities.get(&ent.entity_index) {
                Some(e) => e,
                None => {
                    let mut e = Entity::default();
                    e.apply_delta(ent);
                    self.entities.insert(ent.entity_index as u16, e);
                    // println!(
                    //     "packet_entities: we failed to get the baseline! {} -> {:?}",
                    //     index, ent
                    // );
                    continue;
                }
            };
            let mut e = baseline.clone();
            e.apply_delta(ent);
            // println!(
            //     "packet_entities: inserting ({}) model({}) ({})",
            //     ent.entity_index, e.model, baseline.model
            // );
            self.entities.insert(ent.entity_index as u16, e);
        }
    }

    fn deltapacket_entities(&mut self, deltapacket_entities: &Deltapacketentities) {
        // TODO: handle this error state
        let mut i = -1;
        let mut delta_index = deltapacket_entities.from;
        for deltapacket_entity in &deltapacket_entities.entities {
            if i == -1 {
                i = deltapacket_entity.entity_index as i32;
            }
            if i > deltapacket_entity.entity_index as i32 {}

            i = deltapacket_entity.entity_index as i32;
            if deltapacket_entity.remove {
                self.entities.remove(&deltapacket_entity.entity_index);
            } else {
                match self.entities.get_mut(&deltapacket_entity.entity_index) {
                    Some(ent) => {
                        ent.apply_delta(deltapacket_entity);
                    }
                    None => {
                        match self
                            .baseline_entities
                            .get_mut(&deltapacket_entity.entity_index)
                        {
                            Some(e) => {
                                let mut ent = e.clone();
                                ent.apply_delta(deltapacket_entity);
                                ent.index = deltapacket_entity.entity_index;
                                self.entities.insert(deltapacket_entity.entity_index, ent);
                            }
                            None => {
                                let mut ent = Entity::default();
                                ent.apply_delta(deltapacket_entity);
                                ent.index = deltapacket_entity.entity_index;
                                self.entities.insert(deltapacket_entity.entity_index, ent);
                            }
                        };
                    }
                };
            }
        }
    }

    fn temp_entities(&mut self, temp_entity: &Tempentity) {
        self.temp_entities
            .insert(temp_entity.entity, temp_entity.clone());
    }

    pub fn apply_messages_mvd(&mut self, messages: &'_ Vec<ServerMessage>, last: &MvdTarget) {
        for message in messages {
            match message {
                ServerMessage::Serverdata(data) => {
                    self.serverdata = data.clone();
                }
                ServerMessage::Soundlist(data) => {
                    self.sounds.extend(data.sounds.clone());
                }
                ServerMessage::Modellist(data) => {
                    self.models.extend(data.models.clone());
                }
                ServerMessage::Spawnstatic(data) => self.static_entities.push(*data),
                ServerMessage::Cdtrack(_) => continue,
                ServerMessage::Stufftext(_) => continue,
                ServerMessage::Spawnstaticsound(data) => {
                    self.static_sounds.push(data.clone());
                }
                ServerMessage::Updatefrags(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateping(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updatepl(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateentertime(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateuserinfo(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Playerinfo(data) => match data {
                    Playerinfo::PlayerinfoMvdT(playerinfo_mvd) => {
                        self.update_player(playerinfo_mvd.player_number as u16, message);
                    }
                    Playerinfo::PlayerinfoConnectionT(playerinfo_connection) => todo!(),
                },
                ServerMessage::Updatestatlong(_) => {
                    self.update_player(last.to as u16, message);
                }
                ServerMessage::Updatestat(_) => {
                    self.update_player(last.to as u16, message);
                }
                ServerMessage::Setinfo(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Lightstyle(_) => {
                    // ignore
                }
                ServerMessage::Serverinfo(_) => {
                    // ignore, but probably shouldnt be
                }
                ServerMessage::Centerprint(_) => {
                    // ignore
                }
                ServerMessage::Spawnbaseline(data) => {
                    let mut e = Entity::from_baseline(data);
                    let index = data.index;
                    e.index = index;
                    self.baseline_entities.insert(index, e);
                }
                ServerMessage::Packetentities(data) => {
                    self.packet_entities(data);
                }
                ServerMessage::Deltapacketentities(data) => {
                    self.deltapacket_entities(data);
                }
                ServerMessage::Tempentity(data) => {
                    self.temp_entities(data);
                }
                ServerMessage::Print(_) => {
                    // ignore
                }
                ServerMessage::Sound(_) => {
                    // maybe keep the sounds?
                }
                ServerMessage::Damage(_) => {
                    // ignore
                }
                ServerMessage::Setangle(data) => {
                    self.update_player(data.index as u16, message);
                }
                ServerMessage::Smallkick(_) => {
                    // ignore
                }
                ServerMessage::Muzzleflash(_) => {
                    // ignore
                }
                ServerMessage::Chokecount(_) => {
                    // ignore
                }
                ServerMessage::Bigkick(_) => {
                    // ignore
                }
                ServerMessage::Intermission(_) => {
                    // ignore
                }
                ServerMessage::Disconnect(_) => {
                    // ignore
                }
                _ => {
                    panic!(
                        "noooo! {:?} to: {}, type: {}",
                        message, last.to, last.command
                    )
                }
            }
        }
    }

    pub fn apply_messages(&mut self, messages: &'_ Vec<ServerMessage>) {
        for message in messages {
            match message {
                ServerMessage::Serverdata(data) => {
                    self.serverdata = data.clone();
                }
                ServerMessage::Soundlist(data) => {
                    self.sounds.extend(data.sounds.clone());
                }
                ServerMessage::Modellist(data) => {
                    self.models.extend(data.models.clone());
                }
                ServerMessage::Spawnbaseline(data) => {
                    self.baseline_entities
                        .insert(data.index, Entity::from_baseline(data));
                }
                ServerMessage::Spawnstatic(data) => self.static_entities.push(*data),
                ServerMessage::Cdtrack(_) => continue,
                ServerMessage::Stufftext(_) => continue,
                ServerMessage::Spawnstaticsound(data) => {
                    self.static_sounds.push(data.clone());
                }
                ServerMessage::Updatefrags(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateping(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updatepl(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateentertime(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateuserinfo(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Playerinfo(data) => match data {
                    Playerinfo::PlayerinfoMvdT(playerinfo_mvd) => {
                        self.update_player(playerinfo_mvd.player_number as u16, message);
                    }
                    Playerinfo::PlayerinfoConnectionT(playerinfo_connection) => todo!(),
                },
                ServerMessage::Updatestatlong(_) => {
                    //    self.update_player(last_to as u16, message);
                }
                ServerMessage::Updatestat(_) => {
                    //    self.update_player(last_to as u16, message);
                }
                ServerMessage::Setinfo(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Lightstyle(_) => {
                    // ignore
                }
                ServerMessage::Serverinfo(_) => {
                    // ignore, but probably shouldnt be
                }
                ServerMessage::Centerprint(_) => {
                    // ignore
                }
                ServerMessage::Packetentities(data) => {
                    self.packet_entities(data);
                }
                ServerMessage::Deltapacketentities(data) => {
                    self.deltapacket_entities(data);
                }
                ServerMessage::Tempentity(data) => {
                    self.temp_entities(data);
                }
                ServerMessage::Print(_) => {
                    // ignore
                }
                ServerMessage::Sound(_) => {
                    // maybe keep the sounds?
                }
                ServerMessage::Damage(_) => {
                    // ignore
                }
                ServerMessage::Setangle(data) => {
                    self.update_player(data.index as u16, message);
                }
                ServerMessage::Smallkick(_) => {
                    // ignore
                }
                ServerMessage::Muzzleflash(_) => {
                    // ignore
                }
                ServerMessage::Chokecount(_) => {
                    // ignore
                }
                ServerMessage::Bigkick(_) => {
                    // ignore
                }
                ServerMessage::Intermission(_) => {
                    // ignore
                }
                ServerMessage::Disconnect(_) => {
                    // ignore
                }
                _ => {}
            }
        }
    }
}
