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
    pub playpause_toggle_area: Rect,
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
        let board_margin_width = if term_size.width > game_board.width * 2 {
            (term_size.width - (game_board.width * 2)) / 2
        } else {
            0
        };

        let game_row_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(board_margin_width),
                Constraint::Length(game_board.width * 2 + 2),
                Constraint::Length(board_margin_width),
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
                Constraint::Length(2),
                Constraint::Length(7),
                Constraint::Min(1),
            ])
            .split(controls_row_columns[1]);

        let controls_left_column_rows = Layout::default()
            .constraints([
                Constraint::Length(2),
                Constraint::Length(7),
                Constraint::Min(1),
            ])
            .split(controls_row_columns[0]);

        let controls_right_column_rows = Layout::default()
            .constraints([
                Constraint::Length(2),
                Constraint::Length(7),
                Constraint::Min(1),
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
            ListItem::new("TAB        : Change shape selection"),
            ListItem::new("'C' / 'R'  : Clear / Randomize"),
            ListItem::new("ESC or 'Q' : Quit"),
        ]);

        GolUi {
            game_row: screen_rows[0],
            game_area: game_row_columns[1],
            controls_row: screen_rows[1],
            controls_list_area: controls_main_column_rows[1],
            shape_display_area: controls_left_column_rows[1],
            playpause_toggle_area: controls_right_column_rows[1],
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
    const PLAYPAUSE: [(usize, usize); 36] = [
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (0, 1),
        (6, 1),
        (0, 2),
        (1, 2),
        (5, 2),
        (6, 2),
        (0, 3),
        (1, 3),
        (2, 3),
        (4, 3),
        (5, 3),
        (6, 3),
        (0, 4),
        (1, 4),
        (2, 4),
        (3, 4),
        (4, 4),
        (5, 4),
        (6, 4),
        (1, 6),
        (2, 6),
        (3, 6),
        (4, 6),
        (5, 6),
        (1, 8),
        (2, 8),
        (3, 8),
        (4, 8),
        (5, 8),
    ];
}

impl Widget for ControlToggle {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let shape = Shape::new(ControlToggle::PLAYPAUSE.to_vec(), None);
        let width = if area.width > 10 { area.width } else { 10 };
        let height = if area.height > 7 { area.height } else { 7 };
        let mut cells = match self {
            ControlToggle::Play => vec![vec![Cell::Dead; width as usize]; height as usize],
            ControlToggle::Pause => vec![vec![Cell::Alive; width as usize]; height as usize],
        };

        shape.pattern.into_iter().for_each(|pos| {
            cells[pos.row][pos.column] = match self {
                ControlToggle::Pause => Cell::Dead,
                ControlToggle::Play => Cell::Alive,
            }
        });
        let draw_width = if area.width < 20 { area.width } else { 20 };
        let margin = if width > draw_width {
            (width - draw_width) / 2 - 2
        } else {
            0
        };
        let draw_height = if area.height < 7 { area.height } else { 7 };
        for x in margin..(margin + draw_width) {
            for y in 0..draw_height {
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

        let draw_width = if area.width < self.width * 2 {
            area.width
        } else {
            self.width * 2
        };
        let draw_height = if area.height < self.height {
            area.height
        } else {
            self.height
        };
        for x in 0..draw_width {
            for y in 0..draw_height {
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
        let width = if area.width > 9 { area.width } else { 9 };
        let height = if area.height > 6 { area.height } else { 6 };
        let mut cells = vec![vec![Cell::Dead; width as usize]; height as usize];

        let mut max_column = 0;
        for pos in self.pattern {
            cells[pos.row + 2][pos.column + 1] = Cell::Alive;
            if pos.column > max_column {
                max_column = pos.column;
            }
        }

        let shape_width = max_column as u16;
        let draw_width = if area.width > (shape_width + 3) * 2 {
            (shape_width + 3) * 2
        } else {
            area.width
        };
        let margin = if width > draw_width {
            (width - draw_width) / 2
        } else {
            0
        };
        let draw_height = if area.height < height {
            area.height
        } else {
            height
        };

        for x in margin..(margin + draw_width) {
            for y in 0..draw_height {
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
