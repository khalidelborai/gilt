//! Protocol utilities for third-party crate interoperability.
//!
//! This module provides utilities for checking renderability, casting objects,
//! and converting types to renderable representations. It implements Rust equivalents
//! of Python's `__gilt__` protocol from the library.
//!
//! # The `__gilt__` Protocol
//!
//! In Python's library, objects can implement a `__gilt__` method that returns
//! a renderable representation. This module brings that same concept to Rust through
//! the [`GiltCast`] trait.
//!
//! ## Key Types
//!
//! - [`GiltCast`] - The core trait for objects that can be converted to renderables.
//!   Implement this on your types to make them printable with gilt.
//! - [`IntoRenderable`] - A conversion trait that allows any `GiltCast` type to be
//!   converted to a `Box<dyn Renderable>`.
//! - [`gilt_cast`] - Attempt to downcast a `Box<dyn Any>` to a concrete renderable type.
//! - [`RenderableBox`] - A type-erased wrapper for renderable values.
//!
//! # Examples
//!
//! ```
//! use gilt::protocol::{GiltCast, IntoRenderable};
//! use gilt::prelude::*;
//!
//! // Implement GiltCast for a custom type
//! struct MyData {
//!     name: String,
//!     value: i32,
//! }
//!
//! impl GiltCast for MyData {
//!     fn __gilt__(self) -> Box<dyn gilt::console::Renderable> {
//!         let text = Text::from(format!("{} = {}", self.name, self.value));
//!         Box::new(Panel::new(text))
//!     }
//! }
//!
//! // Now MyData can be converted to a renderable
//! let data = MyData { name: "count".into(), value: 42 };
//! let renderable = data.into_renderable();
//! ```

use std::any::Any;

use crate::console::Renderable;

/// Attempt to cast a `Box<dyn Any>` to a concrete renderable type.
///
/// This function is useful when you have a boxed `Any` trait object and want to
/// downcast it to a specific renderable type. It returns `None` if the type
/// doesn't match.
///
/// # Type Parameters
///
/// * `T` - The concrete renderable type to cast to. Must implement `Renderable`.
///
/// # Parameters
///
/// * `value` - The boxed `Any` value to cast.
///
/// # Returns
///
/// Returns `Some(Box<T>)` if the cast succeeds, `None` otherwise.
///
/// # Examples
///
/// ```
/// use gilt::protocol::gilt_cast;
/// use gilt::prelude::*;
///
/// // Create a boxed Text
/// let text: Box<dyn std::any::Any> = Box::new(Text::from("Hello"));
///
/// // Try to cast it back to Text
/// if let Some(text) = gilt_cast::<Text>(text) {
///     println!("Successfully cast to Text");
/// }
///
/// // Trying to cast to wrong type returns None
/// let text: Box<dyn std::any::Any> = Box::new(Text::from("Hello"));
/// assert!(gilt_cast::<Panel>(text).is_none());
/// ```
pub fn gilt_cast<T: Renderable + 'static>(value: Box<dyn Any>) -> Option<Box<T>> {
    value.downcast::<T>().ok()
}

/// Check if a value is a specific renderable type.
///
/// This is a convenience function that attempts to downcast and returns
/// a boolean indicating success.
///
/// # Examples
///
/// ```
/// use gilt::protocol::{is_type, gilt_cast};
/// use gilt::prelude::*;
///
/// let text: Box<dyn std::any::Any> = Box::new(Text::from("Hello"));
/// assert!(is_type::<Text>(&*text));
/// assert!(!is_type::<Panel>(&*text));
/// ```
pub fn is_type<T: 'static>(value: &dyn Any) -> bool {
    value.is::<T>()
}

/// Trait for types that can be converted to a renderable representation.
///
/// This is the Rust equivalent of Python's `__gilt__` protocol. Implement this
/// trait on your custom types to make them convertible to renderable widgets.
///
/// Once implemented, your type automatically implements [`IntoRenderable`] and
/// can be converted to a `Box<dyn Renderable>`.
///
/// # Examples
///
/// ```
/// use gilt::protocol::{GiltCast, IntoRenderable};
/// use gilt::prelude::*;
///
/// struct User {
///     name: String,
///     email: String,
///     active: bool,
/// }
///
/// impl GiltCast for User {
///     fn __gilt__(self) -> Box<dyn gilt::console::Renderable> {
///         let status = if self.active { "✓ Active" } else { "✗ Inactive" };
///         let content = Text::from(format!(
///             "Name: {}\nEmail: {}\nStatus: {}",
///             self.name, self.email, status
///         ));
///         Box::new(Panel::new(content).with_title("User Profile"))
///     }
/// }
///
/// let user = User {
///     name: "Alice".into(),
///     email: "alice@example.com".into(),
///     active: true,
/// };
///
/// // Convert to renderable and print
/// let renderable = user.into_renderable();
/// ```
pub trait GiltCast: Sized + 'static {
    /// Convert this value to a renderable representation.
    ///
    /// This method should create a widget (like a [`Panel`](crate::panel::Panel),
    /// [`Table`](crate::table::Table), or [`Text`](crate::text::Text)) that
    /// represents this value visually.
    ///
    /// # Returns
    ///
    /// A boxed trait object implementing `Renderable`.
    fn __gilt__(self) -> Box<dyn Renderable>;
}

/// Trait for types that can be converted into a `Box<dyn Renderable>`.
///
/// This trait provides a uniform way to convert various types into renderable
/// objects. It's automatically implemented for any type implementing [`GiltCast`]
/// via a blanket implementation.
///
/// You typically won't need to implement this trait directly - instead, implement
/// [`GiltCast`] and get this trait for free.
///
/// # Examples
///
/// ```
/// use gilt::protocol::IntoRenderable;
/// use gilt::prelude::*;
///
/// // Types that implement GiltCast also implement IntoRenderable
/// struct Message(String);
///
/// impl gilt::protocol::GiltCast for Message {
///     fn __gilt__(self) -> Box<dyn gilt::console::Renderable> {
///         Box::new(Panel::new(Text::from(self.0)))
///     }
/// }
///
/// let msg = Message("Hello, World!".into());
/// let renderable = msg.into_renderable();
/// ```
pub trait IntoRenderable {
    /// Convert this value into a boxed renderable.
    ///
    /// # Returns
    ///
    /// A `Box<dyn Renderable>` that can be passed to console methods.
    fn into_renderable(self) -> Box<dyn Renderable>;
}

// Blanket implementation: any GiltCast type automatically implements IntoRenderable
impl<T: GiltCast> IntoRenderable for T {
    fn into_renderable(self) -> Box<dyn Renderable> {
        self.__gilt__()
    }
}

/// Extension trait for types that implement `Renderable`.
///
/// This trait provides convenience methods for working with renderable values.
/// Since it requires `Renderable` as a bound, it will only be implemented
/// for types that are actually renderable.
///
/// # Examples
///
/// ```
/// use gilt::protocol::RenderableExt;
/// use gilt::prelude::*;
///
/// // Wrap a Text value
/// let text = Text::from("Hello");
/// let boxed = text.into_boxed_renderable();
/// ```
pub trait RenderableExt: Renderable + Sized + 'static {
    /// Convert this renderable into a `RenderableBox` for type-erased storage.
    ///
    /// # Returns
    ///
    /// A `RenderableBox` wrapping this value.
    fn into_boxed_renderable(self) -> RenderableBox;
}

impl<T: Renderable + 'static> RenderableExt for T {
    fn into_boxed_renderable(self) -> RenderableBox {
        RenderableBox::new(self)
    }
}

// ---------------------------------------------------------------------------
// CastWrapper — bridges GiltCast → Renderable without a blanket impl
// ---------------------------------------------------------------------------

/// A newtype that wraps a [`GiltCast`] value and exposes it as a [`Renderable`].
///
/// # Why a newtype instead of a blanket impl?
///
/// A blanket `impl<T: GiltCast> Renderable for T` is rejected by the Rust
/// coherence checker because nothing prevents a downstream crate (or the
/// codebase itself) from implementing *both* `GiltCast` and `Renderable` on
/// the same type — which would create an overlapping implementation. The
/// newtype sidesteps coherence: `CastWrapper<T>` is a distinct type that has
/// exactly one `Renderable` impl, with no risk of overlap.
///
/// `CastWrapper` stores its value in a `RefCell<Option<T>>`. The first time
/// `gilt_console` is called it moves the value out and invokes `__gilt__`,
/// delegating to the resulting [`Renderable`]. Subsequent calls will produce
/// an empty segment list (the value has been consumed). This matches Python's
/// `__rich__` protocol: an object is rendered at most once per print call.
///
/// # Usage
///
/// ```
/// use gilt::protocol::{GiltCast, CastWrapper};
/// use gilt::prelude::*;
///
/// struct MyData(String);
///
/// impl GiltCast for MyData {
///     fn __gilt__(self) -> Box<dyn gilt::console::Renderable> {
///         Box::new(Text::from(self.0))
///     }
/// }
///
/// let wrapper = CastWrapper::new(MyData("hello".into()));
/// let mut console = Console::builder().width(80).build();
/// console.begin_capture();
/// console.print(&wrapper);
/// let out = console.end_capture();
/// assert!(out.contains("hello"));
/// ```
///
/// Alternatively, call [`Console::print_cast`] which constructs the wrapper
/// for you:
///
/// ```
/// use gilt::protocol::GiltCast;
/// use gilt::prelude::*;
///
/// struct Msg(String);
/// impl GiltCast for Msg {
///     fn __gilt__(self) -> Box<dyn gilt::console::Renderable> {
///         Box::new(Text::from(self.0))
///     }
/// }
///
/// let mut console = Console::builder().width(80).build();
/// console.begin_capture();
/// console.print_cast(Msg("world".into()));
/// let out = console.end_capture();
/// assert!(out.contains("world"));
/// ```
pub struct CastWrapper<T: GiltCast> {
    inner: std::cell::RefCell<Option<T>>,
}

impl<T: GiltCast> CastWrapper<T> {
    /// Wrap a [`GiltCast`] value so it can be passed to `Console::print`.
    pub fn new(value: T) -> Self {
        Self {
            inner: std::cell::RefCell::new(Some(value)),
        }
    }
}

impl<T: GiltCast> Renderable for CastWrapper<T> {
    fn gilt_console(
        &self,
        console: &crate::console::Console,
        options: &crate::console::ConsoleOptions,
    ) -> Vec<crate::segment::Segment> {
        // Take ownership of the wrapped value (consumes it on first call).
        // `gilt_console` takes `&self` so we use `RefCell` for interior
        // mutability. Subsequent calls after the value has been taken return
        // an empty segment list, mirroring Python's single-render protocol.
        let maybe_val = self.inner.borrow_mut().take();
        match maybe_val {
            Some(val) => {
                let boxed: Box<dyn Renderable> = val.__gilt__();
                boxed.gilt_console(console, options)
            }
            None => vec![],
        }
    }
}

/// A type-erased wrapper that can hold any renderable value.
///
/// This struct wraps a `Box<dyn Renderable>` and can be used when you need
/// to store renderable values in a type-erased context while still being
/// able to use them as renderables.
///
/// # Examples
///
/// ```
/// use gilt::protocol::{RenderableBox, RenderableExt};
/// use gilt::prelude::*;
///
/// let text = Text::from("Hello");
/// let boxed = text.into_boxed_renderable();
///
/// // boxed can now be stored in collections or passed around
/// let items: Vec<RenderableBox> = vec![boxed];
/// ```
pub struct RenderableBox {
    inner: Box<dyn Renderable>,
}

impl RenderableBox {
    /// Create a new RenderableBox from any renderable value.
    pub fn new<R: Renderable + 'static>(renderable: R) -> Self {
        Self {
            inner: Box::new(renderable),
        }
    }

    /// Get a reference to the inner renderable.
    pub fn as_renderable(&self) -> &dyn Renderable {
        &*self.inner
    }

    /// Convert back into a boxed renderable.
    pub fn into_inner(self) -> Box<dyn Renderable> {
        self.inner
    }
}

impl Renderable for RenderableBox {
    fn gilt_console(
        &self,
        console: &crate::console::Console,
        options: &crate::console::ConsoleOptions,
    ) -> Vec<crate::segment::Segment> {
        self.inner.gilt_console(console, options)
    }
}

/// Attempt to cast a reference to a renderable trait object.
///
/// This is similar to `gilt_cast` but works with references instead of owned values.
///
/// # Examples
///
/// ```
/// use gilt::protocol::as_renderable_ref;
/// use gilt::prelude::*;
///
/// let text = Text::from("Hello");
/// let renderable = as_renderable_ref(&text);
/// // renderable is &dyn Renderable
/// // Use renderable here
/// ```
pub fn as_renderable_ref<T: Renderable>(value: &T) -> &dyn Renderable {
    value
}

/// Attempt to cast a mutable reference to a renderable trait object.
///
/// # Examples
///
/// ```
/// use gilt::protocol::as_renderable_mut;
/// use gilt::prelude::*;
///
/// let mut text = Text::from("Hello");
/// let renderable = as_renderable_mut(&mut text);
/// ```
pub fn as_renderable_mut<T: Renderable>(value: &mut T) -> &mut dyn Renderable {
    value
}

/// Macro to derive GiltCast implementation (placeholder for future derive macro).
///
/// This macro is a marker for the planned derive macro that will automatically
/// implement `GiltCast` for structs and enums. Currently, it does nothing but
/// documents the intended usage.
///
/// # Future Usage
///
/// ```ignore
/// use gilt::protocol::{GiltCast, IntoRenderable};
///
/// #[derive(GiltCast)]
/// #[rich(panel)]
/// struct User {
///     name: String,
///     email: String,
/// }
/// ```
#[macro_export]
macro_rules! derive_gilt_cast {
    // Placeholder for future derive macro
    ($item:item) => {
        $item
    };
}

/// Macro to implement GiltCast using a closure-like syntax.
///
/// This macro provides a concise way to implement `GiltCast` without writing
/// out the full impl block. The syntax uses a closure pattern where you specify
/// a parameter name for `self`.
///
/// # Examples
///
/// ```
/// use gilt::gilt_cast_impl;
/// use gilt::prelude::*;
///
/// struct Status { code: u16, message: String }
///
/// gilt_cast_impl! { Status => |s|
///     Box::new(Panel::new(Text::from(format!("Status {}: {}",
///         s.code, s.message))))
/// }
/// ```
#[macro_export]
macro_rules! gilt_cast_impl {
    ($type:ty => |$this:ident| $body:expr) => {
        impl $crate::protocol::GiltCast for $type {
            fn __gilt__(self) -> Box<dyn $crate::console::Renderable> {
                let $this = self;
                $body
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_gilt_cast_success() {
        let text = Text::from("Hello, World!");
        let boxed: Box<dyn Any> = Box::new(text);

        let cast_result = gilt_cast::<Text>(boxed);
        assert!(cast_result.is_some());
    }

    #[test]
    fn test_gilt_cast_failure() {
        let text = Text::from("Hello");
        let boxed: Box<dyn Any> = Box::new(text);

        // Trying to cast Text to Panel should fail
        let cast_result = gilt_cast::<Panel>(boxed);
        assert!(cast_result.is_none());
    }

    #[test]
    fn test_gilt_cast_with_panel() {
        let panel = Panel::new(Text::from("Content"));
        let boxed: Box<dyn Any> = Box::new(panel);

        let cast_result = gilt_cast::<Panel>(boxed);
        assert!(cast_result.is_some());
    }

    // Test GiltCast implementation
    struct TestData {
        value: i32,
    }

    impl GiltCast for TestData {
        fn __gilt__(self) -> Box<dyn Renderable> {
            Box::new(Panel::new(Text::from(format!("Value: {}", self.value))))
        }
    }

    #[test]
    fn test_gilt_cast_trait() {
        let data = TestData { value: 42 };
        let renderable = data.into_renderable();

        // The renderable should be usable
        let mut console = crate::console::Console::builder().width(80).build();
        console.begin_capture();
        console.print(&*renderable);
        let output = console.end_capture();

        assert!(output.contains("Value: 42"));
    }

    #[test]
    fn test_into_renderable_blanket_impl() {
        struct SimpleData(&'static str);

        impl GiltCast for SimpleData {
            fn __gilt__(self) -> Box<dyn Renderable> {
                Box::new(Text::from(self.0))
            }
        }

        let data = SimpleData("Test");
        let _renderable: Box<dyn Renderable> = data.into_renderable();
        // If this compiles, the blanket implementation works
    }

    #[test]
    fn test_renderable_box() {
        let text = Text::from("Boxed text");
        let boxed = RenderableBox::new(text);

        // Can be used as a renderable
        let mut console = crate::console::Console::builder().width(80).build();
        console.begin_capture();
        console.print(&boxed);
        let output = console.end_capture();

        assert!(output.contains("Boxed text"));
    }

    #[test]
    fn test_renderable_box_from_panel() {
        let panel = Panel::new(Text::from("Panel content"));
        let boxed = RenderableBox::new(panel);

        let inner = boxed.into_inner();
        // inner is Box<dyn Renderable>
        let mut console = crate::console::Console::builder().width(80).build();
        console.begin_capture();
        console.print(&*inner);
        let output = console.end_capture();

        assert!(output.contains("Panel content"));
    }

    #[test]
    fn test_renderable_ext() {
        let text = Text::from("Extended");
        let boxed = text.into_boxed_renderable();

        let mut console = crate::console::Console::builder().width(80).build();
        console.begin_capture();
        console.print(&boxed);
        let output = console.end_capture();

        assert!(output.contains("Extended"));
    }

    #[test]
    fn test_is_type() {
        let text: Box<dyn Any> = Box::new(Text::from("Test"));
        assert!(is_type::<Text>(&*text));
        assert!(!is_type::<Panel>(&*text));
    }

    #[test]
    fn test_as_renderable_ref() {
        let text = Text::from("Reference");
        let renderable_ref = as_renderable_ref(&text);

        // Should be usable as a renderable reference
        let mut console = crate::console::Console::builder().width(80).build();
        console.begin_capture();
        console.print(renderable_ref);
        let output = console.end_capture();

        assert!(output.contains("Reference"));
    }

    #[test]
    fn test_gilt_cast_impl_macro() {
        struct QuickData {
            x: i32,
            y: i32,
        }

        gilt_cast_impl! { QuickData => |p|
            Box::new(Text::from(format!("Point: ({}, {})", p.x, p.y)))
        }

        let data = QuickData { x: 10, y: 20 };
        let renderable = data.into_renderable();

        let mut console = crate::console::Console::builder().width(80).build();
        console.begin_capture();
        console.print(&*renderable);
        let output = console.end_capture();

        assert!(output.contains("Point: (10, 20)"));
    }

    #[test]
    fn test_collection_of_boxes() {
        let items: Vec<RenderableBox> = vec![
            RenderableBox::new(Text::from("Item 1")),
            RenderableBox::new(Panel::new(Text::from("Item 2"))),
            RenderableBox::new(Rule::with_title("Item 3")),
        ];

        let mut console = crate::console::Console::builder().width(80).build();
        console.begin_capture();

        for item in &items {
            console.print(item);
        }

        let output = console.end_capture();
        assert!(output.contains("Item 1"));
        assert!(output.contains("Item 2"));
        assert!(output.contains("Item 3"));
    }

    // ── Item 1: GiltCast auto-invoked via CastWrapper / print_cast ──────────

    /// A type implementing only GiltCast (not Renderable directly).
    struct MyWidget {
        label: String,
        count: usize,
    }

    impl GiltCast for MyWidget {
        fn __gilt__(self) -> Box<dyn Renderable> {
            Box::new(Text::from(format!("{}: {}", self.label, self.count)))
        }
    }

    #[test]
    fn test_cast_wrapper_renders_gilt_cast_type() {
        // CastWrapper wraps a GiltCast value and implements Renderable.
        let w = CastWrapper::new(MyWidget {
            label: "hits".into(),
            count: 99,
        });
        let mut console = crate::console::Console::builder()
            .width(80)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print(&w);
        let output = console.end_capture();
        assert!(
            output.contains("hits: 99"),
            "CastWrapper must render via __gilt__; got: {output:?}"
        );
    }

    #[test]
    fn test_print_cast_renders_gilt_cast_type() {
        // Console::print_cast is the ergonomic shorthand.
        let mut console = crate::console::Console::builder()
            .width(80)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print_cast(MyWidget {
            label: "events".into(),
            count: 7,
        });
        let output = console.end_capture();
        assert!(
            output.contains("events: 7"),
            "print_cast must render via __gilt__; got: {output:?}"
        );
    }

    #[test]
    fn test_cast_wrapper_coherence_no_conflict() {
        // Compile-time proof: a type that implements only GiltCast can be
        // wrapped without conflicting with any existing Renderable impl.
        struct OnlyGiltCast(String);
        impl GiltCast for OnlyGiltCast {
            fn __gilt__(self) -> Box<dyn Renderable> {
                Box::new(Text::from(self.0))
            }
        }
        let w = CastWrapper::new(OnlyGiltCast("unique".into()));
        let mut console = crate::console::Console::builder()
            .width(80)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print(&w);
        let output = console.end_capture();
        assert!(output.contains("unique"));
    }
}
