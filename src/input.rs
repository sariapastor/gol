use crate::game::GolState;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

pub fn process_input(user_event: Event, game: &mut GolState) -> Result<(), ()> {
    match user_event {
        Event::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        }) => Err(()),
        Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            ..
        }) => Ok(game.toggle_playpause()),
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            ..
        }) => {
            if game.paused {
                game.game_board.tick()
            }
            Ok(())
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
        }) => Ok(game.cycle_presets()),
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }) => {
            if let Ok(position) = game.game_board.in_bounds(row, column, game.term_rect) {
                game.game_board.flip_cell(position);
            }
            Ok(())
        }
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::ALT,
        }) => {
            if let Ok(position) = game.game_board.in_bounds(row, column, game.term_rect) {
                game.game_board.add_shape(position, game.current_preset());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
