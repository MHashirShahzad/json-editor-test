use std::{fmt::Alignment, rc::Rc, slice::Chunks};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::*,
    Frame,
};

use crate::app::{App, CurrentScreen, CurrentlyDeleting, CurrentlyEditing};

pub fn ui(frame: &mut Frame, app: &App) {
    // Create the layout sections.
    let chunks: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_title(frame, &chunks);
    render_json_values(frame, app, &chunks);
    render_bottombar(frame, app, &chunks);

    if let Some(editing) = &app.currently_editing {
        render_editing_popup(frame, app, editing);
    }

    if let CurrentScreen::Exiting = app.current_screen {
        render_exit_popup(frame);
    }
    if let CurrentScreen::Deleting = app.current_screen {
        render_deleting_popup(frame, app);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

fn render_title(frame: &mut Frame, chunks: &Rc<[Rect]>) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "JSON editor",
        Style::default().fg(Color::Magenta).bold(),
    ))
    .block(title_block)
    .centered();

    frame.render_widget(title, chunks[0]);
}

fn render_bottombar(frame: &mut Frame, app: &App, chunks: &Rc<[Rect]>) {
    let current_navigation_text = vec![
        // The first half of the text
        match app.current_screen {
            CurrentScreen::Main => Span::styled("Normal Mode", Style::default().fg(Color::Green)),
            CurrentScreen::Editing => Span::styled(
                "Editing Mode",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
            CurrentScreen::Exiting => Span::styled(
                "Exiting",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::RAPID_BLINK),
            ),
            CurrentScreen::Deleting => Span::styled(
                "Deleting",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::RAPID_BLINK),
            ),
        }
        .to_owned(),
        // A white divider bar to separate the two sections
        Span::styled(" | ", Style::default().fg(Color::White)),
        // The final section of the text, with hints on what the user is editing
        {
            if let Some(editing) = &app.currently_editing {
                match editing {
                    CurrentlyEditing::Key => {
                        Span::styled("Editing Json Key", Style::default().fg(Color::Green))
                    }
                    CurrentlyEditing::Value => {
                        Span::styled("Editing Json Value", Style::default().fg(Color::LightGreen))
                    }
                }
            } else {
                Span::styled("Not Editing Anything", Style::default().fg(Color::DarkGray))
            }
        },
    ];

    // Left FOOTER
    let mode_footer = Paragraph::new(Line::from(current_navigation_text)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_type(BorderType::Rounded),
    );

    let current_keys_hint = {
        match app.current_screen {
            CurrentScreen::Main => Span::styled(
                "(q) or (CTRL+c) to quit / (e) to make new pair / (d) to delete a pair",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::Editing => Span::styled(
                "(ESC) to cancel/(Tab) to switch boxes/enter to complete",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::Exiting => Span::styled(
                "(q) to quit / (e) to make new pair",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::Deleting => {
                Span::styled("enter to delete", Style::default().fg(Color::Red))
            }
        }
    };

    let key_notes_footer = Paragraph::new(Line::from(current_keys_hint)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    frame.render_widget(mode_footer, footer_chunks[0]);
    frame.render_widget(key_notes_footer, footer_chunks[1]);
}

fn render_exit_popup(frame: &mut Frame) {
    frame.render_widget(Clear, frame.area()); //this clears the entire screen and anything already drawn
    let popup_block = Block::default()
        .title("Y/N")
        .title_alignment(ratatui::layout::Alignment::Center)
        .add_modifier(Modifier::BOLD)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default());

    let exit_text = Text::styled(
        "Would you like to output the buffer as json? (y/n)",
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::SLOW_BLINK),
    );
    // the `trim: false` will stop the text from being cut off when over the edge of the block
    let exit_paragraph = Paragraph::new(exit_text)
        .block(popup_block)
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: false });

    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(exit_paragraph, area);
}

fn render_editing_popup(frame: &mut Frame, app: &App, editing: &CurrentlyEditing) {
    let popup_block = Block::default()
        .title("Enter a new key-value pair")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default());

    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(popup_block, area);

    let popup_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let mut key_block = Block::default()
        .title("Key")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let mut value_block = Block::default()
        .title("Value")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

    match editing {
        CurrentlyEditing::Key => key_block = key_block.style(active_style),
        CurrentlyEditing::Value => value_block = value_block.style(active_style),
    };

    let key_text = Paragraph::new(app.key_input.clone()).block(key_block);
    frame.render_widget(key_text, popup_chunks[0]);

    let value_text = Paragraph::new(app.value_input.clone()).block(value_block);
    frame.render_widget(value_text, popup_chunks[1]);
}
fn render_deleting_popup(frame: &mut Frame, app: &App) {
    let popup_block = Block::default()
        .title("Enter an index to delete")
        .bold()
        .italic()
        .title_alignment(ratatui::layout::Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default());

    let delete_text = Paragraph::new(app.delete_index.clone()).block(popup_block);

    let area = centered_rect(30, 25, frame.area());
    frame.render_widget(delete_text, area);
}

fn render_json_values(frame: &mut Frame, app: &App, chunks: &Rc<[Rect]>) {
    let mut list_items = Vec::<ListItem>::new();
    let mut index = 0;

    let json_block = Block::new()
        .title("[2] JSON ")
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default());

    let json_chunks: Rc<[Rect]> = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Length(30), Constraint::Min(10)])
        .split(chunks[1]);

    render_file_tree(frame, &json_chunks);
    // frame.render_widget(&json_block, chunks[1]);

    list_items.push(ListItem::new(Line::from(Span::styled(
        format!("{: <25} | {: <25} : {}", "Index", "Key", "Value"),
        Style::default().fg(Color::LightGreen).bold().italic(),
    ))));

    //render_json_title(frame, &json_chunks);

    for key in app.pairs.keys() {
        list_items.push(ListItem::new(Line::from(Span::styled(
            format!(
                "{: <25} | {: <25} : {}",
                index,
                key,
                app.pairs.get(key).unwrap()
            ),
            Style::default().fg(Color::Yellow),
        ))));
        index = index + 1;
    }

    let list = List::new(list_items).block(json_block);

    frame.render_widget(list, json_chunks[1]);
}

fn render_file_tree(frame: &mut Frame, json_chunks: &Rc<[Rect]>) {
    let tree_block = Block::new()
        .title("[1] Tree ")
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default());

    frame.render_widget(tree_block, json_chunks[0]);
}
// fn render_json_title(frame: &mut Frame, json_chunks: &Rc<[Rect]>) {
//     let title_chunks: Rc<[Rect]> = Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints([
//             Constraint::Length(30),
//             Constraint::Min(3),
//             Constraint::Min(3),
//         ])
//         .split(json_chunks[0]);

//     // index
//     let index_block = Block::default()
//         .add_modifier(Modifier::BOLD)
//         .borders(Borders::ALL)
//         .border_type(BorderType::Rounded)
//         .style(Style::default());

//     let index_title = Paragraph::new(Text::styled(
//         "INDEX",
//         Style::default().fg(Color::LightGreen),
//     ))
//     .block(index_block)
//     .centered();

//     let key_block = Block::default()
//         .add_modifier(Modifier::BOLD)
//         .borders(Borders::ALL)
//         .border_type(BorderType::Rounded)
//         .style(Style::default());

//     let key_title = Paragraph::new(Text::styled("Key", Style::default().fg(Color::LightGreen)))
//         .block(key_block)
//         .centered();

//     let value_block = Block::default()
//         .add_modifier(Modifier::BOLD)
//         .borders(Borders::ALL)
//         .border_type(BorderType::Rounded)
//         .style(Style::default());

//     let value_title = Paragraph::new(Text::styled(
//         "VALUE",
//         Style::default().fg(Color::LightGreen),
//     ))
//     .block(value_block)
//     .centered();

//     frame.render_widget(index_title, title_chunks[0]);
//     frame.render_widget(key_title, title_chunks[1]);
//     frame.render_widget(value_title, title_chunks[2]);
// }
