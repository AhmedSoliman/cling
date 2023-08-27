use crate::anymap::AnyMap;

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
impl<T> CollectableKind for T where
    T: for<'a> CliParam<'a> + Clone + Send + Sync + ?Sized
{
}

pub trait UnknownKind {
    fn as_collectable(&self) -> Uncollectable {
        Uncollectable
    }
}

// Note the type &T here, this means that it's lower priority.
impl<T> UnknownKind for &T {}

#[doc(hidden)]
pub trait CliParam<'a>: Sized {
    fn extract_param(args: &'a CollectedParams) -> Option<Self>;
}

// implementation is per type that derives CliParam
// impl<'a, T> CliParam<'a> for T
// where
//     T: Parser + Send + Sync + Clone + 'static,
// {
//     fn extract_param(args: &'a CollectedParams) -> Option<Self> {
//         args.get::<Self>().cloned()
//     }
// }

/// Returns the value by reference if T implements CliParam!
impl<'a, T> CliParam<'a> for &'a T
where
    T: Sync + Send + 'static,
    T: CliParam<'a> + 'static,
{
    fn extract_param(args: &'a CollectedParams) -> Option<Self> {
        args.get::<T>()
    }
}

impl<'a, T> CliParam<'a> for Vec<T>
where
    T: Sync + Send + 'static,
    T: CliParam<'a> + 'static,
    Vec<T>: Clone,
{
    fn extract_param(args: &'a CollectedParams) -> Option<Self> {
        args.get::<Vec<T>>().cloned()
    }
}

impl<'a, T> CliParam<'a> for Option<T>
where
    T: Sync + Send + 'static,
    T: CliParam<'a> + 'static,
    Option<T>: Clone,
{
    fn extract_param(args: &'a CollectedParams) -> Option<Self> {
        args.get::<Option<T>>().cloned()
    }
}

/// Holds all the arguments collected from the command line during parsing a
/// specific command. It also includes types that are not related to the command
/// line, but are useful to pass around. Such types are helpful to access the
/// environment, context, or other global states that were injected by upper
/// layers.
#[derive(Default)]
pub struct CollectedParams {
    map: Option<AnyMap>,
}

impl CollectedParams {
    #[inline]
    pub fn new() -> Self {
        CollectedParams { map: None }
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map.as_ref().and_then(|map| map.get())
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.map.as_mut().and_then(|map| map.get_mut())
    }

    /// Inserts a value into the collected arguments. If the value already
    /// exists, it will be returned.
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) {
        let res = self.map.get_or_insert_with(Default::default).insert(val);
        if res.is_some() {
            // We have collected the same type twice, we overwrite with the
            // newest value but we must inform the user/dev about it
            // to avoid confusion.
            ::tracing::log::warn!(
                "Collected the same type {} twice while aggregating \
                 parameters. This is usually a sign of a bug in the code. \
                 Either two struct fields in the hierarchy of this command \
                 derive `CliParam` or a field (or more) of the same type is \
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
