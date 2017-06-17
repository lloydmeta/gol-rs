#![feature(test)]

extern crate test;
extern crate gol;

use gol::data::*;
use gol::rendering::*;
use test::Bencher;

#[bench]
fn render_50x50_grid_10times(b: &mut Bencher) {
    let grid = Grid::new(50, 50);
    let mut app = App::new(grid, 1024, 768, 30);
    b.iter(|| for _ in 0..10 {
               app.render()
           })
}

#[bench]
fn render_500x500_grid_10times(b: &mut Bencher) {
    let grid = Grid::new(500, 500);
    let mut app = App::new(grid, 1024, 768, 30);
    b.iter(|| for _ in 0..10 {
               app.render()
           })
}

#[bench]
fn render_1000x1000_grid_10times(b: &mut Bencher) {
    let grid = Grid::new(1000, 1000);
    let mut app = App::new(grid, 1024, 768, 30);
    b.iter(|| for _ in 0..10 {
               app.render()
           })
}
