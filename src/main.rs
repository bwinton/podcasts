extern crate podcasts;
extern crate rayon;

use rayon::prelude::*;

fn main() {
    let names = vec!["spodcast", "diecast"];
    names.par_iter().for_each(|name| podcasts::handle(name));
}
