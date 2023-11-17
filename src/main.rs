extern crate clap;
extern crate gol;

use clap::{App, Arg, ArgMatches};
use gol::data::Grid;
use gol::rendering;
use std::error::Error;
use std::fmt::Display;
use std::process::exit;
use std::str::FromStr;

fn main() {
    exit(match inner_main() {
        Ok(_) => 0,
        Err(err) => {
            println!("{}", err);
            1
        }
    })
}

fn inner_main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Game of Life")
        .version(version().as_ref())
        .about("Conway's Game of Life in OpenGL!")
        .arg(
            Arg::with_name("grid-width")
                .short("w")
                .long("grid-width")
                .default_value("100")
                .help("Width of the grid"),
        )
        .arg(
            Arg::with_name("grid-height")
                .short("h")
                .long("grid-height")
                .default_value("80")
                .help("Height of the grid"),
        )
        .arg(
            Arg::with_name("window-width")
                .long("window-width")
                .default_value("1024")
                .help("Width of the window"),
        )
        .arg(
            Arg::with_name("window-height")
                .long("window-height")
                .default_value("768")
                .help("Height of the window"),
        )
        .arg(
            Arg::with_name("update-rate")
                .short("u")
                .long("update-rate")
                .default_value("30")
                .help("Number of updates to the game board per second"),
        )
        .get_matches();

    let grid_width = get_number("grid-width", Some(0), &matches);
    let grid_height = get_number("grid-height", Some(0), &matches);
    let window_width = get_number("window-width", Some(0), &matches);
    let window_height = get_number("window-height", Some(0), &matches);
    let updates_per_second = get_number("update-rate", None, &matches);

    let grid = Grid::new(grid_width, grid_height);
    let app = rendering::App::new(grid, window_width, window_height, updates_per_second);
    app?.run()
}

fn version() -> String {
    let (maj, min, pat) = (
        option_env!("CARGO_PKG_VERSION_MAJOR"),
        option_env!("CARGO_PKG_VERSION_MINOR"),
        option_env!("CARGO_PKG_VERSION_PATCH"),
    );
    match (maj, min, pat) {
        (Some(maj), Some(min), Some(pat)) => format!("{}.{}.{}", maj, min, pat),
        _ => "".to_owned(),
    }
}

fn get_number<A>(name: &str, maybe_min: Option<A>, matches: &ArgMatches<'_>) -> A
where
    A: FromStr + PartialOrd + Display + Copy,
    <A as FromStr>::Err: std::fmt::Debug,
{
    matches
        .value_of(name)
        .and_then(|s| s.parse::<A>().ok())
        .and_then(|u| match maybe_min {
            Some(min) => {
                if u > min {
                    Some(u)
                } else {
                    None
                }
            }
            _ => Some(u),
        })
        .expect(
            &{
                if let Some(min) = maybe_min {
                    format!("{} should be a positive number greater than {}.", name, min)
                } else {
                    format!("{} should be a positive number.", name)
                }
            }[..],
        )
}
