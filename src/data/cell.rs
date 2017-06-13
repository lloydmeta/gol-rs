
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Status {
    Dead,
    Alive,
}

#[derive(Debug)]
pub struct Cell(pub Status);

impl Cell {
    // Simple check
    pub fn alive(&self) -> bool {
        self.0 == Status::Alive
    }

    // Update alive status based on neighbour count
    // https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life#Rules
    pub fn update(&mut self, neighbours_cnt: usize) -> () {
        self.0 = self.next_status(neighbours_cnt)
    }

    fn next_status(&self, neighbours_cnt: usize) -> Status {
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
