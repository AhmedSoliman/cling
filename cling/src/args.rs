use crate::anymap::AnyMap;

pub trait FromCliArgs<'a>: Sized {
    fn from_args(args: &'a CollectedArgs) -> Option<Self>;
}

/// Returns the value by reference if T implements FromCliArgs!
impl<'a, T> FromCliArgs<'a> for &'a T
where
    T: Sync + Send + 'static,
    T: FromCliArgs<'a>,
{
    fn from_args(args: &'a CollectedArgs) -> Option<Self> {
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
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.map.get_or_insert_with(Default::default).insert(val)
    }

    #[inline]
    pub fn clear(&mut self) {
        if let Some(ref mut map) = self.map {
            map.clear();
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.as_ref().map_or(true, |map| map.is_empty())
    }
}
