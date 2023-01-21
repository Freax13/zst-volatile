#![no_std]

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub use zst_volatile_macro::VolatileStruct;

use offset::{Offset, Zero};

pub mod offset;

pub struct Volatile<T, O = Zero> {
    _type_marker: PhantomData<T>,
    _offset_marker: PhantomData<O>,
}

impl<T> Volatile<T> {
    pub fn from_ref(r: &T) -> &Self {
        unsafe { &*(r as *const T as usize as *const Self) }
    }

    pub fn from_mut(r: &mut T) -> &mut Self {
        unsafe { &mut *(r as *mut T as usize as *mut Self) }
    }
}

impl<T, O> Volatile<T, O>
where
    O: Offset,
{
    pub fn as_ptr(&self) -> NonNull<T> {
        let base = self as *const Self as *const u8;
        // FIXME: Could we just use add here?
        let ptr = base.wrapping_add(O::OFFSET) as *mut T;
        let ptr = ptr as usize as *mut T;
        NonNull::new(ptr).unwrap()
    }

    pub fn read(&self) -> T
    where
        T: Copy,
    {
        let ptr = self.as_ptr();
        unsafe { ptr.as_ptr().read_volatile() }
    }

    pub fn write(&mut self, value: T)
    where
        T: Copy,
    {
        let ptr = self.as_ptr();
        unsafe { ptr.as_ptr().write_volatile(value) }
    }
}

pub unsafe trait VolatileStruct {
    type Struct;
}

impl<T, O> Deref for Volatile<T, O>
where
    T: VolatileStruct,
    O: Offset,
{
    type Target = T::Struct;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.as_ptr().as_ptr() as *const T::Struct) }
    }
}

impl<T, O> DerefMut for Volatile<T, O>
where
    T: VolatileStruct,
    O: Offset,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.as_ptr().as_ptr() as *mut T::Struct) }
    }
}
