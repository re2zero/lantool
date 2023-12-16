use std::{
    error::Error,
    fmt::{self, Display},
};

// FIXME
pub const BTN_LEFT: u32 = 0x110;
pub const BTN_RIGHT: u32 = 0x111;
pub const BTN_MIDDLE: u32 = 0x112;
pub const BTN_BACK: u32 = 0x113;
pub const BTN_FORWARD: u32 = 0x114;

#[derive(Debug, Clone, Copy)]
pub enum PointerEvent {
    Motion {
        time: u32,
        relative_x: f64,
        relative_y: f64,
    },
    Button {
        time: u32,
        button: u32,
        state: u32,
    },
    Axis {
        time: u32,
        axis: u8,
        value: f64,
    },
    Frame {},
}

#[derive(Debug, Clone, Copy)]
pub enum KeyboardEvent {
    Key {
        time: u32,
        key: u32,
        state: u8,
    },
    Modifiers {
        mods_depressed: u32,
        mods_latched: u32,
        mods_locked: u32,
        group: u32,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Pointer(PointerEvent),
    Keyboard(KeyboardEvent),
    Release(),
    Ping(),
    Pong(),
}

impl Display for PointerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PointerEvent::Motion {
                time: _,
                relative_x,
                relative_y,
            } => write!(f, "motion({relative_x},{relative_y})"),
            PointerEvent::Button {
                time: _,
                button,
                state,
            } => write!(f, "button({button}, {state})"),
            PointerEvent::Axis {
                time: _,
                axis,
                value,
            } => write!(f, "scroll({axis}, {value})"),
            PointerEvent::Frame {} => write!(f, "frame()"),
        }
    }
}

impl Display for KeyboardEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyboardEvent::Key {
                time: _,
                key,
                state,
            } => write!(f, "key({key}, {state})"),
            KeyboardEvent::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
            } => write!(
                f,
                "modifiers({mods_depressed},{mods_latched},{mods_locked},{group})"
            ),
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Pointer(p) => write!(f, "{}", p),
            Event::Keyboard(k) => write!(f, "{}", k),
            Event::Release() => write!(f, "release"),
            Event::Ping() => write!(f, "ping"),
            Event::Pong() => write!(f, "pong"),
        }
    }
}

impl Event {
    fn event_type(&self) -> EventType {
        match self {
            Self::Pointer(_) => EventType::Pointer,
            Self::Keyboard(_) => EventType::Keyboard,
            Self::Release() => EventType::Release,
            Self::Ping() => EventType::Ping,
            Self::Pong() => EventType::Pong,
        }
    }
}

impl PointerEvent {
    fn event_type(&self) -> PointerEventType {
        match self {
            Self::Motion { .. } => PointerEventType::Motion,
            Self::Button { .. } => PointerEventType::Button,
            Self::Axis { .. } => PointerEventType::Axis,
            Self::Frame { .. } => PointerEventType::Frame,
        }
    }
}

impl KeyboardEvent {
    fn event_type(&self) -> KeyboardEventType {
        match self {
            KeyboardEvent::Key { .. } => KeyboardEventType::Key,
            KeyboardEvent::Modifiers { .. } => KeyboardEventType::Modifiers,
        }
    }
}

enum PointerEventType {
    Motion,
    Button,
    Axis,
    Frame,
}
enum KeyboardEventType {
    Key,
    Modifiers,
}
enum EventType {
    Pointer,
    Keyboard,
    Release,
    Ping,
    Pong,
}

impl TryFrom<u8> for PointerEventType {
    type Error = Box<dyn Error>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::Motion as u8 => Ok(Self::Motion),
            x if x == Self::Button as u8 => Ok(Self::Button),
            x if x == Self::Axis as u8 => Ok(Self::Axis),
            x if x == Self::Frame as u8 => Ok(Self::Frame),
            _ => Err(Box::new(ProtocolError {
                msg: format!("invalid pointer event type {}", value),
            })),
        }
    }
}

impl TryFrom<u8> for KeyboardEventType {
    type Error = Box<dyn Error>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::Key as u8 => Ok(Self::Key),
            x if x == Self::Modifiers as u8 => Ok(Self::Modifiers),
            _ => Err(Box::new(ProtocolError {
                msg: format!("invalid keyboard event type {}", value),
            })),
        }
    }
}

impl From<&Event> for Vec<u8> {
    fn from(event: &Event) -> Self {
        let event_id = vec![event.event_type() as u8];
        let event_data = match event {
            Event::Pointer(p) => p.into(),
            Event::Keyboard(k) => k.into(),
            Event::Release() => vec![],
            Event::Ping() => vec![],
            Event::Pong() => vec![],
        };
        [event_id, event_data].concat()
    }
}

#[derive(Debug)]
struct ProtocolError {
    msg: String,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Protocol violation: {}", self.msg)
    }
}
impl Error for ProtocolError {}

impl TryFrom<Vec<u8>> for Event {
    type Error = Box<dyn Error>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let event_id = u8::from_be_bytes(value[..1].try_into()?);
        match event_id {
            i if i == (EventType::Pointer as u8) => Ok(Event::Pointer(value.try_into()?)),
            i if i == (EventType::Keyboard as u8) => Ok(Event::Keyboard(value.try_into()?)),
            i if i == (EventType::Release as u8) => Ok(Event::Release()),
            i if i == (EventType::Ping as u8) => Ok(Event::Ping()),
            i if i == (EventType::Pong as u8) => Ok(Event::Pong()),
            _ => Err(Box::new(ProtocolError {
                msg: format!("invalid event_id {}", event_id),
            })),
        }
    }
}

impl From<&PointerEvent> for Vec<u8> {
    fn from(event: &PointerEvent) -> Self {
        let id = vec![event.event_type() as u8];
        let data = match event {
            PointerEvent::Motion {
                time,
                relative_x,
                relative_y,
            } => {
                let time = time.to_be_bytes();
                let relative_x = relative_x.to_be_bytes();
                let relative_y = relative_y.to_be_bytes();
                [&time[..], &relative_x[..], &relative_y[..]].concat()
            }
            PointerEvent::Button {
                time,
                button,
                state,
            } => {
                let time = time.to_be_bytes();
                let button = button.to_be_bytes();
                let state = state.to_be_bytes();
                [&time[..], &button[..], &state[..]].concat()
            }
            PointerEvent::Axis { time, axis, value } => {
                let time = time.to_be_bytes();
                let axis = axis.to_be_bytes();
                let value = value.to_be_bytes();
                [&time[..], &axis[..], &value[..]].concat()
            }
            PointerEvent::Frame {} => {
                vec![]
            }
        };
        [id, data].concat()
    }
}

impl TryFrom<Vec<u8>> for PointerEvent {
    type Error = Box<dyn Error>;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        match data.get(1) {
            Some(id) => {
                let event_type = match id.to_owned().try_into() {
                    Ok(event_type) => event_type,
                    Err(e) => return Err(e),
                };
                match event_type {
                    PointerEventType::Motion => {
                        let time = match data.get(2..6) {
                            Some(d) => u32::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 4 Bytes at index 2".into(),
                                }))
                            }
                        };
                        let relative_x = match data.get(6..14) {
                            Some(d) => f64::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 8 Bytes at index 6".into(),
                                }))
                            }
                        };
                        let relative_y = match data.get(14..22) {
                            Some(d) => f64::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 8 Bytes at index 14".into(),
                                }))
                            }
                        };
                        Ok(Self::Motion {
                            time,
                            relative_x,
                            relative_y,
                        })
                    }
                    PointerEventType::Button => {
                        let time = match data.get(2..6) {
                            Some(d) => u32::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 4 Bytes at index 2".into(),
                                }))
                            }
                        };
                        let button = match data.get(6..10) {
                            Some(d) => u32::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 4 Bytes at index 10".into(),
                                }))
                            }
                        };
                        let state = match data.get(10..14) {
                            Some(d) => u32::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 4 Bytes at index 14".into(),
                                }))
                            }
                        };
                        Ok(Self::Button {
                            time,
                            button,
                            state,
                        })
                    }
                    PointerEventType::Axis => {
                        let time = match data.get(2..6) {
                            Some(d) => u32::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 4 Bytes at index 2".into(),
                                }))
                            }
                        };
                        let axis = match data.get(6) {
                            Some(d) => *d,
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 1 Byte at index 6".into(),
                                }));
                            }
                        };
                        let value = match data.get(7..15) {
                            Some(d) => f64::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 8 Bytes at index 7".into(),
                                }));
                            }
                        };
                        Ok(Self::Axis { time, axis, value })
                    }
                    PointerEventType::Frame => Ok(Self::Frame {}),
                }
            }
            None => Err(Box::new(ProtocolError {
                msg: "Expected an element at index 0".into(),
            })),
        }
    }
}

impl From<&KeyboardEvent> for Vec<u8> {
    fn from(event: &KeyboardEvent) -> Self {
        let id = vec![event.event_type() as u8];
        let data = match event {
            KeyboardEvent::Key { time, key, state } => {
                let time = time.to_be_bytes();
                let key = key.to_be_bytes();
                let state = state.to_be_bytes();
                [&time[..], &key[..], &state[..]].concat()
            }
            KeyboardEvent::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
            } => {
                let mods_depressed = mods_depressed.to_be_bytes();
                let mods_latched = mods_latched.to_be_bytes();
                let mods_locked = mods_locked.to_be_bytes();
                let group = group.to_be_bytes();
                [
                    &mods_depressed[..],
                    &mods_latched[..],
                    &mods_locked[..],
                    &group[..],
                ]
                .concat()
            }
        };
        [id, data].concat()
    }
}

impl TryFrom<Vec<u8>> for KeyboardEvent {
    type Error = Box<dyn Error>;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        match data.get(1) {
            Some(id) => {
                let event_type = match id.to_owned().try_into() {
                    Ok(event_type) => event_type,
                    Err(e) => return Err(e),
                };
                match event_type {
                    KeyboardEventType::Key => {
                        let time = match data.get(2..6) {
                            Some(d) => u32::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 4 Bytes at index 6".into(),
                                }))
                            }
                        };
                        let key = match data.get(6..10) {
                            Some(d) => u32::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 4 Bytes at index 10".into(),
                                }))
                            }
                        };
                        let state = match data.get(10) {
                            Some(d) => *d,
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 1 Bytes at index 14".into(),
                                }))
                            }
                        };
                        Ok(KeyboardEvent::Key { time, key, state })
                    }
                    KeyboardEventType::Modifiers => {
                        let mods_depressed = match data.get(2..6) {
                            Some(d) => u32::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 4 Bytes at index 6".into(),
                                }))
                            }
                        };
                        let mods_latched = match data.get(6..10) {
                            Some(d) => u32::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 4 Bytes at index 10".into(),
                                }))
                            }
                        };
                        let mods_locked = match data.get(10..14) {
                            Some(d) => u32::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 4 Bytes at index 14".into(),
                                }))
                            }
                        };
                        let group = match data.get(14..18) {
                            Some(d) => u32::from_be_bytes(d.try_into()?),
                            None => {
                                return Err(Box::new(ProtocolError {
                                    msg: "Expected 4 Bytes at index 18".into(),
                                }))
                            }
                        };
                        Ok(KeyboardEvent::Modifiers {
                            mods_depressed,
                            mods_latched,
                            mods_locked,
                            group,
                        })
                    }
                }
            }
            None => Err(Box::new(ProtocolError {
                msg: "Expected an element at index 0".into(),
            })),
        }
    }
}
