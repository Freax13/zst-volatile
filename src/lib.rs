#![no_std]

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub use zst_volatile_macro::VolatileStruct;

use alignment::{Alignment, Normal};
use offset::{Offset, Zero};

pub mod alignment;
pub mod offset;

pub struct Volatile<T, O = Zero, A = Normal> {
    _type_marker: PhantomData<T>,
    _offset_marker: PhantomData<O>,
    _alignment_marker: PhantomData<A>,
}

impl<T> Volatile<T> {
    pub fn from_ref(r: &T) -> &Self {
        unsafe { &*(r as *const T as usize as *const Self) }
    }

    pub fn from_mut(r: &mut T) -> &mut Self {
        unsafe { &mut *(r as *mut T as usize as *mut Self) }
    }
}

impl<T, O, A> Volatile<T, O, A>
where
    O: Offset,
    A: Alignment,
{
    pub fn as_ptr(&self) -> NonNull<T> {
        let base = self as *const Self as *const u8;
        // FIXME: Could we just use add here?
        let ptr = base.wrapping_add(O::OFFSET) as *mut T;
        let ptr = ptr as usize as *mut T;
        NonNull::new(ptr).unwrap()
    }

    fn as_wrapper_pointer(&self) -> NonNull<A::Wrapper<T>> {
        self.as_ptr().cast()
    }

    pub fn read(&self) -> T
    where
        T: Copy,
    {
        // Read the value through the wrapper. This ensures that the pointer is
        // always aligned.
        let ptr = self.as_wrapper_pointer();
        let wrapped = unsafe { ptr.as_ptr().read_volatile() };
        A::unwrap(wrapped)
    }

    pub fn write(&mut self, value: T)
    where
        T: Copy,
    {
        // Write the value through the wrapper. This ensures that the pointer
        // is always aligned.
        let value = A::wrap(value);
        let ptr = self.as_wrapper_pointer();
        unsafe { ptr.as_ptr().write_volatile(value) }
    }
}

pub unsafe trait VolatileStruct {
    type Struct<A>
    where
        A: Alignment;
}

impl<T, O, A> Deref for Volatile<T, O, A>
where
    T: VolatileStruct,
    O: Offset,
    A: Alignment,
{
    type Target = T::Struct<A>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.as_ptr().as_ptr() as *const T::Struct<A>) }
    }
}

impl<T, O, A> DerefMut for Volatile<T, O, A>
where
    T: VolatileStruct,
    O: Offset,
    A: Alignment,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.as_ptr().as_ptr() as *mut T::Struct<A>) }
    }
}
