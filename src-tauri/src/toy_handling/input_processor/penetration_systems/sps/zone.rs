/*
 * One SPS zone (a socket, plug or touch area) and the readings it gives.
 *
 * Addresses (after /avatar/parameters/):
 *
 *   OGB/<Type>/<Id>/<Contact>
 *   VFH/Zone/<Type>/<Id>/<Contact>
 *
 * <Contact> may itself contain slashes (collision tags do).
 *
 * | <Type> | Zone   | Format |
 * |--------|--------|--------|
 * | Pen    | plug   | legacy |
 * | Orf    | socket | legacy |
 * | Plug   | plug   | SPS3   |
 * | Socket | socket | SPS3   |
 * | Touch  | touch  | -      |
 *
 * Legacy zones report depth as a proximity (PenSelf, PenOthers), plus
 * root/tip proximities on sockets to learn the plug's length. SPS3 zones
 * report a distance (Depth*_PointSeparation) and the plug's length, so depth
 * is worked out directly.
 */

use std::collections::HashMap;

use vibecheck_core::Signal;

use super::length::{LengthLearner, TIP_INSIDE};

/// Longest plug length estimate believed from geometry, in metres.
const MAX_ESTIMATED_LENGTH: f64 = 3.0;

/// Contacts that describe a zone rather than report contact: they're set
/// once when the avatar loads. VRChat only sends parameters over OSC when they
/// change, so if one sits at its default it's never sent (see
/// osc/avatar_query.rs, which reads them from VRChat's OSCQuery instead).
const SETTINGS: [&str; 3] = ["LengthMult1", "LengthMult2", "DepthRadius"];

/// Whether a zone contact is a setting rather than a contact reading.
pub fn is_setting(contact: &str) -> bool {
    SETTINGS.contains(&contact)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneKind {
    Plug,
    Socket,
    Touch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Legacy,
    Sps3,
}

/// An SPS parameter address, split up.
#[derive(Debug, PartialEq, Eq)]
pub struct Address<'a> {
    /// Identifies the zone: everything up to and including its id.
    pub zone: &'a str,
    pub kind: ZoneKind,
    pub format: Format,
    /// The zone's name, e.g. a socket's menu label.
    pub id: &'a str,
    pub contact: &'a str,
}

impl<'a> Address<'a> {
    /// `param` is the address after `/avatar/parameters/`. `None` for
    /// anything that isn't a zone contact (e.g. `OGB/Version/11`).
    pub fn parse(param: &'a str) -> Option<Self> {
        let rest_start = if param.starts_with("OGB/") {
            "OGB/".len()
        } else if param.starts_with("VFH/Zone/") {
            "VFH/Zone/".len()
        } else {
            return None;
        };
        let mut parts = param[rest_start..].splitn(3, '/');
        let (wire_type, id, contact) = (parts.next()?, parts.next()?, parts.next()?);
        if id.is_empty() || contact.is_empty() {
            return None;
        }
        let (kind, format) = match wire_type {
            "Pen" => (ZoneKind::Plug, Format::Legacy),
            "Orf" => (ZoneKind::Socket, Format::Legacy),
            "Plug" => (ZoneKind::Plug, Format::Sps3),
            "Socket" => (ZoneKind::Socket, Format::Sps3),
            "Touch" => (ZoneKind::Touch, Format::Legacy),
            _ => return None,
        };
        let zone_len = rest_start + wire_type.len() + 1 + id.len();
        Some(Self {
            zone: &param[..zone_len],
            kind,
            format,
            id,
            contact,
        })
    }
}

/// Whose contact: the avatar's own body, or someone else's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Party {
    Own,
    Others,
}

impl Party {
    const BOTH: [Party; 2] = [Party::Own, Party::Others];

    fn name(self) -> &'static str {
        match self {
            Party::Own => "Self",
            Party::Others => "Others",
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// A value read as a flag: `true`, or a number above 0.
fn flag(signal: Signal) -> bool {
    match signal {
        Signal::Bool(b) => b,
        Signal::Float(v) => v > 0.0,
    }
}

#[derive(Debug)]
pub struct Zone {
    kind: ZoneKind,
    format: Format,
    values: HashMap<String, Signal>,
    /// Legacy sockets: the plug length learned from root/tip, per party.
    learners: [LengthLearner; 2],
    /// SPS3 sockets: the length of the plug in the socket, per party.
    plug_length: [Option<f64>; 2],
    /// SPS3 sockets with a legacy plug: the length estimated from geometry,
    /// kept while the plug is fully in (when the estimate can't improve).
    estimate: [Option<f64>; 2],
    /// Which parties' lengths need working out again.
    stale: [bool; 2],
}

impl Zone {
    pub fn new(kind: ZoneKind, format: Format) -> Self {
        Self {
            kind,
            format,
            values: HashMap::new(),
            learners: Default::default(),
            plug_length: [None; 2],
            estimate: [None; 2],
            stale: [false; 2],
        }
    }

    pub fn set(&mut self, contact: &str, value: Signal) {
        for party in Party::BOTH {
            if self.affects_length(contact, party) {
                self.stale[party.index()] = true;
            }
        }
        self.values.insert(contact.to_owned(), value);
    }

    fn affects_length(&self, contact: &str, party: Party) -> bool {
        let p = party.name();
        match (self.kind, self.format) {
            (ZoneKind::Socket, Format::Legacy) => {
                contact == format!("Pen{p}NewRoot") || contact == format!("Pen{p}NewTip")
            }
            (ZoneKind::Socket, Format::Sps3) => {
                contact == "DepthRadius"
                    || contact
                        .strip_prefix("Depth")
                        .and_then(|c| c.strip_prefix(p))
                        .is_some_and(|c| c.is_empty() || c.starts_with('_'))
            }
            _ => false,
        }
    }

    /// Works out plug lengths from what changed since the last call. Called
    /// once per tick, so root and tip (or proximity and separation) that
    /// changed in the same tick are used together.
    pub fn settle(&mut self) {
        for party in Party::BOTH {
            let i = party.index();
            if !std::mem::take(&mut self.stale[i]) {
                continue;
            }
            match self.format {
                Format::Legacy => {
                    let p = party.name();
                    let root = self.number(&format!("Pen{p}NewRoot"));
                    let tip = self.number(&format!("Pen{p}NewTip"));
                    self.learners[i].observe(root, tip);
                }
                Format::Sps3 => self.plug_length[i] = self.socket_plug_length(party),
            }
        }
    }

    /// Every reading this zone gives, from 0 upwards. Linear features only
    /// follow depth, since touch has no position to stroke to.
    pub fn readings(&self, linear: bool) -> Vec<f64> {
        let mut readings = Vec::new();
        match self.kind {
            ZoneKind::Plug | ZoneKind::Socket => {
                readings.extend(Party::BOTH.map(|party| self.depth(party)));
                if !linear {
                    let legacy = self.format == Format::Legacy;
                    let plug = self.kind == ZoneKind::Plug;
                    readings.push(self.touch("TouchSelf", legacy));
                    readings.push(self.touch("TouchOthers", legacy));
                    // Legacy sockets don't gate frot on a Close flag
                    readings.push(self.touch("FrotOthers", legacy && plug));
                }
            }
            ZoneKind::Touch => {
                if !linear {
                    readings.push(self.touch("Self", false));
                    readings.push(self.touch("Others", false));
                }
            }
        }
        readings
    }

    fn number(&self, contact: &str) -> Option<f64> {
        match self.values.get(contact) {
            Some(Signal::Float(v)) if v.is_finite() => Some(*v),
            _ => None,
        }
    }

    fn flag(&self, contact: &str) -> Option<bool> {
        self.values.get(contact).map(|v| flag(*v))
    }

    fn depth(&self, party: Party) -> f64 {
        let p = party.name();
        let depth = match (self.kind, self.format) {
            (ZoneKind::Touch, _) => None,
            (_, Format::Sps3) => self.sps3_depth(party),
            (ZoneKind::Plug, Format::Legacy) => self.number(&format!("Pen{p}")),
            (ZoneKind::Socket, Format::Legacy) => self.root_tip_depth(party).or_else(|| {
                // PenOthers counts while PenOthersClose is on, or if the
                // avatar doesn't send that flag at all
                let close = party == Party::Own || self.flag("PenOthersClose").unwrap_or(true);
                close.then(|| self.number(&format!("Pen{p}"))).flatten()
            }),
        };
        depth.unwrap_or(0.0).clamp(0.0, 1.0)
    }

    /// Legacy socket depth from root/tip and the learned length. `None` when
    /// no plug is near, so the plain proximity is used instead.
    fn root_tip_depth(&self, party: Party) -> Option<f64> {
        let p = party.name();
        let root = self.number(&format!("Pen{p}NewRoot"))?;
        let tip = self.number(&format!("Pen{p}NewTip"))?;
        if root <= 0.0 && tip <= 0.0 {
            return None;
        }
        // Until the tip is in, there's no depth yet
        let length = self.learners[party.index()]
            .length()
            .filter(|_| tip > TIP_INSIDE);
        Some(length.map_or(0.0, |length| 1.0 - (1.0 - root) / length))
    }

    /// SPS3 depth: how much of the plug's length is past the contact point.
    fn sps3_depth(&self, party: Party) -> Option<f64> {
        let contact = format!("Depth{}", party.name());
        let proximity = self.number(&contact)?;
        let separation = self.number(&format!("{contact}_PointSeparation"))?;
        if proximity <= 0.0 || separation < 0.0 {
            return None;
        }
        let length = match self.kind {
            ZoneKind::Plug => self.number("LengthMult1")? * self.number("LengthMult2")?,
            _ => self.plug_length[party.index()]?,
        };
        (length.is_finite() && length > 0.0).then(|| 1.0 - separation / length)
    }

    /// SPS3 socket: the length of the plug in it. Native SPS3 plugs say
    /// (they carry an SPS/Plug collision tag); legacy plugs are estimated
    /// from the receiver's radius and how far in they are.
    fn socket_plug_length(&mut self, party: Party) -> Option<f64> {
        let i = party.index();
        let contact = format!("Depth{}", party.name());
        let proximity = self.number(&contact);
        let separation = self.number(&format!("{contact}_PointSeparation"));
        let (Some(proximity), Some(separation)) = (proximity, separation) else {
            self.estimate[i] = None;
            return None;
        };
        if proximity <= 0.0 || separation < 0.0 {
            self.estimate[i] = None;
            return None;
        }

        let tag_prefix = format!("{contact}_CollisionTag_SPS/Plug/");
        let native = self
            .values
            .iter()
            .any(|(key, value)| key.starts_with(&tag_prefix) && flag(*value));
        if native {
            self.estimate[i] = None;
            return self
                .number(&format!("{contact}_SenderLossyScaleZ"))
                .filter(|length| *length > 0.0);
        }

        // Fully in, the geometry only gives a lower bound: keep what we had
        if proximity >= 1.0 && self.estimate[i].is_some() {
            return self.estimate[i];
        }
        let Some(radius) = self.number("DepthRadius").filter(|r| *r > 0.0) else {
            return self.estimate[i];
        };
        let estimate = separation - radius * (1.0 - proximity);
        self.estimate[i] = (estimate > 0.0 && estimate <= MAX_ESTIMATED_LENGTH).then_some(estimate);
        self.estimate[i]
    }

    /// A touch or frot reading. If the avatar sends the toucher's speed, the
    /// reading is that speed (1 m/s is full), while in contact. Otherwise
    /// it's the proximity, which `gated` contacts only count while their
    /// Close flag is on.
    fn touch(&self, contact: &str, gated: bool) -> f64 {
        let proximity = self.number(contact).unwrap_or(0.0);
        let speed_key = format!("{contact}_SenderVelocityMagnitude");
        if self.values.contains_key(&speed_key) {
            let speed = self.number(&speed_key).unwrap_or(0.0);
            return if proximity > 0.0 { speed.max(0.0) } else { 0.0 };
        }
        if gated && !self.flag(&format!("{contact}Close")).unwrap_or(false) {
            return 0.0;
        }
        proximity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_split_into_zone_and_contact() {
        assert_eq!(
            Address::parse("OGB/Socket/Mouth/DepthOthers_CollisionTag_SPS/Plug/a/b"),
            Some(Address {
                zone: "OGB/Socket/Mouth",
                kind: ZoneKind::Socket,
                format: Format::Sps3,
                id: "Mouth",
                contact: "DepthOthers_CollisionTag_SPS/Plug/a/b",
            })
        );
        let vfh = Address::parse("VFH/Zone/Touch/Hand/Others").unwrap();
        assert_eq!(
            (vfh.zone, vfh.kind),
            ("VFH/Zone/Touch/Hand", ZoneKind::Touch)
        );
        assert_eq!(
            Address::parse("OGB/Pen/Dp/PenSelf").unwrap().format,
            Format::Legacy
        );

        for not_a_zone in [
            "OGB/Version/11",
            "OGB/HapticsDisabled",
            "OGB/Orf/Mouth",
            "OGB/Orf//PenSelf",
            "TPS_Internal/Orf/0/Depth_In",
        ] {
            assert_eq!(Address::parse(not_a_zone), None, "{not_a_zone}");
        }
    }
}
