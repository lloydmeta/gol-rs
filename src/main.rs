extern crate gol;
extern crate clap;

use gol::rendering;
use gol::data::Grid;
use clap::{Arg, App, ArgMatches};
use std::str::FromStr;

fn main() {

    let matches = App::new("Game of Life")
        .version(version().as_ref())
        .about("Conway's Game of Life in OpenGL!")
        .arg(Arg::with_name("grid-width")
                 .short("w")
                 .long("grid-width")
                 .default_value("100")
                 .help("Width of the grid"))
        .arg(Arg::with_name("grid-height")
                 .short("h")
                 .long("grid-height")
                 .default_value("80")
                 .help("Height of the grid"))
        .arg(Arg::with_name("window-width")
                 .long("window-width")
                 .default_value("1024")
                 .help("Width of the window"))
        .arg(Arg::with_name("window-height")
                 .long("window-height")
                 .default_value("768")
                 .help("Height of the window"))
        .arg(Arg::with_name("update-rate")
                 .short("u")
                 .long("update-rate")
                 .default_value("30")
                 .help("Number of updates to the game board per second"))
        .get_matches();

    let grid_width = get_positive("grid-width", &matches);
    let grid_height = get_positive("grid-height", &matches);
    let window_width = get_positive("window-width", &matches);
    let window_height = get_positive("window-height", &matches);
    let updates_per_second = get_positive("update-rate", &matches);

    let grid = Grid::new(grid_width, grid_height);
    let mut app = rendering::App::new(grid, window_width, window_height, updates_per_second);
    app.run();
}

fn version() -> String {
    let (maj, min, pat) = (option_env!("CARGO_PKG_VERSION_MAJOR"),
                           option_env!("CARGO_PKG_VERSION_MINOR"),
                           option_env!("CARGO_PKG_VERSION_PATCH"));
    match (maj, min, pat) {
        (Some(maj), Some(min), Some(pat)) => format!("{}.{}.{}", maj, min, pat),
        _ => "".to_owned(),
    }
}

fn get_positive<'a, A>(name: &str, matches: &ArgMatches<'a>) -> A
    where A: FromStr,
          <A as FromStr>::Err: std::fmt::Debug
{
    matches
        .value_of(name)
        .and_then(|s| s.parse::<A>().ok())
        .expect(&format!("{} should be a positive number", name)[..])
}
