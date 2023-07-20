pub trait FromCliArgs: Sized {
    fn from_args(args: &mut CollectedArgs) -> Option<Self>;
}

pub struct CollectedArgs {}

impl CollectedArgs {
    pub fn get<T>(&self) -> Option<T> {
        todo!()
    }
}
