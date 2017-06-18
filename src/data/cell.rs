
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum Status {
    Dead,
    Alive,
}

#[derive(Debug, Clone)]
pub struct Cell(pub Status);

impl Cell {
    /// Wraps status
    pub fn alive(&self) -> bool {
        self.0 == Status::Alive
    }

    // Update alive status based on neighbour count
    // https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life#Rules
    pub fn update(&mut self, status: Status) -> () {
        self.0 = status
    }

    // Returns the next status given a number of neighbours
    pub fn next_status(&self, neighbours_cnt: usize) -> Status {
        match (&self.0, neighbours_cnt) {
            (_, 3) => Status::Alive,
            (&Status::Alive, 2) => Status::Alive,
            _ => Status::Dead,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_next_status() {
        let cell = Cell(Status::Alive);
        assert_eq!(cell.next_status(0), Status::Dead);
        assert_eq!(cell.next_status(5), Status::Dead);
        assert_eq!(cell.next_status(3), Status::Alive);
        assert_eq!(cell.next_status(2), Status::Alive);
    }
}
