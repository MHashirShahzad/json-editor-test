use std::{error::Error, io};

use app::CurrentlyDeleting;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
            KeyModifiers,
        },
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

mod app;
mod ui;
use crate::{
    app::{App, CurrentScreen, CurrentlyEditing},
    ui::ui,
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);
    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    if let Ok(do_print) = res {
        if do_print {
            app.print_json()?;
        }
    } else if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
} // added comment 2
  //
  // added a comment
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_screen {
                // Main Screen Inputs
                CurrentScreen::Main => match (key.code, key.modifiers) {
                    (KeyCode::Char('e'), KeyModifiers::NONE) => {
                        app.current_screen = CurrentScreen::Editing;
                        app.currently_editing = Some(CurrentlyEditing::Key);
                    }
                    (KeyCode::Char('q'), KeyModifiers::NONE) => {
                        app.current_screen = CurrentScreen::Exiting;
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        app.current_screen = CurrentScreen::Exiting;
                    }
                    (KeyCode::Char('d'), KeyModifiers::NONE) => {
                        app.current_screen = CurrentScreen::Deleting;
                        app.currently_deleting = Some(CurrentlyDeleting::Index);
                    }
                    (KeyCode::Char('1'), KeyModifiers::NONE) => {
                        app.current_screen = CurrentScreen::FileTree;
                    }
                    _ => {}
                },
                // Exiting inputs
                CurrentScreen::Exiting => match (key.code, key.modifiers) {
                    (KeyCode::Char('y'), KeyModifiers::NONE) => {
                        return Ok(true);
                    }
                    (KeyCode::Char('n'), KeyModifiers::NONE)
                    | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                        return Ok(false);
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        return Ok(false);
                    }
                    _ => {}
                },
                // Editing Inputs
                CurrentScreen::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::Key => {
                                    app.currently_editing = Some(CurrentlyEditing::Value);
                                }
                                CurrentlyEditing::Value => {
                                    app.save_key_value();
                                    app.current_screen = CurrentScreen::Main;
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::Key => {
                                    app.key_input.pop();
                                }
                                CurrentlyEditing::Value => {
                                    app.value_input.pop();
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                        app.currently_editing = None;
                    }
                    KeyCode::Tab => {
                        app.toggle_editing();
                    }
                    KeyCode::Char(value) => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::Key => {
                                    app.key_input.push(value);
                                }
                                CurrentlyEditing::Value => {
                                    app.value_input.push(value);
                                }
                            }
                        }
                    }
                    _ => {}
                },
                // Deleting Inputs
                CurrentScreen::Deleting if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        if let Some(deleting) = &app.currently_deleting {
                            match deleting {
                                CurrentlyDeleting::Index => {
                                    app.currently_deleting = Some(CurrentlyDeleting::Index);
                                    app.delete_key();
                                    app.delete_index = String::new();
                                    app.current_screen = CurrentScreen::Main;
                                }
                            }
                        }
                    }
                    // Backspace
                    KeyCode::Backspace => {
                        if let Some(deleting) = &app.currently_deleting {
                            match deleting {
                                CurrentlyDeleting::Index => {
                                    app.delete_index.pop();
                                }
                            }
                        }
                    }
                    // Escape
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                        app.currently_editing = None;
                        app.delete_index = String::new();
                    }
                    // Any Character
                    KeyCode::Char(value) => {
                        if let Some(deleting) = &app.currently_deleting {
                            match deleting {
                                CurrentlyDeleting::Index => {
                                    app.delete_index.push(value);
                                }
                            }
                        }
                    }
                    _ => {}
                },
                // viewing file tree
                CurrentScreen::FileTree if key.kind == KeyEventKind::Press => {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('2'), KeyModifiers::NONE) => {
                            app.current_screen = CurrentScreen::Main;
                        }
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            app.current_screen = CurrentScreen::Exiting;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
