mod game;
mod input;
mod ui;

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use game::{Board, GolState, Shape};
use std::{io, sync::mpsc::channel, thread, time::Duration};
use tui::{backend::CrosstermBackend, Terminal};
use ui::{ControlToggle, GolUi};

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
    // configure
    let args = Args::parse();

    let init = match args.shape.as_str() {
        "acorn" => Some(Shape::ACORN.to_vec()),
        "glider" => Some(Shape::GLIDER.to_vec()),
        "rpentomino" => Some(Shape::R_PENTOMINO.to_vec()),
        _ => None,
    };
    let board = Board::new(args.columns, args.rows, init, args.offset);

    // listen for user input
    let (tx, rx) = channel::<Event>();

    thread::spawn(move || loop {
        if event::poll(Duration::from_millis(500)).unwrap_or(false) {
            let _ = tx.send(
                event::read().expect("Should have been an event to read since poll returned true."),
            );
        }
    });

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnableMouseCapture, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // initialize game state
    let term_rect = terminal.size().expect("Error getting terminal dimensions");
    let mut game_state = GolState::new(board, term_rect);

    loop {
        if let Ok(user_event) = rx.try_recv() {
            if input::process_input(user_event, &mut game_state).is_err() {
                break;
            }
        } else {
            terminal.draw(|frame| {
                let board = game_state.game_board.clone();
                let layout = GolUi::new(frame.size(), &board);
                frame.render_widget(layout.screen_border, frame.size());
                frame.render_widget(layout.controls_border, layout.controls_row);
                frame.render_widget(board, layout.game_area);
                frame.render_widget(layout.controls_list, layout.controls_list_area);
                frame.render_widget(
                    // shape_presets[preset_index].clone(),
                    game_state.current_preset(),
                    layout.shape_display_area,
                );
                frame.render_widget(
                    match game_state.paused {
                        true => ControlToggle::Play,
                        false => ControlToggle::Pause,
                    },
                    layout.controls_toggle_area,
                );
            })?;
            if !game_state.paused {
                game_state.game_board.tick();
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    // restore terminal on exit
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}
