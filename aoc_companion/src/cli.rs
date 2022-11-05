pub(crate) use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Options {
    #[arg(short, long)]
    pub empty_cache: bool,
}
