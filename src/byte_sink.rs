

/**
 * A type which accepts a sequence of bytes.  This is
 * mostly the same as Write, except not polluted with io
 * stuff.  It just returns None when the limit is reached,
 * and keeps returning None when more is sent to it.
 */
pub trait ByteSink {
    fn send_byte(&mut self, b: u8) -> Option<usize>;
    fn send_slice<BS>(&mut self, bs: &BS) -> Option<usize>
      where BS: ?Sized + AsRef<[u8]>;
}

/**
 * A type which infallibly produces a sequence of bytes.
 * The methods are unsafe for that reason.
 */
pub trait ByteSource {
    unsafe fn take(&mut self) -> u8;
    unsafe fn take_slice(&mut self, n: usize) -> &[u8];
}

/** A type which can be written to a `ByteSink`. */
pub trait ByteSerialize: Sized {
    fn send_to<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink;

    unsafe fn take_from<S>(src: &mut S) -> (usize, Self)
      where S: ByteSource;
}

/**
 * Implementation of ByteSink with an accumulating
 * vector backend.
 */
impl ByteSink for Vec<u8> {
    fn send_byte(&mut self, b: u8) -> Option<usize> {
        self.push(b);
        Some(1)
    }
    fn send_slice<BS>(&mut self, bs: &BS) -> Option<usize>
      where BS: ?Sized + AsRef<[u8]>
    {
        let bytes = bs.as_ref();
        self.extend_from_slice(bytes);
        Some(bytes.len())
    }
}

/**
 * Helper types to encode and decode integer values
 * with LEB128.
 */
pub struct Leb128U(u64);

impl Leb128U {
    pub fn new(v: u64) -> Leb128U { Leb128U(v) }
    pub fn as_u64(&self) -> u64 { self.0 }
}
impl<T: Into<u64>> From<T> for Leb128U {
    fn from(val: T) -> Leb128U { Leb128U(val.into()) }
}

impl ByteSerialize for Leb128U {
    fn send_to<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        let mut cur: u64 = self.0;
        let mut nw: usize = 0;
        loop {
            if cur < 0x80 {
                sink.send_byte(cur as u8) ?;
                nw += 1;
                break;
            }
            sink.send_byte(((cur & 0x7F) as u8) | 0x80);
            nw += 1;
            cur >>= 7;
        }
        Some(nw)
    }

    unsafe fn take_from<S>(src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
        let mut accum: u64 = 0;
        let mut nr: usize = 0;

        loop {
            let b = src.take();
            accum = (accum << 7) | ((b & 0x7F) as u64);
            nr += 1;
            if b < 0x80 {
                break;
            }
        }

        (nr, Leb128U(accum))
    }
}
