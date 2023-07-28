/// An alignment restriction to be placed on a volatile value. In most cases
/// this will be the normal alignment of the type. The fields of
/// `#[repr(packed(n))]` structs will have their alignment restricted to `n`.
///
/// A volatile value with this restriction must only be read and written
/// through the wrapper type.
///
/// Users of this crate shouldn't have to worry about this and they usually
/// shouldn't have to implement this trait.
///
/// # Safety
///
/// `Wrapper<T>` must be a transparent wrapper around `T` except in that the
/// wrapper is allowed to restrict the alignment with a `#[repr(packed(n))]`
/// attribute.
/// The `wrap` and `unwrap` values can be used to move a value into and out of
/// the wrapper.
pub unsafe trait Alignment {
    type Wrapper<T>;

    fn wrap<T>(value: T) -> Self::Wrapper<T>;
    fn unwrap<T>(value: Self::Wrapper<T>) -> T;
}

/// The normal alignment of the type.
pub struct Normal;

unsafe impl Alignment for Normal {
    type Wrapper<T> = T;

    #[inline]
    fn wrap<T>(value: T) -> Self::Wrapper<T> {
        value
    }

    #[inline]
    fn unwrap<T>(value: Self::Wrapper<T>) -> T {
        value
    }
}
