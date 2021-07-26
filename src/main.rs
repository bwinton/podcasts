extern crate podcasts;
extern crate rayon;

use color_eyre::Report;
use rayon::prelude::*;

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let names = vec!["spodcast", "diecast", "vortex_theatre"];
    names.par_iter().for_each(|name| podcasts::handle(name));
    Ok(())
}
