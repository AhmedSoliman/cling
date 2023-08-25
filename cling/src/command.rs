use crate::params::CollectedParams;
use crate::prelude::CliError;

#[doc(hidden)]
#[async_trait::async_trait]
pub trait CliRunnable: Send + Sync {
    async fn call(&self, args: &mut CollectedParams) -> Result<(), CliError>;
}
