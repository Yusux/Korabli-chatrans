use crate::packet2::{EntityInfoPacket, EntityMethodPacket, Packet, PacketType};
use crate::unpack_rpc_args;
use serde_derive::Serialize;
use std::collections::HashMap;
use tracing::debug;

/// Enumerates voicelines which can be said in the game.
#[derive(Debug, Clone, Copy, Serialize)]
pub enum VoiceLine {
    IntelRequired,
    FairWinds,
    Wilco,
    Negative,
    WellDone,
    Curses,
    UsingRadar,
    UsingHydroSearch,
    DefendTheBase, // TODO: ...except when it's "thank you"?
    SetSmokeScreen,
    /// "Provide anti-aircraft support"
    ProvideAntiAircraft,
    /// If a player is called out in the message, their avatar ID will be here.
    RequestingSupport(Option<u32>),
    /// If a player is called out in the message, their avatar ID will be here.
    Retreat(Option<i32>),

    /// The position is (letter,number) and zero-indexed. e.g. F2 is (5,1)
    AttentionToSquare((u32, u32)),

    /// Field is the avatar ID of the target
    ConcentrateFire(i32),
}

/// Enumerates the ribbons which appear in the top-right
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize)]
pub enum Ribbon {
    PlaneShotDown,
    Incapacitation,
    SetFire,
    Citadel,
    SecondaryHit,
    OverPenetration,
    Penetration,
    NonPenetration,
    Ricochet,
    TorpedoProtectionHit,
    Captured,
    AssistedInCapture,
    Spotted,
    Destroyed,
    TorpedoHit,
    Defended,
    Flooding,
    DiveBombPenetration,
    RocketPenetration,
    RocketNonPenetration,
    RocketTorpedoProtectionHit,
    DepthChargeHit,
    ShotDownByAircraft,
    BuffSeized,
    SonarOneHit,
    SonarTwoHits,
    SonarNeutralized,
    Unknown(i8),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize)]
pub enum DeathCause {
    Secondaries,
    Artillery,
    Fire,
    Flooding,
    Torpedo,
    DiveBomber,
    AerialRocket,
    AerialTorpedo,
    Detonation,
    Ramming,
    DepthCharge,
    SkipBombs,
    Unknown(u32),
}

pub enum ReplayPlayerProperty {
    AccountId = 0,
    AntiAbuseEnabled = 1,
    AvatarId = 2,
    CamouflageInfo = 3,
    ClanColor = 4,
    ClanId = 5,
    ClanTag = 6,
    CrewParams = 7,
    UNKNOWN8 = 8,
    DogTag = 9,
    FragsCount = 10,
    Id = 12,
    InvitationsEnabled = 13,
    IsAbuser = 14,
    IsAlive = 15,
    IsBot = 16,
    IsClientLoaded = 17,
    IsConnected = 18,
    IsHidden = 19,
    IsLeaver = 20,
    IsPreBattleOwner = 21,
    IsTShooter = 22,
    KilledBuildingsCount = 23,
    IsCookie = 24,
    UNKNOWN24 = 25,
    MaxHealth = 26,
    Name = 27,
    PlayerMode = 28,
    PreBattleIdOnStart = 29,
    PreBattleSign = 30,
    PreBattleId = 31,
    Realm = 32,
    ShipComponents = 33,
    ShipConfigDump = 34,
    ShipId = 35,
    ShipParamsId = 36,
    SkinId = 37,
    TeamId = 38,
    TtkStatus = 39,
}

impl From<ReplayPlayerProperty> for i64 {
    fn from(prop: ReplayPlayerProperty) -> i64 {
        prop as i64
    }
}

impl From<ReplayPlayerProperty> for u32 {
    fn from(prop: ReplayPlayerProperty) -> u32 {
        prop as u32
    }
}

/// Contains the information describing a player
#[derive(Debug, Clone, Serialize)]
pub struct ReceivedPlayer {
    /// The username of this player
    pub username: String,
    /// The player's clan
    pub clan: String,
    /// Their avatar ID in the game
    pub avatarid: i64,
    /// Their ship ID in the game
    pub shipid: i64,
    /// Unknown
    pub playerid: i64,
    //playeravatarid: i64,
    /// Which team they're on.
    pub teamid: i64,
    /// Their starting health
    pub health: i64,

    /// This is a raw dump (with the values converted to strings) of every key for the player.
    // TODO: Replace String with the actual pickle value (which is cleanly serializable)
    pub raw: HashMap<i64, String>,
}

/// Indicates that the given attacker has dealt damage
#[derive(Debug, Clone, Serialize)]
pub struct DamageReceived {
    /// Ship ID of the aggressor
    aggressor: i32,
    /// Amount of damage dealt
    damage: f32,
}

/// Sent to update the minimap display
#[derive(Debug, Clone, Serialize)]
pub struct MinimapUpdate {
    /// The ship ID of the ship to update
    entity_id: i32,
    /// Set to true if the ship should disappear from the minimap (false otherwise)
    disappearing: bool,
    /// The heading of the ship. Unit is degrees, 0 is up, positive is clockwise
    /// (so 90.0 is East)
    heading: f32,

    /// Zero is the left edge of the map, 1.0 is the right edge
    x: f32,

    /// Zero is the bottom edge of the map, 1.0 is the top edge
    y: f32,

    /// Unknown, but this appears to be something related to the big hunt
    unknown: bool,
}

/// Enumerates usable consumables in-game
#[derive(Debug, Clone, Copy, Serialize)]
pub enum Consumable {
    DamageControl,
    SpottingAircraft,
    DefensiveAntiAircraft,
    SpeedBoost,
    RepairParty,
    CatapultFighter,
    MainBatteryReloadBooster,
    TorpedoReloadBooster,
    Smoke,
    Radar,
    HydroacousticSearch,
    Hydrophone,
    EnhancedRudders,
    ReserveBattery,
    Unknown(i8),
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum CameraMode {
    OverheadMap,
    FollowingShells,
    FollowingPlanes,
    FollowingShip,
    FollowingSubmarine,
    FreeFlying,
    Unknown(u32),
}

/// Enumerates the "cruise states". See <https://github.com/lkolbly/wows-replays/issues/14#issuecomment-976784004>
/// for more information.
#[derive(Debug, Clone, Copy, Serialize)]
pub enum CruiseState {
    /// Possible values for the throttle range from -1 for reverse to 4 for full power ahead.
    Throttle,
    /// Note that not all rudder changes are indicated via cruise states, only ones
    /// set via the Q & E keys. Temporarily setting the rudder will not trigger this
    /// packet.
    ///
    /// Possible associated values are:
    /// - -2: Full rudder to port,
    /// - -1: Half rudder to port,
    /// - 0: Neutral
    /// - 1: Half rudder to starboard,
    /// - 2: Full rudder to starboard.
    Rudder,
    /// Sets the dive depth. Known values are:
    /// - 0: 0m
    /// - 1: -6m (periscope depth)
    /// - 2: -18m
    /// - 3: -30m
    /// - 4: -42m
    /// - 5: -54m
    /// - 6: -66m
    /// - 7: -80m
    DiveDepth,
    /// Indicates an unknown cruise state. Send me your replay!
    Unknown(u32),
}

#[derive(Debug, Serialize)]
pub enum DecodedPacketPayload<'replay, 'argtype, 'rawpacket> {
    /// Represents a chat message. Note that this only includes text chats, voicelines
    /// are represented by the VoiceLine variant.
    Chat {
        entity_id: u32, // TODO: Is entity ID different than sender ID?
        /// Avatar ID of the sender
        sender_id: i32,
        /// Represents the audience for the chat: Division, team, or all.
        audience: &'replay str,
        /// The actual chat message.
        message: &'replay str,
    },
    /// Sent when a voice line is played (for example, "Wilco!")
    VoiceLine {
        /// Avatar ID of the player sending the voiceline
        sender_id: i32,
        /// True if the voiceline is visible in all chat, false if only in team chat
        is_global: bool,
        /// Which voiceline it is.
        message: VoiceLine,
    },
    /// Sent when the player earns a ribbon
    Ribbon(Ribbon),
    /// Indicates the position of the given object.
    Position(crate::packet2::PositionPacket),
    /// Indicates the position of the player's object or camera.
    PlayerOrientation(crate::packet2::PlayerOrientationPacket),
    /// Indicates updating a damage statistic. The first tuple, `(i64,i64)`, is a two-part
    /// label indicating what type of damage this refers to. The second tuple, `(i64,f64)`,
    /// indicates the actual damage counter increment.
    ///
    /// Some known keys include:
    /// - (1, 0) key is (# AP hits that dealt damage, total AP damage dealt)
    /// - (1, 3) is (# artillery fired, total possible damage) ?
    /// - (2, 0) is (# HE penetrations, total HE damage)
    /// - (17, 0) is (# fire tick marks, total fire damage)
    DamageStat(Vec<((i64, i64), (i64, f64))>),
    /// Sent when a ship is destroyed.
    ShipDestroyed {
        /// The ship ID (note: Not the avatar ID) of the killer
        killer: i32,
        /// The ship ID (note: Not the avatar ID) of the victim
        victim: i32,
        /// Cause of death
        cause: DeathCause,
    },
    EntityMethod(&'rawpacket EntityMethodPacket<'argtype>),
    EntityProperty(&'rawpacket crate::packet2::EntityPropertyPacket<'argtype>),
    BasePlayerCreate(&'rawpacket crate::packet2::BasePlayerCreatePacket<'replay, 'argtype>),
    CellPlayerCreate(&'rawpacket crate::packet2::CellPlayerCreatePacket<'replay>),
    EntityEnter(&'rawpacket crate::packet2::EntityEnterPacket),
    EntityLeave(&'rawpacket crate::packet2::EntityLeavePacket),
    EntityCreate(&'rawpacket crate::packet2::EntityCreatePacket<'argtype>),
    /// Contains all of the info required to setup the arena state and show the initial loading screen.
    OnArenaStateReceived {
        /// Unknown
        arg0: i64,
        /// Unknown
        arg1: i8,
        /// Unknown
        arg2: HashMap<i64, Vec<Option<HashMap<String, String>>>>,
        /// A list of the players in this game
        players: Vec<ReceivedPlayer>,
    },
    CheckPing(u64),
    /// Indicates that the given victim has received damage from one or more attackers.
    DamageReceived {
        /// Ship ID of the ship being damaged
        victim: u32,
        /// List of damages happening to this ship
        aggressors: Vec<DamageReceived>,
    },
    /// Contains data for a minimap update
    MinimapUpdate {
        /// A list of the updates to make to the minimap
        updates: Vec<MinimapUpdate>,
        /// Unknown
        arg1: &'rawpacket Vec<crate::rpc::typedefs::ArgValue<'argtype>>,
    },
    /// Indicates a property update. Note that many properties contain a hierarchy of properties,
    /// for example the "state" property on the battle manager contains nested dictionaries and
    /// arrays. The top-level entity and property are specified by the `entity_id` and `property`
    /// fields. The nesting structure and how to modify the leaves are indicated by the
    /// `update_cmd` field.
    ///
    /// Within the `update_cmd` field is two fields, `levels` and `action`. `levels` indicates how
    /// to traverse to the leaf property, for example by following a dictionary key or array index.
    /// `action` indicates what action to perform once there, such as setting a subproperty to
    /// a specific value.
    ///
    /// For example, to set the `state[controlPoints][0][hasInvaders]` property, you will see a
    /// packet payload that looks like:
    /// ```ignore
    /// {
    ///     "entity_id": 576258,
    ///     "property": "state",
    ///     "update_cmd": {
    ///         "levels": [
    ///             {"DictKey": "controlPoints"},
    ///             {"ArrayIndex": 0}
    ///         ],
    ///         "action": {
    ///             "SetKey":{"key":"hasInvaders","value":1}
    ///         }
    ///     }
    /// }
    /// ```
    /// This says to take the "state" property on entity 576258, navigate to `state["controlPoints"][0]`,
    /// and set the sub-key `hasInvaders` there to 1.
    ///
    /// The following properties and values are known:
    /// - `state["controlPoints"][N]["invaderTeam"]`: Indicates the team ID of the team currently
    ///   contesting the control point. -1 if nobody is invading point.
    /// - `state["controlPoints"][N]["hasInvaders"]`: 1 if the point is being contested, 0 otherwise.
    /// - `state["controlPoints"][N]["progress"]`: A tuple of two elements. The first is the fraction
    ///   captured, ranging from 0 to 1 as the point is captured, and the second is the amount of
    ///   time remaining until the point is captured.
    /// - `state["controlPoints"][N]["bothInside"]`: 1 if both teams are currently in point, 0 otherwise.
    /// - `state["missions"]["teamsScore"][N]["score"]`: The value of team N's score.
    PropertyUpdate(&'rawpacket crate::packet2::PropertyUpdatePacket<'argtype>),
    /// Indicates that the battle has ended
    BattleEnd {
        /// The team ID of the winning team (corresponds to the teamid in [OnArenaStateReceivedPlayer])
        winning_team: i8,
        /// Unknown
        // TODO: Probably how the game was won? (time expired, score, or ships destroyed)
        unknown: u8,
    },
    /// Sent when a consumable is activated
    Consumable {
        /// The ship ID of the ship using the consumable
        entity: u32,
        /// The consumable
        consumable: Consumable,
        /// How long the consumable will be active for
        duration: f32,
    },
    /// Indicates a change to the "cruise state," which is the fixed settings for various controls
    /// such as steering (using the Q & E keys), throttle, and dive planes.
    CruiseState {
        /// Which cruise state is being affected
        state: CruiseState,
        /// See [CruiseState] for what the values mean.
        value: i32,
    },
    Map(&'rawpacket crate::packet2::MapPacket<'replay>),
    /// A string representation of the game version this replay is from.
    Version(String),
    Camera(&'rawpacket crate::packet2::CameraPacket),
    /// Indicates a change in the current camera mode
    CameraMode(CameraMode),
    /// If true, indicates that the player has enabled the "free look" camera (by holding right click)
    CameraFreeLook(bool),
    /// This is a packet of unknown type
    Unknown(&'replay [u8]),
    /// This is a packet of known type, but which we were unable to parse
    Invalid(&'rawpacket crate::packet2::InvalidPacket<'replay>),
    /// If parsing with audits enabled, this indicates a packet that may be of special interest
    /// for whoever is reading the audits.
    Audit(String),
    /*
    ArtilleryHit(ArtilleryHitPacket<'a>),
    */
    /// This is a packet of EntityInfo type,
    /// and since we only need players' info, we only parse the player info.
    EntityInfo{
        players: Vec<ReceivedPlayer>,
    }
}

fn try_convert_hashable_pickle_to_string(
    value: serde_pickle::value::HashableValue,
) -> serde_pickle::value::HashableValue {
    match value {
        serde_pickle::value::HashableValue::Bytes(b) => {
            if let Ok(s) = std::str::from_utf8(&b) {
                serde_pickle::value::HashableValue::String(s.to_owned())
            } else {
                serde_pickle::value::HashableValue::Bytes(b)
            }
        }
        serde_pickle::value::HashableValue::Tuple(t) => serde_pickle::value::HashableValue::Tuple(
            t.into_iter()
                .map(|item| try_convert_hashable_pickle_to_string(item))
                .collect(),
        ),
        serde_pickle::value::HashableValue::FrozenSet(s) => {
            serde_pickle::value::HashableValue::FrozenSet(
                s.into_iter()
                    .map(|item| try_convert_hashable_pickle_to_string(item))
                    .collect(),
            )
        }
        value => value,
    }
}

fn try_convert_pickle_to_string(value: serde_pickle::value::Value) -> serde_pickle::value::Value {
    match value {
        serde_pickle::value::Value::Bytes(b) => {
            if let Ok(s) = std::str::from_utf8(&b) {
                serde_pickle::value::Value::String(s.to_owned())
            } else {
                serde_pickle::value::Value::Bytes(b)
            }
        }
        serde_pickle::value::Value::List(l) => serde_pickle::value::Value::List(
            l.into_iter()
                .map(|item| try_convert_pickle_to_string(item))
                .collect(),
        ),
        serde_pickle::value::Value::Tuple(t) => serde_pickle::value::Value::Tuple(
            t.into_iter()
                .map(|item| try_convert_pickle_to_string(item))
                .collect(),
        ),
        serde_pickle::value::Value::Set(s) => serde_pickle::value::Value::Set(
            s.into_iter()
                .map(|item| try_convert_hashable_pickle_to_string(item))
                .collect(),
        ),
        serde_pickle::value::Value::FrozenSet(s) => serde_pickle::value::Value::FrozenSet(
            s.into_iter()
                .map(|item| try_convert_hashable_pickle_to_string(item))
                .collect(),
        ),
        serde_pickle::value::Value::Dict(d) => serde_pickle::value::Value::Dict(
            d.into_iter()
                .map(|(k, v)| {
                    (
                        try_convert_hashable_pickle_to_string(k),
                        try_convert_pickle_to_string(v),
                    )
                })
                .collect(),
        ),
        value => value,
    }
}

impl<'replay, 'argtype, 'rawpacket> DecodedPacketPayload<'replay, 'argtype, 'rawpacket>
where
    'rawpacket: 'replay,
    'rawpacket: 'argtype,
{
    fn from(
        audit: bool,
        payload: &'rawpacket crate::packet2::PacketType<'replay, 'argtype>,
        packet_type: u32,
    ) -> Self {
        // debug!("payload = {:?}", payload);
        match payload {
            PacketType::EntityMethod(ref em) => {
                DecodedPacketPayload::from_entity_method(audit, em)
            }
            PacketType::Camera(camera) => DecodedPacketPayload::Camera(camera),
            PacketType::CameraMode(mode) => match mode {
                3 => DecodedPacketPayload::CameraMode(CameraMode::OverheadMap),
                5 => DecodedPacketPayload::CameraMode(CameraMode::FollowingShells),
                6 => DecodedPacketPayload::CameraMode(CameraMode::FollowingPlanes),
                8 => DecodedPacketPayload::CameraMode(CameraMode::FollowingShip),
                9 => DecodedPacketPayload::CameraMode(CameraMode::FreeFlying),
                11 => DecodedPacketPayload::CameraMode(CameraMode::FollowingSubmarine),
                _ => {
                    if audit {
                        DecodedPacketPayload::Audit(format!("CameraMode({})", mode))
                    } else {
                        DecodedPacketPayload::CameraMode(CameraMode::Unknown(*mode))
                    }
                }
            },
            PacketType::CameraFreeLook(freelook) => match freelook {
                0 => DecodedPacketPayload::CameraFreeLook(false),
                1 => DecodedPacketPayload::CameraFreeLook(true),
                _ => {
                    if audit {
                        DecodedPacketPayload::Audit(format!("CameraFreeLook({})", freelook))
                    } else {
                        DecodedPacketPayload::CameraFreeLook(true)
                    }
                }
            },
            PacketType::CruiseState(cs) => match cs.key {
                0 => DecodedPacketPayload::CruiseState {
                    state: CruiseState::Throttle,
                    value: cs.value,
                },
                1 => DecodedPacketPayload::CruiseState {
                    state: CruiseState::Rudder,
                    value: cs.value,
                },
                2 => DecodedPacketPayload::CruiseState {
                    state: CruiseState::DiveDepth,
                    value: cs.value,
                },
                _ => {
                    if audit {
                        DecodedPacketPayload::Audit(format!(
                            "CruiseState(unknown={}, {})",
                            cs.key, cs.value
                        ))
                    } else {
                        DecodedPacketPayload::CruiseState {
                            state: CruiseState::Unknown(cs.key),
                            value: cs.value,
                        }
                    }
                }
            },
            PacketType::Map(map) => {
                if audit && map.unknown != 0 && map.unknown != 1 {
                    DecodedPacketPayload::Audit(format!(
                        "Map: Unknown bool is not a bool (is {})",
                        map.unknown
                    ))
                } else if audit
                    && map.matrix
                        != [
                            0, 0, 128, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            128, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63,
                        ]
                {
                    DecodedPacketPayload::Audit(format!(
                        "Map: Unit matrix is not a unit matrix (is {:?})",
                        map.matrix
                    ))
                } else {
                    DecodedPacketPayload::Map(map)
                }
            }
            PacketType::EntityProperty(p) => DecodedPacketPayload::EntityProperty(p),
            PacketType::Position(pos) => DecodedPacketPayload::Position((*pos).clone()),
            PacketType::PlayerOrientation(pos) => {
                DecodedPacketPayload::PlayerOrientation((*pos).clone())
            }
            PacketType::BasePlayerCreate(b) => DecodedPacketPayload::BasePlayerCreate(b),
            PacketType::CellPlayerCreate(c) => DecodedPacketPayload::CellPlayerCreate(c),
            PacketType::EntityEnter(e) => DecodedPacketPayload::EntityEnter(e),
            PacketType::EntityLeave(e) => DecodedPacketPayload::EntityLeave(e),
            PacketType::EntityCreate(e) => DecodedPacketPayload::EntityCreate(e),
            PacketType::PropertyUpdate(update) => DecodedPacketPayload::PropertyUpdate(update),
            PacketType::Version(version) => DecodedPacketPayload::Version(version.clone()),
            PacketType::EntityInfo(ref entity_info) => {
                DecodedPacketPayload::from_entity_info(entity_info)
            },
            PacketType::Unknown(u) => {
                if packet_type == 0x18 {
                    if audit
                        && u != &[
                            00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00,
                            00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00,
                            00, 00, 00, 00, 00, 00, 0x80, 0xbf, 00, 00, 0x80, 0xbf, 00, 00, 0x80,
                            0xbf,
                        ]
                    {
                        DecodedPacketPayload::Audit(format!("Camera18 unexpected value!"))
                    } else {
                        DecodedPacketPayload::Unknown(&u)
                    }
                } else {
                    DecodedPacketPayload::Unknown(&u)
                }
            }
            PacketType::Invalid(u) => DecodedPacketPayload::Invalid(&u),
        }
    }

    fn from_entity_method(
        _audit: bool,
        packet: &'rawpacket EntityMethodPacket<'argtype>,
    ) -> Self {
        let entity_id = &packet.entity_id;
        let method = &packet.method;
        let args = &packet.args;
        if *method == "onChatMessageRegular" {
            let target = match &args[1] {
                crate::rpc::typedefs::ArgValue::String(s) => s,
                _ => panic!("foo"),
            };
            let message = match &args[2] {
                crate::rpc::typedefs::ArgValue::String(s) => s,
                _ => panic!("foo"),
            };
            let sender_id = match &args[0] {
                crate::rpc::typedefs::ArgValue::Int32(i) => i,
                _ => panic!("foo"),
            };
            DecodedPacketPayload::Chat {
                entity_id: *entity_id,
                sender_id: *sender_id,
                audience: std::str::from_utf8(&target).unwrap(),
                message: std::str::from_utf8(&message).unwrap(),
            }
        } else if *method == "onArenaStateReceived" {
            debug!("onArenaStateReceived {:?}", args);
            let (arg0, arg1) = unpack_rpc_args!(args, i64, i8);

            let value = serde_pickle::de::value_from_slice(
                match &args[2] {
                    crate::rpc::typedefs::ArgValue::Blob(x) => x,
                    _ => panic!("foo"),
                },
                serde_pickle::de::DeOptions::new(),
            )
            .unwrap();

            let value = match value {
                serde_pickle::value::Value::Dict(d) => d,
                _ => panic!(),
            };
            let mut arg2 = HashMap::new();
            for (k, v) in value.iter() {
                let k = match k {
                    serde_pickle::value::HashableValue::I64(i) => *i,
                    _ => panic!(),
                };
                let v = match v {
                    serde_pickle::value::Value::List(l) => l,
                    _ => panic!(),
                };
                let v: Vec<_> = v
                    .iter()
                    .map(|elem| match elem {
                        serde_pickle::value::Value::Dict(d) => Some(
                            d.iter()
                                .map(|(k, v)| {
                                    let k = match k {
                                        serde_pickle::value::HashableValue::Bytes(b) => {
                                            std::str::from_utf8(b).unwrap().to_string()
                                        }
                                        _ => panic!(),
                                    };
                                    let v = format!("{:?}", v);
                                    (k, v)
                                })
                                .collect(),
                        ),
                        serde_pickle::value::Value::None => None,
                        _ => panic!(),
                    })
                    .collect();
                arg2.insert(k, v);
            }

            let value = serde_pickle::de::value_from_slice(
                match &args[3] {
                    crate::rpc::typedefs::ArgValue::Blob(x) => x,
                    _ => panic!("foo"),
                },
                serde_pickle::de::DeOptions::new(),
            )
            .unwrap();
            let value = try_convert_pickle_to_string(value);

            let mut players_out = vec![];
            if let serde_pickle::value::Value::List(players) = &value {
                for player in players.iter() {
                    let mut values = HashMap::new();
                    if let serde_pickle::value::Value::List(elements) = player {
                        for elem in elements.iter() {
                            if let serde_pickle::value::Value::Tuple(kv) = elem {
                                let key = match kv[0] {
                                    serde_pickle::value::Value::I64(key) => key,
                                    _ => panic!(),
                                };
                                values.insert(key, kv[1].clone());
                            }
                        }
                    }

                    let avatar = values.get(&ReplayPlayerProperty::AvatarId.into()).unwrap();
                    let username = values.get(&ReplayPlayerProperty::Name.into()).unwrap();
                    let username = match username {
                        serde_pickle::value::Value::String(s) => s,
                        _ => {
                            panic!("{:?}", username);
                        }
                    };
                    let clan = values.get(&ReplayPlayerProperty::ClanTag.into()).unwrap();
                    let clan = match clan {
                        serde_pickle::value::Value::String(s) => s.clone(),
                        _ => {
                            panic!("{:?}", clan);
                        }
                    };
                    let shipid = values.get(&ReplayPlayerProperty::ShipId.into()).unwrap();
                    let playerid = values.get(&ReplayPlayerProperty::Id.into()).unwrap();
                    let _playeravatarid = values.get(&ReplayPlayerProperty::SkinId.into()).unwrap();
                    let team = values.get(&ReplayPlayerProperty::TeamId.into()).unwrap();
                    let health = values.get(&ReplayPlayerProperty::MaxHealth.into()).unwrap();

                    let mut raw = HashMap::new();
                    for (k, v) in values.iter() {
                        raw.insert(*k, format!("{:?}", v));
                    }
                    players_out.push(ReceivedPlayer {
                        username: username.to_string(),
                        clan: clan,
                        avatarid: match avatar {
                            serde_pickle::value::Value::I64(i) => *i,
                            _ => panic!("foo"),
                        },
                        shipid: match shipid {
                            serde_pickle::value::Value::I64(i) => *i,
                            _ => panic!("foo"),
                        },
                        playerid: match playerid {
                            serde_pickle::value::Value::I64(i) => *i,
                            _ => panic!("foo"),
                        },
                        teamid: match team {
                            serde_pickle::value::Value::I64(i) => *i,
                            _ => panic!("foo"),
                        },
                        health: match health {
                            serde_pickle::value::Value::I64(i) => *i,
                            _ => panic!("foo"),
                        },
                        raw: raw,
                    });
                }
            }
            DecodedPacketPayload::OnArenaStateReceived {
                arg0,
                arg1,
                arg2,
                players: players_out,
            }
        } else {
            DecodedPacketPayload::EntityMethod(packet)
        }
    }

    fn from_entity_info(packet: &'rawpacket EntityInfoPacket<'replay>) -> Self {
        let players = packet.entities
            .iter()
            .filter_map(|entity| {
                if entity.is_bot {
                    return None;
                } else {
                    debug!("The raw blobs are:");
                    debug!("\tusername: {:?}", entity.data.get(&ReplayPlayerProperty::Name.into()).unwrap().blob);
                    debug!("\tclan: {:?}", entity.data.get(&ReplayPlayerProperty::ClanTag.into()).unwrap().blob);
                    debug!("\tavatarid: {:?}", entity.data.get(&ReplayPlayerProperty::AvatarId.into()).unwrap().blob);
                    debug!("\tplayerid: {:?}", entity.data.get(&ReplayPlayerProperty::Id.into()).unwrap().blob);
                    debug!("\thealth: {:?}", entity.data.get(&ReplayPlayerProperty::MaxHealth.into()).unwrap().blob);
                    return Some(ReceivedPlayer {
                        username: entity.data.get(&ReplayPlayerProperty::Name.into()).unwrap().as_string(),
                        clan: entity.data.get(&ReplayPlayerProperty::ClanTag.into()).unwrap().as_string(),
                        avatarid: entity.data.get(&ReplayPlayerProperty::AvatarId.into()).unwrap().as_u32().into(),
                        shipid: entity.data.get(&ReplayPlayerProperty::ShipId.into()).unwrap().as_u32().into(),
                        playerid: entity.data.get(&ReplayPlayerProperty::Id.into()).unwrap().as_u32().into(),
                        teamid: entity.data.get(&ReplayPlayerProperty::TeamId.into()).unwrap().as_u32().into(),
                        health: entity.data.get(&ReplayPlayerProperty::MaxHealth.into()).unwrap().as_u32().into(),
                        raw: HashMap::new(), // Naive implementation
                    });
                }
            }).collect();
        debug!("{:?}", players);
        DecodedPacketPayload::EntityInfo {
            players: players,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DecodedPacket<'replay, 'argtype, 'rawpacket> {
    pub packet_type: u32,
    pub clock: f32,
    pub payload: DecodedPacketPayload<'replay, 'argtype, 'rawpacket>,
}

impl<'replay, 'argtype, 'rawpacket> DecodedPacket<'replay, 'argtype, 'rawpacket>
where
    'rawpacket: 'replay,
    'rawpacket: 'argtype,
{
    pub fn from(
        audit: bool,
        packet: &'rawpacket Packet<'_, '_>,
    ) -> Self {
        let decoded = Self {
            clock: packet.clock,
            packet_type: packet.packet_type,
            payload: DecodedPacketPayload::from(
                audit,
                &packet.payload,
                packet.packet_type,
            ),
        };
        decoded
    }
}
