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

macro_rules! raw_offset {
    ($base:expr, $field:expr) => {
        core::ptr::addr_of!($field) as usize - $base
    };
}

macro_rules! volatile_offset {
    ($base:expr, $field:expr) => {
        $field.as_ptr().as_ptr() as usize - $base
    };
}

#[test]
fn test_repr_packed() {
    #[derive(VolatileStruct)]
    #[repr(C)]
    struct Child1 {
        x: u8,
        y: u16,
    }

    #[derive(VolatileStruct)]
    #[repr(C)]
    #[repr(align(8))]
    struct Child2 {
        x: u8,
        y: u16,
    }

    #[derive(VolatileStruct)]
    #[repr(C)]
    #[repr(packed(1))]
    struct Child3 {
        x: u8,
        y: u16,
    }

    #[derive(VolatileStruct)]
    #[repr(C)]
    struct Parent1 {
        a: u8,
        b: Child1,
        c: Child2,
        d: u8,
    }

    assert_eq!(std::mem::align_of::<Parent1>(), 8);
    assert_eq!(std::mem::size_of::<Parent1>(), 24);

    let p1 = &Parent1 {
        a: 1,
        b: Child1 { x: 2, y: 3 },
        c: Child2 { x: 4, y: 5 },
        d: 6,
    };
    let v1 = Volatile::from_ref(p1);
    let p1_addr = p1 as *const Parent1 as usize;
    let v1_addr = v1.as_ptr().as_ptr() as usize;

    assert_eq!(p1_addr, v1_addr);
    assert_eq!(raw_offset!(p1_addr, p1.a), volatile_offset!(v1_addr, v1.a));
    assert_eq!(raw_offset!(p1_addr, p1.b), volatile_offset!(v1_addr, v1.b));
    assert_eq!(raw_offset!(p1_addr, p1.c), volatile_offset!(v1_addr, v1.c));
    assert_eq!(raw_offset!(p1_addr, p1.d), volatile_offset!(v1_addr, v1.d));
    assert_eq!(
        raw_offset!(p1_addr, p1.b.x),
        volatile_offset!(v1_addr, v1.b.x)
    );
    assert_eq!(
        raw_offset!(p1_addr, p1.b.y),
        volatile_offset!(v1_addr, v1.b.y)
    );
    assert_eq!(
        raw_offset!(p1_addr, p1.c.x),
        volatile_offset!(v1_addr, v1.c.x)
    );
    assert_eq!(
        raw_offset!(p1_addr, p1.c.y),
        volatile_offset!(v1_addr, v1.c.y)
    );

    #[derive(VolatileStruct)]
    #[repr(C)]
    #[repr(C, packed)] // packed(1)
    struct Parent2 {
        a: u8,
        b: Child1,
        c: Child3,
    }

    assert_eq!(std::mem::align_of::<Parent2>(), 1);
    assert_eq!(std::mem::size_of::<Parent2>(), 8);

    let p2 = &Parent2 {
        a: 1,
        b: Child1 { x: 2, y: 3 },
        c: Child3 { x: 4, y: 5 },
    };
    let v2 = Volatile::from_ref(p2);
    let p2_addr = p2 as *const Parent2 as usize;
    let v2_addr = v2.as_ptr().as_ptr() as usize;

    assert_eq!(p2_addr, v2_addr);
    assert_eq!(raw_offset!(p2_addr, p2.a), volatile_offset!(v2_addr, v2.a));
    assert_eq!(raw_offset!(p2_addr, p2.b), volatile_offset!(v2_addr, v2.b));
    assert_eq!(raw_offset!(p2_addr, p2.c), volatile_offset!(v2_addr, v2.c));
    assert_eq!(
        raw_offset!(p2_addr, p2.b.x),
        volatile_offset!(v2_addr, v2.b.x)
    );
    assert_eq!(
        raw_offset!(p2_addr, p2.b.y),
        volatile_offset!(v2_addr, v2.b.y)
    );
    assert_eq!(
        raw_offset!(p2_addr, p2.c.x),
        volatile_offset!(v2_addr, v2.c.x)
    );
    assert_eq!(
        raw_offset!(p2_addr, p2.c.y),
        volatile_offset!(v2_addr, v2.c.y)
    );

    #[derive(VolatileStruct)]
    #[repr(C)]
    #[repr(C, packed(2))]
    struct Parent3 {
        a: u8,
        b: Child1,
        c: Child3,
    }

    assert_eq!(std::mem::align_of::<Parent3>(), 2);
    assert_eq!(std::mem::size_of::<Parent3>(), 10);

    let p3 = &Parent3 {
        a: 1,
        b: Child1 { x: 2, y: 3 },
        c: Child3 { x: 4, y: 5 },
    };
    let v3 = Volatile::from_ref(p3);
    let p3_addr = p3 as *const Parent3 as usize;
    let v3_addr = v3.as_ptr().as_ptr() as usize;

    assert_eq!(p3_addr, v3_addr);
    assert_eq!(raw_offset!(p3_addr, p3.a), volatile_offset!(v3_addr, v3.a));
    assert_eq!(raw_offset!(p3_addr, p3.b), volatile_offset!(v3_addr, v3.b));
    assert_eq!(raw_offset!(p3_addr, p3.c), volatile_offset!(v3_addr, v3.c));
    assert_eq!(
        raw_offset!(p3_addr, p3.b.x),
        volatile_offset!(v3_addr, v3.b.x)
    );
    assert_eq!(
        raw_offset!(p3_addr, p3.b.y),
        volatile_offset!(v3_addr, v3.b.y)
    );
    assert_eq!(
        raw_offset!(p3_addr, p3.c.x),
        volatile_offset!(v3_addr, v3.c.x)
    );
    assert_eq!(
        raw_offset!(p3_addr, p3.c.y),
        volatile_offset!(v3_addr, v3.c.y)
    );
}
