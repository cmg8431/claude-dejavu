use anyhow::Result;

pub fn run() -> Result<()> {
    // `init` is now an alias for `install`
    super::install::run()
}
