pub fn print_decimal_into_buf(mut value: u32, buf: &mut [u8], pos: usize) -> usize {
    if value == 0 {
        buf[pos] = b'0';
        return pos + 1;
    }
    let mut i = pos;
    while value > 0 && i < buf.len() {
        buf[i] = b'0' + (value % 10) as u8;
        value /= 10;
        i += 1;
    }
    let mut start = pos;
    let mut end = i - 1;
    while start < end {
        buf.swap(start, end);
        start += 1;
        end -= 1;
    }
    i
}

pub fn emplace_str_into_buf(s: &[u8], buf: &mut [u8], pos: usize) -> usize {
    let end = pos + s.len();
    if end > buf.len() {
        return pos;
    }
    buf[pos..end].copy_from_slice(s);
    end
}
