

pub fn write_leb128u<V: Into<u64>>(
    v: V, vec: &mut Vec<u8>)
{
    let mut cur: u64 = v.into();
    loop {
        if cur < 0x80 {
            vec.push(cur as u8);
            break;
        }
        vec.push(((cur & 0x7F) as u8) | 0x80);
        cur >>= 7;
    }
}

pub unsafe fn read_leb128u(bytes: &[u8]) -> (usize, u64) {
    let mut accum: u64 = 0;
    let mut i: usize = 0;
    loop {
        debug_assert!(i < bytes.len());
        let b = *bytes.get_unchecked(i);
        accum = (accum << 7) | ((b & 0x7F) as u64);
        i += 1;
        if b < 0x80 {
            break;
        }
    }

    (i, accum)
}
