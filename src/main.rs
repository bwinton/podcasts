extern crate podcasts;
extern crate rayon;

use rayon::prelude::*;

fn main() {
    let names = vec!["spodcast", "diecast", "vortex_theatre"];
    names.par_iter().for_each(|name| podcasts::handle(name));
}
