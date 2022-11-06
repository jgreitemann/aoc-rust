pub(crate) use clap::Parser;

#[derive(Debug, Default, Clone, Parser)]
pub(crate) struct Options {
    #[arg(short, long)]
    pub empty_cache: bool,
    #[arg(short, long)]
    pub skip_solved: bool,
}
