/// SkBuff is inspired by Linux kernel protocol stack `struct sk_buff`.
///
/// `buf` is slice of head of buffer (must contain all headers of a packet in its range).
///
/// <-------------------- buf ----------------------->
/// [ Ethernet Header | IP Header | TCP Header | ... ]
/// ^                 ^            ^
/// |                 |            |
///
///  `head` moves on heads of each layer protocol header.
pub(crate) struct SkBuff<'a> {
    head: usize,
    buf: &'a [u8],
}
