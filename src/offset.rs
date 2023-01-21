use core::mem::size_of;

pub struct Zero(());

pub struct Align<B, T> {
    _base: B,
    _ty: T,
}

pub struct PastField<B, T> {
    _base: B,
    _ty: T,
}

pub trait Offset {
    const OFFSET: usize;
}

impl Offset for Zero {
    const OFFSET: usize = 0;
}

impl<B, T> Offset for Align<B, T>
where
    B: Offset,
{
    const OFFSET: usize = align::<T>(B::OFFSET);
}

impl<B, T> Offset for PastField<B, T>
where
    B: Offset,
{
    const OFFSET: usize = B::OFFSET + size_of::<T>();
}

const fn align<T>(offset: usize) -> usize {
    let align = core::mem::align_of::<T>();
    ((offset + align - 1) / align) * align
}
