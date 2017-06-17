#![feature(test)]

extern crate test;
extern crate gol;

use gol::data::*;
use test::Bencher;

#[bench]
fn grid_50x50_advance_50_times(b: &mut Bencher) {
    let mut grid = Grid::new(50, 50);
    b.iter(|| for _ in 0..50 {
               grid.advance()
           })
}

#[bench]
fn grid_500x500_advance_10_times(b: &mut Bencher) {
    let mut grid = Grid::new(500, 500);
    b.iter(|| for _ in 0..10 {
               grid.advance()
           })
}

#[bench]
fn grid_1000x1000_advance_10_times(b: &mut Bencher) {
    let mut grid = Grid::new(1000, 1000);
    b.iter(|| for _ in 0..10 {
               grid.advance()
           })
}
