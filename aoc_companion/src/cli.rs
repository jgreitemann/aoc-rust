use clap::value_parser;
pub(crate) use clap::Parser;

#[derive(Debug, Default, Clone, Parser)]
pub(crate) struct Options {
    #[arg(
        short,
        long,
        help = "Empty the cache",
        long_help = "Empty the cache, requiring inputs and correct results to be fetched anew"
    )]
    pub empty_cache: bool,
    #[arg(short, long, help = "Do not recompute answer for solved problems")]
    pub skip_solved: bool,
    #[arg(short, long, value_parser = value_parser!(u32).range(1..=25), help="Only solve problems for the specified day")]
    pub day: Option<u32>,
}
