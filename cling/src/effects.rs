use crate::params::CollectedArgs;
use crate::prelude::CliError;
use crate::State;

// Internal struct, not meant for public use.
pub struct _Sync;
// Internal struct, not meant for public use.
pub struct _Async;

/// A type returned by handlers to set state for downstream handlers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct SetState<S: Clone + Send + Sync + 'static>(pub S);

/// Defines the handler effect behaviour
pub trait HandlerEffect {
    fn apply_effect(self, args: &mut CollectedArgs);
}

impl HandlerEffect for () {
    fn apply_effect(self, _args: &mut CollectedArgs) {}
}

impl<S> HandlerEffect for SetState<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn apply_effect(self, args: &mut CollectedArgs) {
        args.insert(State(self.0), true)
    }
}

/// Handlers can return any type that implements this trait
pub trait IntoEffect<Type> {
    type Effect: HandlerEffect;

    fn into_effect(self) -> impl std::future::Future<Output = Result<Self::Effect, CliError>> + Send;
}

impl IntoEffect<_Sync> for () {
    type Effect = ();

    async fn into_effect(self) -> Result<(), CliError> {
        Ok(())
    }
}

impl<S> IntoEffect<_Sync> for State<S>
where
    S: Clone + Send + Sync + 'static,
{
    type Effect = SetState<S>;

    async fn into_effect(self) -> Result<Self::Effect, CliError> {
        Ok(SetState(self.0))
    }
}

impl<S> IntoEffect<_Sync> for SetState<S>
where
    S: Clone + Send + Sync + 'static,
{
    type Effect = SetState<S>;

    async fn into_effect(self) -> Result<Self::Effect, CliError> {
        Ok(self)
    }
}

impl<E, F> IntoEffect<_Sync> for Result<F, E>
where
    E: Into<CliError>,
    F: IntoEffect<_Sync> + Send,
    Self: Send,
{
    type Effect = F::Effect;

    async fn into_effect(self) -> Result<Self::Effect, CliError> {
        match self {
            | Ok(f) => f.into_effect().await,
            | Err(e) => Err(e.into()),
        }
    }
}

/// Adaptor to allow async handlers as long as their return type is also
/// [IntoEffect]
impl<T, Output> IntoEffect<_Async> for T
where
    T: std::future::Future<Output = Output> + Send,
    Output: IntoEffect<_Sync> + Send,
{
    type Effect = Output::Effect;

    async fn into_effect(self) -> Result<Self::Effect, CliError> {
        self.await.into_effect().await
    }
}
