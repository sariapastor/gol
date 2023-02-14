use tui::{
    self,
    buffer::{self, Buffer},
    layout::Rect,
    style::{self, Color, Style},
    widgets::Widget,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Alive,
    Dead,
}

impl Cell {
    fn flip(&mut self) {
        if *self == Cell::Alive {
            *self = Cell::Dead;
        } else {
            *self = Cell::Alive;
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

impl From<usize> for Position {
    fn from(u: usize) -> Self {
        Position { row: u, column: u }
    }
}

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

    pub fn get_cells(self) -> Vec<Position> {
        let mut cells = self.pattern.clone();
        if let Some(point) = self.offset {
            cells.iter_mut().for_each(|pos| {
                pos.row += point.row;
                pos.column += point.column;
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

    pub fn new(width: u16, height: u16, initial_life: Option<Vec<Position>>) -> Self {
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

    pub fn add_shape(&mut self, pos: Position) {
        let shape = Shape::new(Shape::ACORN.to_vec(), Some(pos));
        shape
            .get_cells()
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

impl Widget for Board {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_cells: Vec<Vec<buffer::Cell>> = self
            .cells
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|cell| buffer::Cell {
                        symbol: String::from("\u{25A0}"),
                        fg: match cell {
                            Cell::Alive => Color::Black,
                            Cell::Dead => Color::White,
                        },
                        ..Default::default()
                    })
                    .collect()
            })
            .collect();

        for x in 0..(self.width * 2) {
            for y in 0..self.height {
                if x % 2 == 0 {
                    buf.get_mut(area.left() + x, area.top() + y)
                        .clone_from(&content_cells[y as usize][(x / 2) as usize]);
                } else {
                    buf.get_mut(area.left() + x, area.top() + y)
                        .set_symbol(tui::symbols::line::VERTICAL)
                        .set_fg(Color::Black)
                        .set_style(Style {
                            add_modifier: style::Modifier::DIM,
                            ..Default::default()
                        });
                }
            }
        }
    }
}
