use crate::args::{CliParam, CollectedArgs};

// Extractor for the state
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
