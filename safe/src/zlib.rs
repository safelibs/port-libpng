use crate::read_util::checked_decompressed_len;
use crate::types::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AncillaryInflateState {
    pub declared_bytes: usize,
    pub requested_bytes: usize,
    pub malloc_limit: png_alloc_size_t,
}

impl AncillaryInflateState {
    pub(crate) fn validate(self) -> Result<(), &'static [u8]> {
        validate_ancillary_allocation_limit(
            self.declared_bytes,
            self.requested_bytes,
            self.malloc_limit,
        )
    }
}

pub(crate) fn validate_ancillary_allocation_limit(
    declared_bytes: usize,
    requested_bytes: usize,
    malloc_limit: png_alloc_size_t,
) -> Result<(), &'static [u8]> {
    let Some(total) = checked_decompressed_len(declared_bytes, requested_bytes) else {
        return Err(&b"ancillary size overflow\0"[..]);
    };

    if malloc_limit != 0 && total > malloc_limit {
        return Err(&b"chunk data is too large\0"[..]);
    }

    Ok(())
}
