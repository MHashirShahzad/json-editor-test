use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub enum CurrentScreen {
    Main,
    Editing,
    FileTree,
    Exiting,
    Deleting,
}
use ratatui::widgets::{Scrollbar, ScrollbarState};
pub enum CurrentlyEditing {
    Key,
    Value,
}
pub enum CurrentlyDeleting {
    Index,
}

pub struct App {
    pub key_input: String,   // the currently being edited json key.
    pub value_input: String, // the currently being edited json value.
    pub delete_index: String,
    pub pairs: HashMap<String, String>, // The representation of our key and value pairs with serde Serialize support
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub currently_editing: Option<CurrentlyEditing>,
    pub currently_deleting: Option<CurrentlyDeleting>, // the optional state containing which of the key or value pair the user is editing. It is an option, because when the user is not directly editing a key-value pair, this will be set to `None`.
    pub vertical_scroll_state: Option<ScrollbarState>,
    //pub horizontal_scroll_state: Option<ScrollbarState>,
    pub vertical_scroll: usize,
    //pub horizontal_scroll: usize,
}

impl App {
    pub fn new() -> App {
        App {
            key_input: String::new(),
            value_input: String::new(),
            delete_index: String::new(),
            pairs: HashMap::new(),
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            currently_deleting: None,
            vertical_scroll_state: None,
            vertical_scroll: 0,
        }
    }

    pub fn save_key_value(&mut self) {
        self.pairs
            .insert(self.key_input.clone(), self.value_input.clone());
        self.key_input = String::new();
        self.value_input = String::new();
        self.currently_editing = None;
    }

    // pub fn save_deleting_index(&mut self) {
    //     self.delete_index
    //         .insert(self.key_input.clone(), self.value_input.clone());
    //     self.key_input = String::new();
    //     self.value_input = String::new();
    //     self.currently_editing = None;
    // }

    pub fn delete_key(&mut self) {
        // Parse String as int
        match self.delete_index.parse::<usize>() {
            Ok(index) => {
                if let Some(pair_to_del) = self.pairs.iter().nth(index) {
                    let key_to_del = pair_to_del.0.clone();
                    self.pairs.remove(&key_to_del);
                }
            }
            Err(_) => {}
        }
    }

    pub fn toggle_editing(&mut self) {
        if let Some(edit_mode) = &self.currently_editing {
            match edit_mode {
                CurrentlyEditing::Key => self.currently_editing = Some(CurrentlyEditing::Value),
                CurrentlyEditing::Value => self.currently_editing = Some(CurrentlyEditing::Key),
            };
        } else {
            self.currently_editing = Some(CurrentlyEditing::Key);
        }
    }

    pub fn print_json(&self) -> serde_json::Result<()> {
        // create and write json file
        let json_string = serde_json::to_string_pretty(&self.pairs)?;
        let mut file = File::create("output.json")?;
        file.write_all(json_string.as_bytes())?;
        Ok(())
    }
}
