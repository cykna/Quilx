mod initial;
mod long;
pub use initial::*;
pub(crate) use long::*;
pub enum QuicPacket {
    Initial(InitialPacket),
}
