///Struct that contains information about the Long Header defined in Section 17.2 of QUIC specification.
///Note that all the fields are in order of appearance on the QUIC representation of it
pub struct LongHeader {
    first_byte: u8, //header, fixed bit, long packet type, type specific are included here in this order
    version: u32,
    ///Length of both dest_id and src_id is ommited here, since we can calculate them via `.len()` method, but they are added on the raw bytes
    conn_dest_id: Vec<u8>, //0..160
    ///Length of both dest_id and src_id is ommited here, since we can calculate them via `.len()` method, but they are added on the raw bytes
    conn_src_id: Vec<u8>, //0..160
}
