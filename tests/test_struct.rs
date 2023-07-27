use zst_volatile::{Volatile, VolatileStruct};

#[derive(VolatileStruct)]
#[repr(C)]
pub struct Child1 {
    field1: u32,
    field2: u32,
    field3: u32,
    field4: u32,
}

#[derive(VolatileStruct)]
#[repr(C)]
pub struct Child2 {
    field1: u32,
    field2: u32,
    field3: u32,
}

#[derive(VolatileStruct)]
#[repr(C)]
pub struct Parent {
    child1: Child1,
    child2: Child2,
}

pub fn sum_parent(parent: &VolatileParent) -> u32 {
    // You can access the fields of children directly...
    let sum_part1 = parent.child1.field1.read() + parent.child1.field2.read();

    // Or you can even use destructuring.
    let VolatileChild1 { field3, field4, .. } = &*parent.child1;
    let sum_part2 = field3.read() + field4.read();

    // You can also pass a field to another method.
    let sum_part3 = sum_child2(&parent.child2);

    sum_part1 + sum_part2 + sum_part3
}

pub fn sum_child2(child: &VolatileChild2) -> u32 {
    child.field1.read() + child.field2.read() + child.field3.read()
}

pub fn modify(parent: &mut VolatileParent) {
    let child1 = &mut parent.child1;
    let child2 = &mut parent.child2;

    // Access child 1.
    child1.field3.write(3);

    // Access child 2.
    child2.field1.write(3);

    // Access child 1 again.
    //
    // If this compiles then the compiler was aware that child1 and child2 are
    // different fields pointing to different data. This is difficult to
    // emulate with methods, because methods usually require a mutable
    // reference to the whole struct.
    child1.field3.write(3);
}

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use zst_volatile::Volatile;

    use super::*;

    #[test]
    fn struct_is_zero_sized() {
        assert_eq!(size_of::<VolatileParent>(), 0);
        assert_eq!(size_of::<VolatileChild1>(), 0);
        assert_eq!(size_of::<VolatileChild2>(), 0);
    }

    #[test]
    fn test_sum() {
        let parent = Parent {
            child1: Child1 {
                field1: 1,
                field2: 2,
                field3: 3,
                field4: 4,
            },
            child2: Child2 {
                field1: 5,
                field2: 6,
                field3: 7,
            },
        };

        // Create a volatile reference.
        let volatile_parent = Volatile::from_ref(&parent);

        // Do some work on the volatile reference.
        assert_eq!(sum_parent(volatile_parent), 1 + 2 + 3 + 4 + 5 + 6 + 7);
    }

    #[test]
    fn test_modify() {
        let mut parent = Parent {
            child1: Child1 {
                field1: 1,
                field2: 2,
                field3: 3,
                field4: 4,
            },
            child2: Child2 {
                field1: 5,
                field2: 6,
                field3: 7,
            },
        };

        // Create a volatile reference.
        let volatile_parent = Volatile::from_mut(&mut parent);

        modify(volatile_parent);
    }
}

#[test]
fn test_repr_packed() {
    #[derive(VolatileStruct)]
    #[repr(C)]
    #[repr(packed(2))]
    struct S {
        a: u8,
        b: u64,
        c: u8,
        d: u32,
        e: u8,
    }

    assert_eq!(std::mem::align_of::<S>(), 2);
    assert_eq!(std::mem::size_of::<S>(), 18);

    let s = &S {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e: 5,
    };
    let v = Volatile::from_ref(s);

    let s_addr = s as *const S as usize;
    let v_addr = v.as_ptr().as_ptr() as usize;
    assert_eq!(s_addr, v_addr);

    // TODO: Volatile offset tests

    // assert_eq!(
    //     core::ptr::addr_of!(s.a) as usize - s_addr,
    //     v.a.as_ptr().as_ptr() as usize - v_addr,
    //     "field a offset"
    // );
    // assert_eq!(
    //     core::ptr::addr_of!(s.b) as usize - s_addr,
    //     v.b.as_ptr().as_ptr() as usize - v_addr,
    //     "field b offset"
    // );
    // assert_eq!(
    //     core::ptr::addr_of!(s.c) as usize - s_addr,
    //     v.c.as_ptr().as_ptr() as usize - v_addr,
    //     "field c offset"
    // );
    // assert_eq!(
    //     core::ptr::addr_of!(s.d) as usize - s_addr,
    //     v.d.as_ptr().as_ptr() as usize - v_addr,
    //     "field d offset"
    // );
}

#[test]
fn test_repr_align() {
    #[repr(C)]
    struct Child1 {
        c1: u8,
        c2: u16,
    }

    #[repr(C)]
    #[repr(align(8))]
    struct Child2 {
        c1: u8,
        c2: u16,
    }

    #[repr(C)]
    #[repr(packed(1))]
    struct Child3 {
        c1: u8,
        c2: u16,
    }

    #[derive(VolatileStruct)]
    #[repr(C)]
    #[repr(packed)]
    struct Parent1 {
        p1: Child1,
        p3: Child3,
    }

    #[derive(VolatileStruct)]
    #[repr(C)]
    struct Parent2 {
        p1: Child1,
        p2: Child2,
        p3: Child3,
    }

    assert_eq!(std::mem::align_of::<Child1>(), 2);
    assert_eq!(std::mem::size_of::<Child1>(), 4);

    assert_eq!(std::mem::align_of::<Child2>(), 8);
    assert_eq!(std::mem::size_of::<Child2>(), 8);

    assert_eq!(std::mem::align_of::<Child3>(), 1);
    assert_eq!(std::mem::size_of::<Child3>(), 3);

    assert_eq!(std::mem::align_of::<Parent1>(), 1);
    assert_eq!(std::mem::size_of::<Parent1>(), 7);

    assert_eq!(std::mem::align_of::<Parent2>(), 8);
    assert_eq!(std::mem::size_of::<Parent2>(), 24);

    // TODO: Voltile offset tests
}
