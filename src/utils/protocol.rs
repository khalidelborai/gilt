//! Protocol utilities for third-party crate interoperability.
//!
//! This module provides utilities for checking renderability, casting objects,
//! and converting types to renderable representations. It implements Rust equivalents
//! of Python's `__gilt__` protocol from the Rich library.
//!
//! # The `__gilt__` Protocol
//!
//! In Python's Rich library, objects can implement a `__gilt__` method that returns
//! a renderable representation. This module brings that same concept to Rust through
//! the [`RichCast`] trait.
//!
//! ## Key Types
//!
//! - [`RichCast`] - The core trait for objects that can be converted to renderables.
//!   Implement this on your types to make them printable with gilt.
//! - [`IntoRenderable`] - A conversion trait that allows any `RichCast` type to be
//!   converted to a `Box<dyn Renderable>`.
//! - [`gilt_cast`] - Attempt to downcast a `Box<dyn Any>` to a concrete renderable type.
//! - [`RenderableBox`] - A type-erased wrapper for renderable values.
//!
//! # Examples
//!
//! ```
//! use gilt::protocol::{RichCast, IntoRenderable};
//! use gilt::prelude::*;
//!
//! // Implement RichCast for a custom type
//! struct MyData {
//!     name: String,
//!     value: i32,
//! }
//!
//! impl RichCast for MyData {
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
/// use gilt::protocol::{RichCast, IntoRenderable};
/// use gilt::prelude::*;
///
/// struct User {
///     name: String,
///     email: String,
///     active: bool,
/// }
///
/// impl RichCast for User {
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
pub trait RichCast: Sized + 'static {
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
/// objects. It's automatically implemented for any type implementing [`RichCast`]
/// via a blanket implementation.
///
/// You typically won't need to implement this trait directly - instead, implement
/// [`RichCast`] and get this trait for free.
///
/// # Examples
///
/// ```
/// use gilt::protocol::IntoRenderable;
/// use gilt::prelude::*;
///
/// // Types that implement RichCast also implement IntoRenderable
/// struct Message(String);
///
/// impl gilt::protocol::RichCast for Message {
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

// Blanket implementation: any RichCast type automatically implements IntoRenderable
impl<T: RichCast> IntoRenderable for T {
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

    /// Try to downcast to a concrete type.
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
    /// if let Some(text) = boxed.downcast_ref::<Text>() {
    ///     println!("It's a Text!");
    /// }
    /// ```
    pub fn downcast_ref<T: Renderable + 'static>(&self) -> Option<&T> {
        // We can't downcast dyn Renderable directly, but we can try to
        // get a reference and use Any::downcast_ref if we had stored the type
        // For now, this returns None since we don't store type information
        None
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

/// Macro to derive RichCast implementation (placeholder for future derive macro).
///
/// This macro is a marker for the planned derive macro that will automatically
/// implement `RichCast` for structs and enums. Currently, it does nothing but
/// documents the intended usage.
///
/// # Future Usage
///
/// ```ignore
/// use gilt::protocol::{RichCast, IntoRenderable};
///
/// #[derive(RichCast)]
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

/// Macro to implement RichCast using a closure-like syntax.
///
/// This macro provides a concise way to implement `RichCast` without writing
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
        impl $crate::protocol::RichCast for $type {
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

    // Test RichCast implementation
    struct TestData {
        value: i32,
    }

    impl RichCast for TestData {
        fn __gilt__(self) -> Box<dyn Renderable> {
            Box::new(Panel::new(Text::from(format!("Value: {}", self.value))))
        }
    }

    #[test]
    fn test_gilt_cast_trait() {
        let data = TestData { value: 42 };
        let renderable = data.into_renderable();
        
        // The renderable should be usable
        let mut console = crate::console::Console::builder()
            .width(80)
            .build();
        console.begin_capture();
        console.print(&*renderable);
        let output = console.end_capture();
        
        assert!(output.contains("Value: 42"));
    }

    #[test]
    fn test_into_renderable_blanket_impl() {
        struct SimpleData(&'static str);
        
        impl RichCast for SimpleData {
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
        let mut console = crate::console::Console::builder()
            .width(80)
            .build();
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
        let mut console = crate::console::Console::builder()
            .width(80)
            .build();
        console.begin_capture();
        console.print(&*inner);
        let output = console.end_capture();
        
        assert!(output.contains("Panel content"));
    }

    #[test]
    fn test_renderable_ext() {
        let text = Text::from("Extended");
        let boxed = text.into_boxed_renderable();
        
        let mut console = crate::console::Console::builder()
            .width(80)
            .build();
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
        let mut console = crate::console::Console::builder()
            .width(80)
            .build();
        console.begin_capture();
        console.print(renderable_ref);
        let output = console.end_capture();
        
        assert!(output.contains("Reference"));
    }

    #[test]
    fn test_gilt_cast_impl_macro() {
        struct QuickData { x: i32, y: i32 }
        
        gilt_cast_impl! { QuickData => |p|
            Box::new(Text::from(format!("Point: ({}, {})", p.x, p.y)))
        }
        
        let data = QuickData { x: 10, y: 20 };
        let renderable = data.into_renderable();
        
        let mut console = crate::console::Console::builder()
            .width(80)
            .build();
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
        
        let mut console = crate::console::Console::builder()
            .width(80)
            .build();
        console.begin_capture();
        
        for item in &items {
            console.print(item);
        }
        
        let output = console.end_capture();
        assert!(output.contains("Item 1"));
        assert!(output.contains("Item 2"));
        assert!(output.contains("Item 3"));
    }
}
