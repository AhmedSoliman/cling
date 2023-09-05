use crate::anymap::AnyMap;

// With the hope that one day marker traits attributes
// [marker_trait_attr](https://github.com/rust-lang/rust/issues/29864) will be
// stabilized, we will leave this as a conditional feature on unstable rustc
// until this day comes. In short, this allows us to automatically collect all
// relevant clap types (including Option<T> and Vec<T>) without asking users to
// explicitly mark their clap types with #[derive(Collect)].
#[cfg_attr(unstable, marker)]
pub trait Collect {}

#[cfg(unstable)]
impl<T> Collect for T where T: clap::ValueEnum {}
#[cfg(unstable)]
impl<T> Collect for T where T: clap::Args {}

impl<T> Collect for Option<T> where T: Collect {}
impl<T> Collect for Vec<T> where T: Collect {}

// --
// This is a clever trick inspired by "autref" specialization by
// [dtolnay](https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md) to
// that allows us to selectively choose which types can be collected based on a
// generic bound without the unstable specialization feature.
pub struct Collectable;
impl Collectable {
    pub fn can_collect(&self) -> bool {
        true
    }
}

pub struct Uncollectable;

impl Uncollectable {
    pub fn can_collect(&self) -> bool {
        false
    }
}

pub trait CollectableKind {
    fn as_collectable(&self) -> Collectable {
        Collectable
    }
}

// Does not require autoref is called with &(arg).collectable()
impl<T> CollectableKind for T where T: Collect {}

pub trait UnknownKind {
    fn as_collectable(&self) -> Uncollectable {
        Uncollectable
    }
}

// Note the type &T here, this means that it's lower priority.
impl<T> UnknownKind for &T {}

#[doc(hidden)]
pub trait HandlerParam<'a>: Sized {
    fn extract_param(args: &'a CollectedArgs) -> Option<Self>;
}

/// Blanked implementation that allows handlers to accept shared references to
/// any collectable type.
impl<'a, T> HandlerParam<'a> for &'a T
where
    T: Sync + Send + 'static,
    T: Collect,
{
    fn extract_param(args: &'a CollectedArgs) -> Option<Self> {
        args.get::<T>()
    }
}

/// Holds all the arguments collected from the command line during parsing a
/// specific command. It also includes types that are not related to the command
/// line, but are useful to pass around. Such types are helpful to access the
/// environment, context, or other global states that were injected by upper
/// layers.
#[derive(Default)]
pub struct CollectedArgs {
    map: Option<AnyMap>,
}

impl CollectedArgs {
    #[inline]
    pub fn new() -> Self {
        CollectedArgs { map: None }
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map.as_ref().and_then(|map| map.get())
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.map.as_mut().and_then(|map| map.get_mut())
    }

    /// Inserts a value into the collected arguments. If the value already
    /// exists, it will be returned.
    pub fn insert<T: Send + Sync + 'static>(
        &mut self,
        val: T,
        override_is_expected: bool,
    ) {
        let res = self.map.get_or_insert_with(Default::default).insert(val);
        if res.is_some() && !override_is_expected {
            // We have collected the same type twice, we overwrite with the
            // newest value but we must inform the user/dev about it
            // to avoid confusion.
            ::tracing::log::warn!(
                "Collected the same type {} twice while aggregating \
                 arguments. This is usually a sign of a bug in the code. \
                 Either two struct fields in the hierarchy of this command \
                 derive `Collect` or a field (or more) of the same type is \
                 annotated with `#[cling(collect)].`",
                std::any::type_name::<T>()
            );
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        if let Some(ref mut map) = self.map {
            map.clear();
        }
    }

    pub fn collected_types(&self) -> Vec<String> {
        self.map
            .as_ref()
            .map_or(Vec::new(), |map| map.known_types())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.as_ref().map_or(true, |map| map.is_empty())
    }
}
