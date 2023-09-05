use crate::params::{CollectedArgs, HandlerParam};

/// Extractor for state in handlers
///
/// When a cling program gets executed with a state of type `S`. The state can
/// be accessed via this extractor in any handler.
///
/// Type `S` must implement [Clone] as it gets cloned every time this extractor
/// runs.
///
/// Example:
/// ```rust
/// use cling::prelude::*;
///
/// #[derive(Collect, ValueEnum, Debug, Clone)]
/// pub enum Colors {
///     Red,
///     Green,
///     Blue,
/// }
///
/// #[derive(Clone, Debug)]
/// struct Database {
///     _data: String,
/// }
///
/// async fn init(
///     State(database): State<Database>,
///     colors: &Option<Vec<Colors>>,
/// ) {
///     println!( ">> Hello world! color: {:?}, database: {:?}", colors, database);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct State<S: Clone + Send + Sync + 'static>(pub S);

impl<'a, S> HandlerParam<'a> for State<S>
where
    S: Send + Sync + Clone + 'static,
{
    fn extract_param(args: &'a CollectedArgs) -> Option<Self> {
        let Some(state) = args.get::<State<S>>() else {
            return None;
        };
        Some(State(state.0.clone()))
    }
}

#[derive(Clone, Debug)]
/// An extractor for fields annotated with `#[cling(collect)]`
pub struct Collected<T>(pub T);

impl<'a, T> HandlerParam<'a> for Collected<T>
where
    T: Send + Sync + Clone + 'static,
{
    fn extract_param(args: &'a CollectedArgs) -> Option<Self> {
        args.get::<Self>().cloned()
    }
}
