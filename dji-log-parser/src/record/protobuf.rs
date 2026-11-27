//! Minimal reader of protocol buffers messages, whose schema isn't published.

/// Value of a protocol buffers field
pub(crate) enum Value<'a> {
    Varint(u64),
    Fixed64([u8; 8]),
    Bytes(&'a [u8]),
    Fixed32,
}

/// Splits protocol buffers data into fields: a field number and a value. Trailing zeros end the
/// data. Fails if the data is not made of fields.
pub(crate) fn fields(mut data: &[u8]) -> Option<Vec<(u32, Value<'_>)>> {
    let mut fields = Vec::new();
    while !data.is_empty() {
        if data.iter().all(|byte| *byte == 0) {
            break;
        }
        let key = varint(&mut data)?;
        let number = u32::try_from(key >> 3).ok().filter(|number| *number > 0)?;
        let value = match key & 0x7 {
            0 => Value::Varint(varint(&mut data)?),
            1 => Value::Fixed64(take(&mut data, 8)?.try_into().ok()?),
            2 => {
                let length = usize::try_from(varint(&mut data)?).ok()?;
                Value::Bytes(take(&mut data, length)?)
            }
            5 => take(&mut data, 4).map(|_| Value::Fixed32)?,
            _ => return None,
        };
        fields.push((number, value));
    }
    Some(fields)
}

fn varint(data: &mut &[u8]) -> Option<u64> {
    let mut value = 0;
    for shift in (0..64).step_by(7) {
        let byte = *take(data, 1)?.first()?;
        value |= u64::from(byte & 0x7F) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

fn take<'a>(data: &mut &'a [u8], length: usize) -> Option<&'a [u8]> {
    let (taken, rest) = data.split_at_checked(length)?;
    *data = rest;
    Some(taken)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_end_at_trailing_zeros() {
        let fields = fields(&[0x08, 0x01, 0x12, 0x01, 0x41, 0x00, 0x00]).unwrap();

        assert!(matches!(
            fields[..],
            [(1, Value::Varint(1)), (2, Value::Bytes(b"A"))]
        ));
    }

    #[test]
    fn data_that_isn_t_made_of_fields_is_rejected() {
        assert!(fields(&[0x00, 0x01]).is_none());
        assert!(fields(&[0x12, 0x05, 0x41]).is_none());
    }
}
