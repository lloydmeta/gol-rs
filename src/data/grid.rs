use data::cell::{Cell, Status};
use rand;
use rand::Rng;

#[derive(Debug)]
pub struct Grid {
    /* Addressed by from-zero (i, j) notation, where i is row number, j is column number
     * such that given the following shows coordinates for cells in a 3 x 3 grid:
     *
     * [ (0,0) (0,1) (0,2) ]
     * [ (1,0) (1,1) (1,2) ]
     * [ (2,0) (2,1) (2,2) ]
     */
    cells: Vec<Vec<Cell>>,
    max_i: usize,
    max_j: usize,
    coords_with_neighbours: Vec<CoordNeighbours>,
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Clone)]
struct Coord {
    i: usize,
    j: usize,
}

#[derive(PartialEq, Eq, Debug)]
struct CoordNeighbours {
    coord: Coord,
    neighbours: [Coord; 8],
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Grid {
        let mut rng = rand::thread_rng();
        let mut cells = Vec::with_capacity(height);
        for _ in 0..height {
            let mut columns = Vec::with_capacity(width);
            for _ in 0..width {
                let status = if rng.gen() {
                    Status::Alive
                } else {
                    Status::Dead
                };
                let cell = Cell(status);
                columns.push(cell);
            }
            cells.push(columns);
        }
        let (max_i, max_j) = max_coordinates(&cells);
        let area = width * height;
        let coords_with_neighbours = coords_with_neighbours(max_i, max_j, area, &cells);
        Grid {
            cells,
            max_i,
            max_j,
            coords_with_neighbours,
        }
    }

    // Returns a read only vector with references to this grid's cells
    pub fn cells(&self) -> Vec<Vec<&Cell>> {
        self.cells
            .iter()
            .map(|r| r.iter().map(|c| c).collect())
            .collect()
    }

    pub fn height(&self) -> usize {
        self.max_i + 1
    }

    pub fn width(&self) -> usize {
        self.max_j + 1
    }

    pub fn area(&self) -> usize {
        self.height() * self.width()
    }

    // Advances the grid to its next iteration
    pub fn advance(&mut self) -> () {
        let coords_with_alives = self.coords_with_alives();
        for &(Coord { i, j }, alives) in coords_with_alives.iter() {
            let ref mut cell = self.cells[i][j];
            cell.update(alives)
        }
    }

    fn coords_with_alives(&self) -> Vec<(Coord, usize)> {
        self.coords_with_neighbours
            .iter()
            .map(|&CoordNeighbours {
                       ref coord,
                       ref neighbours,
                   }| {
                let alives = neighbours
                    .iter()
                    .fold(0,
                          |acc, &Coord { i, j }| if &self.cells[i][j].0 == &Status::Alive {
                              acc + 1
                          } else {
                              acc
                          });
                // This clone seems to be required in order for advance() to use
                // what it returns when mutating each cell...
                (coord.clone(), alives)
            })
            .collect()
    }
}

fn coords_with_neighbours(max_i: usize,
                          max_j: usize,
                          area: usize,
                          cells: &[Vec<Cell>])
                          -> Vec<CoordNeighbours> {
    let mut v = Vec::with_capacity(area);
    for (i, row) in cells.iter().enumerate() {
        for (j, _) in row.iter().enumerate() {
            let coord = Coord { i, j };
            let neighbours = neighbour_coords(max_i, max_j, &coord);
            v.push(CoordNeighbours { coord, neighbours })
        }
    }
    v
}

// Given an i and j, returns the (maybe wrapped) coordinates of the neighbours of that
// coordinate.
fn neighbour_coords(max_i: usize, max_j: usize, coord: &Coord) -> [Coord; 8] {
    let Coord { i, j } = *coord;

    let i_up = match i {
        0 => max_i,
        _ => i - 1,
    };

    let i_down = match i {
        _ if i == max_i => 0,
        _ => i + 1,
    };

    let j_left = match j {
        0 => max_j,
        _ => j - 1,
    };
    let j_right = match j {
        _ if j == max_j => 0,
        _ => j + 1,
    };

    let north = Coord { i: i_up, j: j };
    let north_east = Coord {
        i: i_up,
        j: j_right,
    };
    let east = Coord { i, j: j_right };
    let south_east = Coord {
        i: i_down,
        j: j_right,
    };
    let south = Coord { i: i_down, j };
    let south_west = Coord {
        i: i_down,
        j: j_left,
    };
    let west = Coord { i, j: j_left };
    let north_west = Coord { i: i_up, j: j_left };
    [north, north_east, east, south_east, south, south_west, west, north_west]
}

fn max_coordinates<A>(mat: &[Vec<A>]) -> (usize, usize) {
    let max_i = mat.len() - 1;
    let max_j = match mat.get(0) {
        Some(r) => r.len() - 1,
        None => 0,
    };
    (max_i, max_j)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_new() {
        let grid = Grid::new(10, 5);
        assert_eq!(grid.cells.len(), 5);
        assert_eq!(grid.cells[0].len(), 10);
    }

    #[test]
    fn test_neighbour_coords() {
        let grid = Grid::new(3, 3);
        let max_i = grid.max_i;
        let max_j = grid.max_j;
        /*
         * [ (0,0) (0,1) (0,2) ]
         * [ (1,0) (1,1) (1,2) ]
         * [ (2,0) (2,1) (2,2) ]
         */
        let n0 = neighbour_coords(max_i, max_j, &Coord { i: 0, j: 0 });
        assert_eq!(n0[0], Coord { i: 2, j: 0 }); // N
        assert_eq!(n0[1], Coord { i: 2, j: 1 }); // NE
        assert_eq!(n0[2], Coord { i: 0, j: 1 }); // E
        assert_eq!(n0[3], Coord { i: 1, j: 1 }); // SE
        assert_eq!(n0[4], Coord { i: 1, j: 0 }); // S
        assert_eq!(n0[5], Coord { i: 1, j: 2 }); // SW
        assert_eq!(n0[6], Coord { i: 0, j: 2 }); // W
        assert_eq!(n0[7], Coord { i: 2, j: 2 }); // NW
        let n1 = neighbour_coords(max_i, max_j, &Coord { i: 1, j: 1 });
        assert_eq!(n1[0], Coord { i: 0, j: 1 }); // N
        assert_eq!(n1[1], Coord { i: 0, j: 2 }); // NE
        assert_eq!(n1[2], Coord { i: 1, j: 2 }); // E
        assert_eq!(n1[3], Coord { i: 2, j: 2 }); // SE
        assert_eq!(n1[4], Coord { i: 2, j: 1 }); // S
        assert_eq!(n1[5], Coord { i: 2, j: 0 }); // SW
        assert_eq!(n1[6], Coord { i: 1, j: 0 }); // W
        assert_eq!(n1[7], Coord { i: 0, j: 0 }); // NW
        let n2 = neighbour_coords(max_i, max_j, &Coord { i: 2, j: 2 });
        assert_eq!(n2[0], Coord { i: 1, j: 2 }); // N
        assert_eq!(n2[1], Coord { i: 1, j: 0 }); // NE
        assert_eq!(n2[2], Coord { i: 2, j: 0 }); // E
        assert_eq!(n2[3], Coord { i: 0, j: 0 }); // SE
        assert_eq!(n2[4], Coord { i: 0, j: 2 }); // S
        assert_eq!(n2[5], Coord { i: 0, j: 1 }); // SW
        assert_eq!(n2[6], Coord { i: 2, j: 1 }); // W
        assert_eq!(n2[7], Coord { i: 1, j: 1 }); // NW
    }

    // Just a test to make sure advance can run for a large number of iterations
    #[test]
    fn test_advance() {
        let mut grid = Grid::new(50, 150);
        print!("{:?}", grid);
        for _ in 0..100 {
            grid.advance();
        }
    }

    #[test]
    fn test_alive_count() {
        let mut grid = Grid::new(3, 3);
        let new_cells = vec![vec![Cell(Status::Alive),
                                  Cell(Status::Alive),
                                  Cell(Status::Alive)],
                             vec![Cell(Status::Alive), Cell(Status::Dead), Cell(Status::Alive)],
                             vec![Cell(Status::Alive),
                                  Cell(Status::Alive),
                                  Cell(Status::Alive)]];
        grid.cells = new_cells;
        assert_eq!(alive_count(&grid), 8)
    }

    fn alive_cells(grid: &Grid) -> Vec<Coord> {
        let mut v = vec![];
        for (i, row) in grid.cells.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if cell.alive() {
                    let coord = Coord { i, j };
                    v.push(coord);
                }
            }
        }
        v
    }

    fn alive_count(grid: &Grid) -> usize {
        alive_cells(grid).len()
    }


}
