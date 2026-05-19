use super::schema::LayoutJsonType;

pub(super) const POINTER_SIZE: u32 = 8;
pub(super) const POINTER_ALIGN: u32 = 8;
pub(super) const USIZE_SIZE: u32 = 8;

pub(super) fn scalar_layout(kind: &'static str, size: u32, alignment: u32) -> LayoutJsonType {
    LayoutJsonType {
        kind,
        size,
        alignment,
        fields: Vec::new(),
        variants: Vec::new(),
    }
}

pub(super) fn align_to(value: u32, alignment: u32) -> u32 {
    if alignment <= 1 {
        value
    } else {
        value.div_ceil(alignment) * alignment
    }
}
