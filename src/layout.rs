use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
};

use crate::board::Board;

pub struct GolUi<'a> {
    pub game_row: Rect,
    pub game_area: Rect,
    pub controls_row: Rect,
    pub controls_area: Rect,
    pub screen_border: Block<'a>,
    pub controls_border: Block<'a>,
    pub controls: List<'a>,
}

impl GolUi<'_> {
    pub fn new(term_size: Rect, game_board: &Board) -> Self {
        let screen_rows = Layout::default()
            .constraints(
                [
                    // Constraint::Min(5),
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
            .vertical_margin(5)
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
                Constraint::Min((screen_rows[1].height - 12) / 2),
                Constraint::Min(12),
                Constraint::Min((screen_rows[1].height - 12) / 2),
            ])
            .split(controls_row_columns[1]);

        let screen_border = Block::default()
            .title("Game of Life")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let controls_border = Block::default()
            .title("Controls")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightGreen));

        let controls = List::new(vec![
            ListItem::new("ESC or 'q' : Quit"),
            ListItem::new("Spacebar   : Play/Pause"),
            ListItem::new("RIGHT →    : Next gen (if PAUSED)"),
            ListItem::new("Click      : Toggle cell at position"),
            ListItem::new("Alt-Click  : Add shape at position"),
        ]);
        GolUi {
            game_row: screen_rows[0],
            game_area: game_row_columns[1],
            controls_row: screen_rows[1],
            controls_area: controls_main_column_rows[1],
            screen_border,
            controls_border,
            controls,
        }
    }
}
