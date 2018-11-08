use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Ctx {
    pub logger: slog::Logger,
    pub dst_folder: PathBuf,
    pub src_uri: String,
}

pub fn process(ctx: Ctx) -> Result<(), Box<Error>> {
    // TODO download templates if it not local
    // TODO list actions to execute
    // TODO executes actions
    unimplemented!()
}
