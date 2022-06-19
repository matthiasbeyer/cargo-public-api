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
