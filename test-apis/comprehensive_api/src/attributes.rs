#[doc(hidden)]
pub fn doc_hidden() {}

#[non_exhaustive]
pub enum NonExhaustive {
    MoreToCome,
}

#[must_use]
pub fn must_use() -> usize {
    0
}

#[repr(C)]
pub struct C {
    pub b: bool,
}
