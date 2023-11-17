use super::cell::{Cell, Status};
use rand;
use rand::Rng;
use rayon::prelude::*;
use std::mem;

pub const PAR_THRESHOLD_AREA: usize = 250_000;

/// Used for indexing into the grid
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq)]
pub struct GridIdx(pub usize);

#[derive(Debug)]
pub struct Grid {
    /* Addressed by from-zero (i, j) notation, where i is row number, j is column number
     * such that given the following shows coordinates for cells in a 3 x 3 grid:
     *
     * [ (0,0) (0,1) (0,2) ]
     * [ (1,0) (1,1) (1,2) ]
     * [ (2,0) (2,1) (2,2) ]
     *
     * will get flattened into a single vector:
     * [ (0,0), (0,1), (0,2), (1,0), (1,1), (1,2), (2,0), (2,1), (2,2) ]
     */
    cells: Vec<Cell>,
    scratchpad_cells: Vec<Cell>,
    max_i: usize,
    max_j: usize,
    area: usize,
    // Cache of where the neighbours are for each point
    neighbours: Vec<[GridIdx; 8]>,
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Clone)]
pub struct Coord {
    pub i: usize,
    pub j: usize,
}

impl Grid {
    /// Creates a grid with the given width and height
    pub fn new(width: usize, height: usize) -> Self {
        let mut rng = rand::thread_rng();
        // Grid is a matrix with {height} rows and {width} columns, addressed
        // via (i, j) (row, column) convention. Used for finding neightbours because it's
        // just an easier mental model to work with for that problem. It gets flattened later.
        let mut grid = Vec::with_capacity(height);
        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                let status = if rng.gen() {
                    Status::Alive
                } else {
                    Status::Dead
                };
                let cell = Cell(status);
                row.push(cell);
            }
            grid.push(row);
        }

        let max_i = if height == 0 { 0 } else { height - 1 };
        let max_j = if width == 0 { 0 } else { width - 1 };
        let neighbours = neighbours(max_i, max_j, &grid);
        let cells: Vec<Cell> = grid.into_iter().flatten().collect();
        let scratchpad_cells = cells.clone();
        let area = width * height;
        Self {
            cells,
            scratchpad_cells,
            max_i,
            max_j,
            area,
            neighbours,
        }
    }

    /// Returns the i-th Cell in a grid as if the 2 dimensional matrix
    /// has been flattened into a 1 dimensional one row-wise
    ///
    /// TODO: is using iter faster or slower than just doing the checks?
    pub fn get_idx(&self, &GridIdx(idx): &GridIdx) -> Option<&Cell> {
        if idx < self.cells.len() {
            Some(&self.cells[idx])
        } else {
            None
        }
    }

    // TODO delete if not used
    pub const fn to_grid_idx(&self, &Coord { i, j }: &Coord) -> Option<GridIdx> {
        if i <= self.max_i && j <= self.max_j {
            Some(GridIdx(self.width() * i + j))
        } else {
            None
        }
    }

    // Returns a slice with references to this grid's cells
    pub fn cells(&self) -> Vec<Vec<&Cell>> {
        let mut rows = Vec::with_capacity(self.height());
        let mut i = 0;
        for _ in 0..self.height() {
            let mut columns = Vec::with_capacity(self.width());
            for _ in 0..self.width() {
                columns.push(&self.cells[i]);
                i += 1;
            }
            rows.push(columns);
        }
        rows
    }

    pub const fn height(&self) -> usize {
        self.max_i + 1
    }

    pub const fn width(&self) -> usize {
        self.max_j + 1
    }

    pub const fn area(&self) -> usize {
        self.area
    }

    pub fn advance(&mut self) {
        {
            let neighbours = &self.neighbours;
            let last_gen = &self.cells;
            let area_requires_par = self.area() >= PAR_THRESHOLD_AREA;
            let cells = &mut self.scratchpad_cells;
            let cell_op = |(i, cell): (usize, &mut Cell)| {
                let alives = neighbours[i].iter().fold(0, |acc, &GridIdx(idx)| {
                    if last_gen[idx].0 == Status::Alive {
                        acc + 1
                    } else {
                        acc
                    }
                });
                let next_status = last_gen[i].next_status(alives);
                cell.update(next_status);
            };
            if area_requires_par {
                cells.par_iter_mut().enumerate().for_each(cell_op);
            } else {
                for (i, cell) in cells.iter_mut().enumerate() {
                    cell_op((i, cell));
                }
            }
        }
        mem::swap(&mut self.cells, &mut self.scratchpad_cells);
    }
}

fn neighbours(max_i: usize, max_j: usize, cells: &[Vec<Cell>]) -> Vec<[GridIdx; 8]> {
    let mut v = Vec::with_capacity((max_i + 1) * (max_j + 1));
    for (i, row) in cells.iter().enumerate() {
        for (j, _) in row.iter().enumerate() {
            let coord = Coord { i, j };
            v.push(neighbour_coords(max_i, max_j, &coord));
        }
    }
    v
}

fn neighbour_coords(max_i: usize, max_j: usize, coord: &Coord) -> [GridIdx; 8] {
    let width = max_j + 1;
    let Coord { i, j } = *coord;
    let to_grid_idx = |Coord { i, j }: Coord| GridIdx(width * i + j);

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

    let north = Coord { i: i_up, j };
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
    [
        to_grid_idx(north),
        to_grid_idx(north_east),
        to_grid_idx(east),
        to_grid_idx(south_east),
        to_grid_idx(south),
        to_grid_idx(south_west),
        to_grid_idx(west),
        to_grid_idx(north_west),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_new() {
        let grid = Grid::new(10, 5);
        assert_eq!(grid.cells().len(), 5);
        assert_eq!(grid.cells()[0].len(), 10);
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
        assert_eq!(n0[0], grid.to_grid_idx(&Coord { i: 2, j: 0 }).unwrap()); // N
        assert_eq!(n0[1], grid.to_grid_idx(&Coord { i: 2, j: 1 }).unwrap()); // NE
        assert_eq!(n0[2], grid.to_grid_idx(&Coord { i: 0, j: 1 }).unwrap()); // E
        assert_eq!(n0[3], grid.to_grid_idx(&Coord { i: 1, j: 1 }).unwrap()); // SE
        assert_eq!(n0[4], grid.to_grid_idx(&Coord { i: 1, j: 0 }).unwrap()); // S
        assert_eq!(n0[5], grid.to_grid_idx(&Coord { i: 1, j: 2 }).unwrap()); // SW
        assert_eq!(n0[6], grid.to_grid_idx(&Coord { i: 0, j: 2 }).unwrap()); // W
        assert_eq!(n0[7], grid.to_grid_idx(&Coord { i: 2, j: 2 }).unwrap()); // NW
        let n1 = neighbour_coords(max_i, max_j, &Coord { i: 1, j: 1 });
        assert_eq!(n1[0], grid.to_grid_idx(&Coord { i: 0, j: 1 }).unwrap()); // N
        assert_eq!(n1[1], grid.to_grid_idx(&Coord { i: 0, j: 2 }).unwrap()); // NE
        assert_eq!(n1[2], grid.to_grid_idx(&Coord { i: 1, j: 2 }).unwrap()); // E
        assert_eq!(n1[3], grid.to_grid_idx(&Coord { i: 2, j: 2 }).unwrap()); // SE
        assert_eq!(n1[4], grid.to_grid_idx(&Coord { i: 2, j: 1 }).unwrap()); // S
        assert_eq!(n1[5], grid.to_grid_idx(&Coord { i: 2, j: 0 }).unwrap()); // SW
        assert_eq!(n1[6], grid.to_grid_idx(&Coord { i: 1, j: 0 }).unwrap()); // W
        assert_eq!(n1[7], grid.to_grid_idx(&Coord { i: 0, j: 0 }).unwrap()); // NW
        let n2 = neighbour_coords(max_i, max_j, &Coord { i: 2, j: 2 });
        assert_eq!(n2[0], grid.to_grid_idx(&Coord { i: 1, j: 2 }).unwrap()); // N
        assert_eq!(n2[1], grid.to_grid_idx(&Coord { i: 1, j: 0 }).unwrap()); // NE
        assert_eq!(n2[2], grid.to_grid_idx(&Coord { i: 2, j: 0 }).unwrap()); // E
        assert_eq!(n2[3], grid.to_grid_idx(&Coord { i: 0, j: 0 }).unwrap()); // SE
        assert_eq!(n2[4], grid.to_grid_idx(&Coord { i: 0, j: 2 }).unwrap()); // S
        assert_eq!(n2[5], grid.to_grid_idx(&Coord { i: 0, j: 1 }).unwrap()); // SW
        assert_eq!(n2[6], grid.to_grid_idx(&Coord { i: 2, j: 1 }).unwrap()); // W
        assert_eq!(n2[7], grid.to_grid_idx(&Coord { i: 1, j: 1 }).unwrap()); // NW
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
        let new_cells = vec![
            vec![
                Cell(Status::Alive),
                Cell(Status::Alive),
                Cell(Status::Alive),
            ],
            vec![Cell(Status::Alive), Cell(Status::Dead), Cell(Status::Alive)],
            vec![
                Cell(Status::Alive),
                Cell(Status::Alive),
                Cell(Status::Alive),
            ],
        ]
        .into_iter()
        .flat_map(|v| v)
        .collect();
        grid.cells = new_cells;
        assert_eq!(alive_count(&grid), 8)
    }

    #[test]
    fn test_get_idx() {
        let mut grid = Grid::new(3, 3);
        let new_cells: Vec<Cell> = vec![
            vec![
                Cell(Status::Alive),
                Cell(Status::Alive),
                Cell(Status::Alive),
            ],
            vec![Cell(Status::Alive), Cell(Status::Dead), Cell(Status::Alive)],
            vec![
                Cell(Status::Alive),
                Cell(Status::Alive),
                Cell(Status::Alive),
            ],
        ]
        .into_iter()
        .flat_map(|v| v)
        .collect();
        grid.cells = new_cells;
        for idx in 0..9 {
            let cell = grid.get_idx(&GridIdx(idx)).unwrap();
            if idx != 4 {
                assert!(cell.alive())
            } else {
                assert!(!cell.alive())
            }
        }
    }

    /// Given
    ///
    /// [ (0,0) (0,1) (0,2) (0, 3) ]
    /// [ (1,0) (1,1) (1,2) (1, 3) ]
    /// [ (2,0) (2,1) (2,2) (2, 3) ]
    #[test]
    fn test_to_grid_idx() {
        let grid = Grid::new(4, 3);
        assert_eq!(grid.to_grid_idx(&Coord { i: 0, j: 0 }), Some(GridIdx(0)));
        assert_eq!(grid.to_grid_idx(&Coord { i: 1, j: 2 }), Some(GridIdx(6)));
        assert_eq!(grid.to_grid_idx(&Coord { i: 2, j: 3 }), Some(GridIdx(11)));
        assert_eq!(grid.to_grid_idx(&Coord { i: 3, j: 3 }), None);
    }

    fn alive_cells(grid: &Grid) -> Vec<Coord> {
        let mut v = vec![];
        for (i, row) in grid.cells().iter().enumerate() {
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
