use tui::{
    buffer::{self, Buffer},
    layout::{Constraint, Direction, Layout, Rect},
    style::{self, Color, Style},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::game::{Board, Cell, Shape};

pub struct GolUi<'a> {
    pub game_row: Rect,
    pub game_area: Rect,
    pub controls_row: Rect,
    pub controls_list_area: Rect,
    pub shape_display_area: Rect,
    pub controls_toggle_area: Rect,
    pub screen_border: Block<'a>,
    pub controls_border: Block<'a>,
    pub controls_list: List<'a>,
}

impl GolUi<'_> {
    pub fn new(term_size: Rect, game_board: &Board) -> Self {
        let screen_rows = Layout::default()
            .constraints(
                [
                    Constraint::Length(game_board.height + 10),
                    Constraint::Length(10),
                ]
                .as_ref(),
            )
            .split(term_size);

        let game_row_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(((term_size.width - (game_board.width * 2)) / 2) + 1),
                Constraint::Length(game_board.width * 2 + 2),
                Constraint::Length(((term_size.width - (game_board.width * 2)) / 2) - 3),
            ])
            .vertical_margin(Board::GAME_BOARD_TOP)
            .split(screen_rows[0]);

        let controls_row_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((term_size.width - 38) / 2),
                Constraint::Min(38),
                Constraint::Length((term_size.width - 38) / 2),
            ])
            .vertical_margin(2)
            .split(screen_rows[1]);

        let controls_main_column_rows = Layout::default()
            .constraints([
                Constraint::Min(1),
                Constraint::Length(8),
                Constraint::Min(1),
            ])
            .split(controls_row_columns[1]);

        let controls_left_column_rows = Layout::default()
            .constraints([
                Constraint::Min(2),
                Constraint::Length(6),
                Constraint::Min(2),
            ])
            .split(controls_row_columns[0]);

        let controls_right_column_rows = Layout::default()
            .constraints([
                Constraint::Min(2),
                Constraint::Length(6),
                Constraint::Min(2),
            ])
            .split(controls_row_columns[2]);

        let screen_border = Block::default()
            .title("Game of Life")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let controls_border = Block::default()
            .title("Controls")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let controls_list = List::new(vec![
            ListItem::new("SPACE      : Play/Pause"),
            ListItem::new("Right →    : Next gen (if PAUSED)"),
            ListItem::new("Click      : Toggle cell at position"),
            ListItem::new("Alt-Click  : Add shape at position"),
            ListItem::new("TAB or 's' : Change shape selection"),
            ListItem::new("'r'        : Randomize"),
            ListItem::new("'c'        : Clear"),
            ListItem::new("ESC or 'q' : Quit"),
        ]);

        GolUi {
            game_row: screen_rows[0],
            game_area: game_row_columns[1],
            controls_row: screen_rows[1],
            controls_list_area: controls_main_column_rows[1],
            shape_display_area: controls_left_column_rows[1],
            controls_toggle_area: controls_right_column_rows[1],
            screen_border,
            controls_border,
            controls_list,
        }
    }
}

pub enum ControlToggle {
    Play,
    Pause,
}

impl ControlToggle {
    const PLAY: [(usize, usize); 12] = [
        (0, 2),
        (1, 2),
        (1, 3),
        (2, 2),
        (2, 3),
        (2, 4),
        (3, 2),
        (3, 3),
        (3, 4),
        (4, 2),
        (4, 3),
        (5, 2),
    ];
    const PAUSE: [(usize, usize); 16] = [
        (1, 1),
        (1, 2),
        (1, 4),
        (1, 5),
        (2, 1),
        (2, 2),
        (2, 4),
        (2, 5),
        (3, 1),
        (3, 2),
        (3, 4),
        (3, 5),
        (4, 1),
        (4, 2),
        (4, 4),
        (4, 5),
    ];
}

impl Widget for ControlToggle {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let shape = match self {
            ControlToggle::Play => Shape::new(ControlToggle::PLAY.to_vec(), None),
            ControlToggle::Pause => Shape::new(ControlToggle::PAUSE.to_vec(), None),
        };
        let mut cells = vec![vec![Cell::Dead; area.width as usize]; area.height as usize];
        shape
            .pattern
            .into_iter()
            .for_each(|pos| cells[pos.row][pos.column + 4] = Cell::Alive);

        for x in 18..36 {
            for y in 0..area.height {
                if x % 2 == 0 {
                    buf.get_mut(area.left() + x, area.top() + y)
                        .clone_from(&cells[y as usize][((x - 12) / 2) as usize].into());
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

impl Widget for Shape {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut cells = vec![vec![Cell::Dead; area.width as usize]; area.height as usize];
        let mut max_column = 0;
        for pos in self.pattern {
            cells[pos.row + 1][pos.column + 1] = Cell::Alive;
            if pos.column > max_column {
                max_column = pos.column;
            }
        }

        let shape_width = max_column as u16;
        let display_width = (shape_width + 3) * 2;
        let margin = (area.width - display_width) / 2;

        for x in margin..(margin + display_width) {
            for y in 0..area.height {
                if x % 2 == 0 {
                    buf.get_mut(area.left() + x, area.top() + y)
                        .clone_from(&cells[y as usize][((x - margin) / 2) as usize].into());
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
