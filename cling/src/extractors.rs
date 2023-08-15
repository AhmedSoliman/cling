use crate::args::{CliParam, CollectedArgs};

/// Extractor for state in handlers
///
/// When a cling program gets executed with a state of type `S`. The state can
/// be accessed via this extractor in any handler. There are a few restrictions
/// on the type `S`. It must implement `Clone` and it must be `Send + Sync`.
/// Typically, this is achieved by wrapping the state in an `Arc<Mutex<_>>`.
///
/// Example:
/// ```rust
/// use cling::prelude::*;
///
/// #[derive(CliParam, ValueEnum, Debug, Clone)]
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
///     colors: Option<Vec<Colors>>,
/// ) {
///     println!( ">> Hello world! color: {:?}, database: {:?}", colors, database);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct State<S: Clone + Send + Sync + 'static>(pub S);

impl<'a, S> CliParam<'a> for State<S>
where
    S: Send + Sync + Clone + 'static,
{
    fn from_args(args: &'a CollectedArgs) -> Option<Self> {
        let Some(state) = args.get::<State<S>>() else {
            return None;
        };
        Some(State(state.0.clone()))
    }
}
