# zst-volatile

## Warning

This is work in progress.

## Usage

Take a look in the tests directory for an example.

## Explanation

### Offset and Alignment Restrictions

`Volatile` is a zero sized wrapper.

`Volatile<T, O>` has three generics:
1. `T` is that datatype of the pointer.
2. `O` is an offset from a pointer encoded as a type.
2. `A` is a type that provides a GAT to lower the alignment of types if required. This is needed for packed structs.

`Volatile::<T, O>::read` calculates the real pointer by offsetting `self` by `O` bytes. This allows having multiple zero sized volatile references pointing to the same address while still maintaining the information about where the reference actually points.
The `A` parameter is used to wrap `T` in a type that has lower or equal alignment requirements than `T`.

### Structs

For each struct that derives `VolatileStruct`, we generate another struct with the same fields, but using the `Volatile` wrapper.

As an example take this struct:
```rust
#[derive(VolatileStruct)]
#[repr(C)]
pub struct Child1 {
    field1: u32,
    field2: u32,
    field3: u32,
    field4: u32,
}
```

The `VolatileStruct` derive macro expands this into:
```rust
pub struct VolatileChild1<A> {
    field1: Volatile<u32, offset::Align<offset::Zero, u32>, A>,
    field2: Volatile<u32, offset::Align<offset::PastField<offset::Zero, u32>, u32>, A>,
    field3: Volatile<u32, offset::Align<offset::PastField<offset::PastField<offset::Zero, u32>, u32>, u32>, A>,
    field4: Volatile<u32, offset::Align<offset::PastField<offset::PastField<offset::PastField<offset::Zero, u32>, u32>, u32, >, u32>, A>,
}
```

We also generate an implementation for `VolatileStruct`.
```rust
unsafe impl VolatileStruct for Child1 {
    type Struct<A> = VolatileChild1<A>;
}
```

This `VolatileStruct` implementation is used with the `Deref` & `DerefMut` impls that allow `Volatile<Struct>` to decompose into `VolatileStruct`. This is what allows accessing fields of children directly.

```rust
impl<T, O> Deref for Volatile<T, O>
where
    T: VolatileStruct,
    O: Offset,
    A: Alignment,
{
    type Target = T::Struct<A>;

    fn deref(&self) -> &Self::Target {
        // ...
    }
}

impl<T, O> DerefMut for Volatile<T, O>
where
    T: VolatileStruct,
    O: Offset,
    A: Alignment,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // ...
    }
}
```

rust-analyzer can cope suprisingly well with this offering code completitions even to nested fields.