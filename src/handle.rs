//! Typed handle returned by [`Dashboard::add_df`](crate::Dashboard::add_df) for
//! referencing registered `DataFrame`s in chart, filter, and table specs.
//!
//! Using a handle instead of a bare string key makes reference typos a compile
//! error and enables IDE rename refactors. The handle carries the string name
//! internally so the wire format to the Python/native renderer is unchanged.

use std::sync::Arc;

/// Typed reference to a `DataFrame` registered with
/// [`Dashboard::add_df`](crate::Dashboard::add_df).
///
/// Pass it to chart, filter, and table builders in place of the registration
/// key. Cloning is cheap (one atomic increment on the interned name).
#[derive(Clone, Debug)]
pub struct DfHandle {
    #[allow(dead_code)]
    pub(crate) id: u32,
    pub(crate) name: Arc<str>,
}

impl DfHandle {
    /// Construct a handle from a raw name without going through a
    /// [`Dashboard`](crate::Dashboard).
    ///
    /// Prefer [`Dashboard::add_df`](crate::Dashboard::add_df), which also
    /// registers the `DataFrame`. This constructor exists for tests and for
    /// advanced use cases that assemble [`ChartSpec`](crate::charts::ChartSpec)s
    /// outside the dashboard builder.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            id: u32::MAX,
            name: Arc::from(name),
        }
    }

    /// String name this handle refers to — matches the key registered with
    /// [`Dashboard::add_df`](crate::Dashboard::add_df).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl PartialEq for DfHandle {
    fn eq(&self, other: &Self) -> bool {
        *self.name == *other.name
    }
}

impl Eq for DfHandle {}
