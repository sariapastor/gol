use clap::Parser;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::Duration};
use tui::{backend::CrosstermBackend, Terminal};

mod board;
mod layout;

use board::*;
use layout::*;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 32)]
    rows: u16,
    #[arg(short, long, default_value_t = 64)]
    columns: u16,
    #[arg(short, long, default_value_t = String::from("rpentomino"))]
    shape: String,
    #[arg(short, long, default_value_t = 10)]
    offset: usize,
}

fn main() -> Result<(), io::Error> {
    // configure board
    let args = Args::parse();
    let offset: Option<Position> = if args.offset != 0 {
        let offset_row = args.offset % args.rows as usize;
        let offset_col = args.offset % args.columns as usize;
        Some((offset_row, offset_col).into())
    } else {
        None
    };
    let init = match args.shape.as_str() {
        "acorn" => Some(Shape::new(Shape::ACORN.to_vec(), offset).get_cells()),
        "glider" => Some(Shape::new(Shape::GLIDER.to_vec(), offset).get_cells()),
        "rpentomino" => Some(Shape::new(Shape::R_PENTOMINO.to_vec(), offset).get_cells()),
        _ => None,
    };

    let mut game_board = Board::new(args.columns, args.rows, init);
    let preset_shapes = [
        Shape::new(Shape::ACORN.to_vec(), None),
        Shape::new(Shape::GLIDER.to_vec(), None),
        Shape::new(Shape::R_PENTOMINO.to_vec(), None),
    ];
    let mut current_shape_index = 0;

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnableMouseCapture, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut paused = true;

    loop {
        let control_toggle = match paused {
            true => ControlToggle::Play,
            false => ControlToggle::Pause,
        };
        let term_rect = terminal.size().expect("Error getting terminal dimensions");
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => break,
                Event::Key(KeyEvent {
                    code: KeyCode::Char(' '),
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => paused = !paused,
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => {
                    if paused {
                        game_board.tick()
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Tab,
                    modifiers: KeyModifiers::NONE,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('s'),
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => current_shape_index = (current_shape_index + 1) % preset_shapes.len(),
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if let Ok(position) = game_board.in_bounds(row, column, term_rect) {
                        game_board.flip_cell(position);
                    }
                }
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    modifiers: KeyModifiers::ALT,
                }) => {
                    if let Ok(position) = game_board.in_bounds(row, column, term_rect) {
                        game_board.add_shape(position, preset_shapes[current_shape_index].clone());
                    }
                }
                _ => (),
            }
        } else {
            terminal.draw(|frame| {
                let board = game_board.clone();
                let layout = GolUi::new(frame.size(), &board);
                frame.render_widget(layout.screen_border, frame.size());
                frame.render_widget(layout.controls_border, layout.controls_row);
                frame.render_widget(board, layout.game_area);
                frame.render_widget(layout.controls_list, layout.controls_list_area);
                frame.render_widget(
                    preset_shapes[current_shape_index].clone(),
                    layout.shape_display_area,
                );
                frame.render_widget(control_toggle, layout.controls_toggle_area);
            })?;
            if !paused {
                game_board.tick();
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn input_spaceship() -> Board {
        let init = [(1, 2), (2, 3), (3, 1), (3, 2), (3, 3)]
            .into_iter()
            .map(|(row, column)| Position { row, column })
            .collect();
        Board::new(6, 6, Some(init))
    }

    fn expected_spaceship() -> Board {
        let init = [(2, 1), (2, 3), (3, 2), (3, 3), (4, 2)]
            .into_iter()
            .map(|(row, column)| Position { row, column })
            .collect();
        Board::new(6, 6, Some(init))
    }

    #[test]
    fn test_tick() {
        let mut input = input_spaceship();
        input.tick();
        let expected = expected_spaceship();
        assert_eq!(input.cells, expected.cells);
    }
}
