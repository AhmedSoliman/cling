use crate::args::CollectedArgs;
use crate::prelude::CliError;

#[async_trait::async_trait]
pub trait CliRunnable: Send + Sync {
    async fn call(&self, args: &mut CollectedArgs) -> Result<(), CliError>;
}
