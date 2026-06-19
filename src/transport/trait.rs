use crate::protocol::packet::Packet;

pub trait Transport {
    fn connect(&mut self) -> anyhow::Result<()>;
    fn disconnect(&mut self) -> anyhow::Result<()>;
    fn send(&mut self, packet: Packet) -> anyhow::Result<()>;
    fn receive(&mut self) -> anyhow::Result<Option<Packet>>;
}
