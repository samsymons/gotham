//! Defines types for passing request state through `Middleware` and `Handler` implementations

mod data;
mod from_state;
pub mod request_id;

use std::collections::HashMap;
use std::any::{Any, TypeId};

pub use state::data::StateData;
pub use state::from_state::FromState;
pub use state::request_id::request_id;
pub use state::request_id::set_request_id;

/// Provides storage for request state, and stores one item of each type. The types used for
/// storage must implement the `gotham::state::StateData` trait to allow its storage.
///
/// Gotham provides `StateData` to ease this implementation via `derive`.
///
/// # Examples
///
/// ```rust
/// extern crate gotham;
/// #[macro_use]
/// extern crate gotham_derive;
///
/// use gotham::state::State;
///
/// #[derive(StateData)]
/// struct MyStruct {
///   value: i32
/// }
///
/// # fn main() {
/// let mut state = State::new();
///
/// state.put(MyStruct { value: 1 });
/// assert_eq!(state.borrow::<MyStruct>().unwrap().value, 1);
/// # }
/// ```
pub struct State {
    data: HashMap<TypeId, Box<Any + Send>>,
}

impl State {
    /// Creates a new, empty `State`
    pub fn new() -> State {
        State { data: HashMap::new() }
    }

    /// Puts a value into the `State` storage. One value of each type is retained. Successive calls
    /// to `put` will overwrite the existing value of the same type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate gotham;
    /// # #[macro_use]
    /// # extern crate gotham_derive;
    /// #
    /// # use gotham::state::State;
    /// #
    /// # #[derive(StateData)]
    /// # struct MyStruct {
    /// #     value: i32
    /// # }
    /// #
    /// # #[derive(StateData)]
    /// # struct AnotherStruct {
    /// #     value: &'static str
    /// # }
    /// #
    /// # fn main() {
    /// # let mut state = State::new();
    /// #
    /// state.put(MyStruct { value: 1 });
    /// assert_eq!(state.borrow::<MyStruct>().unwrap().value, 1);
    ///
    /// state.put(AnotherStruct { value: "a string" });
    /// state.put(MyStruct { value: 100 });
    ///
    /// assert_eq!(state.borrow::<AnotherStruct>().unwrap().value, "a string");
    /// assert_eq!(state.borrow::<MyStruct>().unwrap().value, 100);
    /// # }
    /// ```
    pub fn put<T>(&mut self, t: T)
    where
        T: StateData,
    {
        let type_id = TypeId::of::<T>();
        trace!(" inserting record to state for type_id `{:?}`", type_id);
        self.data.insert(type_id, Box::new(t));
    }

    /// Determines if the current value exists in `State` storage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate gotham;
    /// # #[macro_use]
    /// # extern crate gotham_derive;
    /// #
    /// # use gotham::state::State;
    /// #
    /// # #[derive(StateData)]
    /// # struct MyStruct {
    /// #     value: i32
    /// # }
    /// #
    /// # #[derive(StateData)]
    /// # struct AnotherStruct {
    /// # }
    /// #
    /// # fn main() {
    /// # let mut state = State::new();
    /// #
    /// state.put(MyStruct { value: 1 });
    /// assert!(state.has::<MyStruct>());
    /// assert_eq!(state.borrow::<MyStruct>().unwrap().value, 1);
    ///
    /// assert!(!state.has::<AnotherStruct>());
    /// # }
    /// ```
    pub fn has<T>(&self) -> bool
    where
        T: StateData,
    {
        let type_id = TypeId::of::<T>();
        self.data.get(&type_id).is_some()
    }

    /// Borrows a value from the `State` storage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate gotham;
    /// # #[macro_use]
    /// # extern crate gotham_derive;
    /// #
    /// # use gotham::state::State;
    /// #
    /// # #[derive(StateData)]
    /// # struct MyStruct {
    /// #     value: i32
    /// # }
    /// #
    /// # #[derive(StateData)]
    /// # struct AnotherStruct {
    /// # }
    /// #
    /// # fn main() {
    /// # let mut state = State::new();
    /// #
    /// state.put(MyStruct { value: 1 });
    /// assert!(state.borrow::<MyStruct>().is_some());
    /// assert_eq!(state.borrow::<MyStruct>().unwrap().value, 1);
    ///
    /// assert!(state.borrow::<AnotherStruct>().is_none());
    /// # }
    /// ```
    pub fn borrow<T>(&self) -> Option<&T>
    where
        T: StateData,
    {
        let type_id = TypeId::of::<T>();
        trace!(" borrowing state data for type_id `{:?}`", type_id);
        self.data.get(&type_id).and_then(|b| b.downcast_ref::<T>())
    }

    /// Mutably borrows a value from the `State` storage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate gotham;
    /// # #[macro_use]
    /// # extern crate gotham_derive;
    /// #
    /// # use gotham::state::State;
    /// #
    /// # #[derive(StateData)]
    /// # struct MyStruct {
    /// #     value: i32
    /// # }
    /// #
    /// # #[derive(StateData)]
    /// # struct AnotherStruct {
    /// # }
    /// #
    /// # fn main() {
    /// # let mut state = State::new();
    /// #
    /// state.put(MyStruct { value: 100 });
    ///
    /// {
    ///     let a = state.borrow_mut::<MyStruct>().unwrap();
    ///     a.value += 10;
    /// }
    ///
    /// assert_eq!(state.borrow::<MyStruct>().unwrap().value, 110);
    ///
    /// assert!(state.borrow_mut::<AnotherStruct>().is_none());
    /// # }
    pub fn borrow_mut<T>(&mut self) -> Option<&mut T>
    where
        T: StateData,
    {
        let type_id = TypeId::of::<T>();
        trace!(" mutably borrowing state data for type_id `{:?}`", type_id);
        self.data.get_mut(&type_id).and_then(
            |b| b.downcast_mut::<T>(),
        )
    }

    /// Moves a value out of the `State` storage, and returns ownership.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate gotham;
    /// # #[macro_use]
    /// # extern crate gotham_derive;
    /// #
    /// # use gotham::state::State;
    /// #
    /// # #[derive(StateData)]
    /// # struct MyStruct {
    /// #     value: i32
    /// # }
    /// #
    /// # #[derive(StateData)]
    /// # struct AnotherStruct {
    /// # }
    /// #
    /// # fn main() {
    /// # let mut state = State::new();
    /// #
    /// state.put(MyStruct { value: 110 });
    ///
    /// assert_eq!(state.take::<MyStruct>().unwrap().value, 110);
    ///
    /// assert!(state.take::<MyStruct>().is_none());
    /// assert!(state.borrow_mut::<MyStruct>().is_none());
    /// assert!(state.borrow::<MyStruct>().is_none());
    ///
    /// assert!(state.take::<AnotherStruct>().is_none());
    /// # }
    pub fn take<T>(&mut self) -> Option<T>
    where
        T: StateData,
    {
        let type_id = TypeId::of::<T>();
        trace!(
            " taking ownership from state data for type_id `{:?}`",
            type_id
        );
        self.data
            .remove(&type_id)
            .and_then(|b| b.downcast::<T>().ok())
            .map(|b| *b)
    }
}
