use tui::{self, buffer, layout::Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Alive,
    Dead,
}

impl Cell {
    fn flip(&mut self) {
        *self = match self {
            Cell::Alive => Cell::Dead,
            Cell::Dead => Cell::Alive,
        }
    }
}

impl From<Cell> for buffer::Cell {
    fn from(cell: Cell) -> buffer::Cell {
        buffer::Cell {
            symbol: String::from("\u{25A0}"),
            fg: match cell {
                Cell::Alive => tui::style::Color::Black,
                Cell::Dead => tui::style::Color::White,
            },
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}

impl From<(usize, usize)> for Position {
    fn from(t: (usize, usize)) -> Self {
        Position {
            row: t.0,
            column: t.1,
        }
    }
}

#[derive(Clone)]
pub struct Shape {
    pub pattern: Vec<Position>,
    pub offset: Option<Position>,
}

impl Shape {
    pub const GLIDER: [(usize, usize); 5] = [(0, 2), (1, 0), (1, 2), (2, 1), (2, 2)];
    pub const ACORN: [(usize, usize); 7] = [(0, 1), (1, 3), (2, 0), (2, 1), (2, 4), (2, 5), (2, 6)];
    pub const R_PENTOMINO: [(usize, usize); 5] = [(0, 1), (0, 2), (1, 0), (1, 1), (2, 1)];

    pub fn new(cells: Vec<(usize, usize)>, offset: Option<Position>) -> Self {
        let pattern = cells.into_iter().map(|t| t.into()).collect();
        Shape { pattern, offset }
    }

    pub fn get_cells(self, width: u16, height: u16) -> Vec<Position> {
        let mut cells = self.pattern.clone();
        if let Some(point) = self.offset {
            cells.iter_mut().for_each(|pos| {
                pos.row = (pos.row + point.row) % height as usize;
                pos.column = (pos.column + point.column) % width as usize;
            });
        }
        cells
    }
}

#[derive(Clone)]
pub struct Board {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Vec<Cell>>,
}

impl Board {
    pub const GAME_BOARD_TOP: u16 = 5;

    pub fn new(
        width: u16,
        height: u16,
        init: Option<Vec<(usize, usize)>>,
        arg_offset: usize,
    ) -> Self {
        let offset: Option<Position> = match arg_offset {
            0 => None,
            offset => {
                let offset_row = offset % height as usize;
                let offset_col = offset % width as usize;
                Some((offset_row, offset_col).into())
            }
        };
        let initial_life = if let Some(shape) = init {
            Some(Shape::new(shape, offset).get_cells(width, height))
        } else {
            None
        };
        let mut cells = vec![vec![Cell::Dead; width as usize]; height as usize];
        if let Some(init) = initial_life {
            init.into_iter()
                .for_each(|pos| cells[pos.row][pos.column] = Cell::Alive);
        }
        Board {
            width,
            height,
            cells,
        }
    }

    fn count_living_neighbors(&self, pos: Position) -> u8 {
        let mut count = 0;

        [self.height - 1, 0, 1].iter().for_each(|dr| {
            let neighbor_row = (pos.row + *dr as usize) % self.height as usize;
            [self.width - 1, 0, 1].iter().for_each(|dc| {
                let neighbor_column = (pos.column + *dc as usize) % self.width as usize;
                if self.cells[neighbor_row][neighbor_column] == Cell::Alive
                    && (neighbor_row, neighbor_column) != (pos.row, pos.column)
                {
                    count += 1
                }
            })
        });
        count
    }

    pub fn flip_cell(&mut self, pos: Position) {
        self.cells[pos.row][pos.column].flip();
    }

    pub fn in_bounds(&self, row: u16, column: u16, term_rect: Rect) -> Result<Position, ()> {
        let left = (term_rect.width - self.width * 2) / 2;
        let right = left + self.width * 2;
        let top = Board::GAME_BOARD_TOP;
        let bottom = top + self.height;
        match row < bottom && row >= top && column < right && column >= left {
            true => Ok(Position {
                row: (row - top) as usize,
                column: (column - left) as usize / 2,
            }),
            false => Err(()),
        }
    }

    pub fn add_shape(&mut self, pos: Position, shape: Shape) {
        let mut positioned_shape = shape.clone();
        positioned_shape.offset = Some(pos);
        positioned_shape
            .get_cells(self.width, self.height)
            .into_iter()
            .for_each(|p| self.cells[p.row][p.column] = Cell::Alive);
    }

    pub fn tick(&mut self) {
        let mut new_cells = self.cells.clone();

        (0..self.height as usize).into_iter().for_each(|row| {
            (0..self.width as usize).into_iter().for_each(|column| {
                match (
                    self.cells[row][column],
                    self.count_living_neighbors(Position { row, column }),
                ) {
                    // Game of Life change of cell state conditions
                    (Cell::Dead, 3) => new_cells[row][column] = Cell::Alive,
                    (Cell::Alive, n) if n > 3 => new_cells[row][column] = Cell::Dead,
                    (Cell::Alive, n) if n < 2 => new_cells[row][column] = Cell::Dead,
                    _ => (),
                }
            });
        });
        self.cells = new_cells;
    }
}

pub struct GolState {
    pub game_board: Board,
    pub paused: bool,
    pub term_rect: Rect,
    shape_presets: [Shape; 3],
    preset_index: usize,
}

impl GolState {
    pub fn new(game_board: Board, term_rect: Rect) -> Self {
        let paused = true;
        let preset_index = 0;
        let shape_presets = [
            Shape::new(Shape::ACORN.to_vec(), None),
            Shape::new(Shape::GLIDER.to_vec(), None),
            Shape::new(Shape::R_PENTOMINO.to_vec(), None),
        ];
        GolState {
            game_board,
            paused,
            term_rect,
            preset_index,
            shape_presets,
        }
    }

    pub fn toggle_playpause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn cycle_presets(&mut self) {
        self.preset_index = (self.preset_index + 1) % self.shape_presets.len();
    }

    pub fn current_preset(&self) -> Shape {
        self.shape_presets[self.preset_index].clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn input_shape() -> Board {
        let shape = vec![(1, 2), (2, 3), (3, 1), (3, 2), (3, 3)];
        Board::new(6, 6, Some(shape), 0)
    }

    fn expected_shape() -> Board {
        let shape = vec![(2, 1), (2, 3), (3, 2), (3, 3), (4, 2)];
        Board::new(6, 6, Some(shape), 0)
    }

    #[test]
    fn test_tick() {
        let mut input = input_shape();
        input.tick();
        let expected = expected_shape();
        assert_eq!(input.cells, expected.cells);
    }
}
