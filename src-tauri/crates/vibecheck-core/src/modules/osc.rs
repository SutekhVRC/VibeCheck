//! Input module for OSC from VRChat (or anything else that speaks OSC).
//!
//! For every message received:
//!
//! - `/avatar/change` becomes an [`EventKind::AvatarChange`] event on the
//!   [`AVATAR_CHANGE_PORT`] port, carrying the avatar id as text.
//! - Any other message with a float or bool as its last argument writes that
//!   value to the signal plane, on a port named after the full OSC address
//!   (e.g. `osc//avatar/parameters/Boop`).
//! - Every message except avatar changes is also passed to the optional
//!   *tap*, in arrival order. The tap is for consumers that need every message
//!   rather than the latest value, such as VibeCheck's per-toy processing.
//!
//! Bundles are unpacked into their messages.

use std::io;
use std::net::SocketAddr;

use async_trait::async_trait;
use rosc::{OscMessage, OscPacket, OscType};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::UnboundedSender;

use crate::error::Result;
use crate::event::{EventKind, Value};
use crate::module::{Input, InputCtx, PortSpec};
use crate::signal::Signal;

pub use rosc;

/// Event port that fires when the avatar changes.
pub const AVATAR_CHANGE_PORT: &str = "avatar_change";

const AVATAR_CHANGE_ADDR: &str = "/avatar/change";

pub struct OscInput {
    socket: UdpSocket,
    tap: Option<UnboundedSender<OscMessage>>,
}

impl OscInput {
    /// Bind the listening socket. Binding is separate from [`Input::run`] so
    /// callers can report a busy port before anything starts.
    pub async fn bind(addr: SocketAddr) -> io::Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(addr).await?,
            tap: None,
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Also send every message (except avatar changes) to `tap`, in order.
    /// When the tap's receiver is dropped, [`Input::run`] returns.
    pub fn with_tap(mut self, tap: UnboundedSender<OscMessage>) -> Self {
        self.tap = Some(tap);
        self
    }

    /// Handles one message. Returns `false` once the tap's receiver is gone.
    fn handle(&self, ctx: &InputCtx, msg: OscMessage) -> bool {
        if msg.addr.starts_with(AVATAR_CHANGE_ADDR) {
            let avatar = match msg.args.last() {
                Some(OscType::String(id)) => Value::Text(id.as_str().into()),
                _ => Value::None,
            };
            ctx.publish(AVATAR_CHANGE_PORT, EventKind::AvatarChange, avatar);
            return true;
        }

        match msg.args.last() {
            Some(OscType::Float(v)) => ctx.write_signal(&msg.addr, Signal::Float(*v as f64)),
            Some(OscType::Bool(b)) => ctx.write_signal(&msg.addr, Signal::Bool(*b)),
            _ => {}
        }

        match &self.tap {
            Some(tap) => tap.send(msg).is_ok(),
            None => true,
        }
    }
}

fn flatten(packet: OscPacket, out: &mut Vec<OscMessage>) {
    match packet {
        OscPacket::Message(msg) => out.push(msg),
        OscPacket::Bundle(bundle) => {
            for packet in bundle.content {
                flatten(packet, out);
            }
        }
    }
}

#[async_trait]
impl Input for OscInput {
    /// Signal ports are created on the fly, one per OSC address received, so
    /// only the fixed event port is listed.
    fn ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::event_out(AVATAR_CHANGE_PORT)]
    }

    async fn run(&mut self, ctx: InputCtx) -> Result<()> {
        let mut buf = vec![0u8; rosc::decoder::MTU];
        let mut messages = Vec::new();
        loop {
            // Receive errors are per packet (e.g. ICMP port unreachable on
            // Windows), so keep listening.
            let Ok((len, _from)) = self.socket.recv_from(&mut buf).await else {
                continue;
            };
            let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..len]) else {
                continue;
            };
            flatten(packet, &mut messages);
            for msg in messages.drain(..) {
                if !self.handle(&ctx, msg) {
                    return Ok(());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::{ModuleInstanceId, PortId};
    use crate::event::EventPlane;
    use crate::signal::SignalPlane;
    use rosc::{OscBundle, OscTime};
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time::timeout;

    fn message(addr: &str, args: Vec<OscType>) -> OscPacket {
        OscPacket::Message(OscMessage {
            addr: addr.into(),
            args,
        })
    }

    async fn next_addr(tap: &mut mpsc::UnboundedReceiver<OscMessage>) -> String {
        timeout(Duration::from_secs(2), tap.recv())
            .await
            .unwrap()
            .unwrap()
            .addr
    }

    async fn send(to: SocketAddr, packet: &OscPacket) {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let buf = rosc::encoder::encode(packet).unwrap();
        socket.send_to(&buf, to).await.unwrap();
    }

    #[tokio::test]
    async fn routes_messages_signals_and_avatar_changes() {
        let input = OscInput::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = input.local_addr().unwrap();

        let (tap_tx, mut tap) = mpsc::unbounded_channel();
        let mut input = input.with_tap(tap_tx);
        let signals = SignalPlane::new();
        let (events, mut event_rx) = EventPlane::new();
        let ctx = InputCtx::new(ModuleInstanceId::new("osc"), signals.clone(), events);
        let task = tokio::spawn(async move { input.run(ctx).await });

        send(
            addr,
            &message("/avatar/parameters/Boop", vec![OscType::Float(0.5)]),
        )
        .await;
        send(
            addr,
            &message("/avatar/change", vec![OscType::String("avtr_x".into())]),
        )
        .await;
        send(
            addr,
            &OscPacket::Bundle(OscBundle {
                timetag: OscTime::from((0, 1)),
                content: vec![
                    message("/avatar/parameters/Tail", vec![OscType::Bool(true)]),
                    message("/avatar/parameters/Boop", vec![OscType::Float(0.75)]),
                ],
            }),
        )
        .await;

        let wait = Duration::from_secs(2);
        // Avatar change is not tapped; the rest arrive in order.
        assert_eq!(next_addr(&mut tap).await, "/avatar/parameters/Boop");
        assert_eq!(next_addr(&mut tap).await, "/avatar/parameters/Tail");
        assert_eq!(next_addr(&mut tap).await, "/avatar/parameters/Boop");

        let event = timeout(wait, event_rx.recv()).await.unwrap().unwrap();
        assert_eq!(event.kind, EventKind::AvatarChange);
        assert_eq!(event.value, Value::Text("avtr_x".into()));
        assert_eq!(
            event.source,
            PortId::new(ModuleInstanceId::new("osc"), AVATAR_CHANGE_PORT)
        );

        let port = |a: &str| PortId::new(ModuleInstanceId::new("osc"), a);
        assert_eq!(
            signals.read(&port("/avatar/parameters/Boop")),
            Some(Signal::Float(0.75))
        );
        assert_eq!(
            signals.read(&port("/avatar/parameters/Tail")),
            Some(Signal::Bool(true))
        );

        // Dropping the tap's receiver ends the input on its next message.
        drop(tap);
        send(
            addr,
            &message("/avatar/parameters/Boop", vec![OscType::Float(0.1)]),
        )
        .await;
        let result = timeout(wait, task).await.unwrap().unwrap();
        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn bind_reports_a_busy_port() {
        let first = OscInput::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = first.local_addr().unwrap();
        let err = OscInput::bind(addr).await.err().unwrap();
        assert_eq!(err.kind(), io::ErrorKind::AddrInUse);
    }
}
