use super::super::super::core::clipboard::Clipboard;
use super::super::cursor::Cursor;
use super::super::cursor::CursorPosition;
use super::undo_redo::UndoRedoManager;
use crate::config::{Config, EditorConfig};
use crate::errors::editor_errors::EditorError::{
    ClipboardFailure, RedoFailure, TextSelectionFailure, UndoFailure,
};
use crate::errors::editor_errors::{ClipboardError, EditorError, TextSelectionError};
use crate::input::actions::{EditorAction, InputAction};
use std::fmt::Display;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum EditAction {
    // single-char operations
    Insert {
        pos: CursorPosition,
        c: char,
    },
    Delete {
        pos: CursorPosition,
        deleted_char: char,
    },
    Replace {
        start: CursorPosition,
        end: CursorPosition,
        old: char,
        new: char,
    },
    ReplaceRange {
        start: CursorPosition,
        end: CursorPosition,
        old: Vec<String>,
        new: Vec<String>,
    },
    // Insert lines of strings
    InsertLines {
        start: CursorPosition,
        lines: Vec<String>,
    },
    // Delete lines of strings
    DeleteLines {
        start: CursorPosition,
        deleted: Vec<String>,
    },
    // Insert strings from start position
    InsertRange {
        start: CursorPosition,
        end: CursorPosition,
        lines: Vec<String>,
    },
    // delete strings from start position
    DeleteRange {
        start: CursorPosition,
        end: CursorPosition,
        deleted: Vec<String>,
    },
    //line split operation, splits line in two and moves right line down
    SplitLine {
        pos: CursorPosition, // cursor position where the split happened
        left: String,        // text before the split
        right: String,       // text after the split
    },
    //line join operation
    JoinLine {
        pos: CursorPosition, // position of the split
        merged: String,      // the full merged text
    },
    // EditorAction for bulking together multiple editoractions into one,
    // Which makes performing undo once, undoes all actions inside bulk
    Bulk(Vec<EditAction>),
}

impl Display for EditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditAction::Insert { pos, c } => {
                write!(f, "Insert char '{}' at position {:?}", c, pos)
            }
            EditAction::Delete { pos, deleted_char } => {
                write!(f, "Delete char '{}' at position {:?}", deleted_char, pos)
            }
            EditAction::Replace {
                start,
                end,
                old,
                new,
            } => {
                write!(
                    f,
                    "Replace char '{}' with '{}' from {:?} to {:?}",
                    old, new, start, end
                )
            }
            EditAction::ReplaceRange {
                start,
                end,
                old,
                new,
            } => {
                let old_str = old.join("\\n");
                let new_str = new.join("\\n");
                write!(
                    f,
                    "Replace range '{:?}' with '{:?}' from {:?} to {:?}",
                    old_str, new_str, start, end
                )
            }
            EditAction::InsertLines { start, lines } => {
                write!(
                    f,
                    "Insert {} line(s) starting at {:?}: [{}]",
                    lines.len(),
                    start,
                    lines.join("\\n")
                )
            }
            EditAction::DeleteLines { start, deleted } => {
                write!(
                    f,
                    "Delete {} line(s) starting at {:?}: [{}]",
                    deleted.len(),
                    start,
                    deleted.join("\\n")
                )
            }
            EditAction::InsertRange { start, end, lines } => {
                let lines_str = lines.join("\\n");
                write!(
                    f,
                    "Insert range of length {} from {:?} to {:?}: [{}]",
                    lines_str.len(),
                    start,
                    end,
                    lines_str
                )
            }
            EditAction::DeleteRange {
                start,
                end,
                deleted,
            } => {
                let deleted_str = deleted.join("\\n");
                write!(
                    f,
                    "Delete range of length {} from {:?} to {:?}: [{}]",
                    deleted_str.len(),
                    start,
                    end,
                    deleted_str
                )
            }
            EditAction::SplitLine { pos, left, right } => {
                write!(
                    f,
                    "Split line at {:?}. Left: '{}', Right: '{}'",
                    pos, left, right
                )
            }
            EditAction::JoinLine { pos, merged } => {
                write!(f, "Join line at {:?}. Merged: '{}'", pos, merged)
            }
            EditAction::Bulk(actions) => {
                write!(f, "Executing {:?} Bulk actions", actions.len())
            }
        }
    }
}

/// handles editor content
#[derive(Debug)]
pub struct Editor {
    pub editor_content: Vec<String>,
    pub visual_cursor_x: i16,
    pub cursor: Cursor, //to save position in editor, when toggling area
    pub text_selection_start: Option<CursorPosition>,
    pub text_selection_end: Option<CursorPosition>,
    pub editor_width: i16,
    pub scroll_offset: i16,
    pub editor_height: u16,
    pub clipboard: Clipboard,
    pub undo_redo_manager: UndoRedoManager,

    //Cached config settings
    pub editor_config: Arc<EditorConfig>,
}

impl Editor {
    pub fn new(config: Arc<EditorConfig>) -> Self {
        Self {
            editor_content: vec![],
            visual_cursor_x: 0,
            text_selection_start: None,
            text_selection_end: None,
            cursor: Cursor::new(),
            editor_width: 0,
            scroll_offset: 0,
            editor_height: 0,
            clipboard: Clipboard::new(),
            undo_redo_manager: UndoRedoManager::new(config.undo_history_limit),
            editor_config: config,
        }
    }

    ///function to handle input action on editor,
    /// responsible for dispatching action to correct internal method.
    pub fn handle_input_action(&mut self, action: InputAction) -> Result<(), EditorError> {
        match action {
            InputAction::TAB => {
                self.tab();
                Ok(())
            }
            InputAction::ENTER => {
                self.enter();
                self.adjust_view_to_cursor();
                self.reset_text_selection_cursor(); //reset selection, to avoid errors
                Ok(())
            }
            InputAction::Editor(editor_action) => match editor_action {
                EditorAction::MoveCursor(direction) => {
                    let (x, y) = direction.to_vector();
                    self.move_cursor(x, y);
                    self.adjust_view_to_cursor();
                    self.reset_text_selection_cursor(); //reset selection, to avoid errors
                    Ok(())
                }
                EditorAction::MoveSelectionCursor(direction) => {
                    let (x, y) = direction.to_vector();
                    self.move_selection_cursor(x, y);
                    Ok(())
                }

                EditorAction::BACKSPACE => {
                    if self.is_text_selected() {
                        self.backspace_text_is_selected();
                    } else {
                        self.backspace();
                    }
                    Ok(())
                }
                EditorAction::DELETE => {
                    if self.is_text_selected() {
                        self.delete_text_is_selected();
                    } else {
                        self.delete();
                    }
                    Ok(())
                }
                EditorAction::COPY => match self.copy() {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e),
                },
                EditorAction::CUT => match self.cut() {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e),
                },
                EditorAction::PASTE => match self.paste() {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e),
                },
                EditorAction::UNDO => match self.undo() {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e),
                },
                EditorAction::REDO => match self.redo() {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e),
                },
                EditorAction::WriteChar(c) => {
                    if self.is_text_selected() {
                        self.write_char_text_is_selected(c)
                    } else {
                        self.write_char(c)
                    }
                    Ok(())
                }
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }

    //undo manager
    ///undo wrapper function, that calls the UndoRedoManager
    pub fn undo(&mut self) -> Result<(), EditorError> {
        match self.undo_redo_manager.undo() {
            Ok(action) => {
                self.apply_action(&action);
                Ok(())
            }
            Err(e) => Err(UndoFailure(e)),
        }
    }
    ///redo wrapper function, that calls the UndoRedoManager
    pub fn redo(&mut self) -> Result<(), EditorError> {
        match self.undo_redo_manager.redo() {
            Ok(action) => {
                self.apply_action(&action);
                Ok(())
            }
            Err(e) => Err(RedoFailure(e)),
        }
    }

    /// applies an EditAction
    fn apply_action(&mut self, action: &EditAction) {
        match action {
            EditAction::Insert { pos, c } => {
                self.insert_char_at(*pos, *c);
                let additive_pos = CursorPosition { x: pos.x + 1, y: 0 };
                let end: CursorPosition = *pos + additive_pos;
                self.set_cursor_position(&end);
            }
            EditAction::Delete { pos, .. } => {
                self.delete_char_at(*pos);
                self.set_cursor_position(pos);
            }
            EditAction::Replace {
                start, end, new, ..
            } => {
                self.replace_selection_with_char(*start, *end, *new);
                self.set_cursor_position(start);
            }
            EditAction::ReplaceRange {
                start,
                end,
                old,
                new,
            } => {
                // replace text from start..end with new
                self.replace_selection_with_lines(*start, *end, old.clone(), new.clone());

                // Calculate cursor position after replacement
                if new.is_empty() {
                    // If replacement is empty, cursor goes to start
                    self.set_cursor_position(start);
                } else {
                    // Cursor should be at the end of the last inserted line
                    let lines_added = new.len().saturating_sub(1); // 0 for single line, n-1 for multi
                    let last_line_len = new.last().map(|s| s.chars().count()).unwrap_or(0);

                    let new_pos = if new.len() == 1 {
                        // Single line replacement: add to start.x
                        CursorPosition {
                            x: start.x + last_line_len,
                            y: start.y,
                        }
                    } else {
                        // Multi-line replacement: cursor at end of last new line
                        CursorPosition {
                            x: last_line_len,
                            y: start.y + lines_added,
                        }
                    };

                    self.set_cursor_position(&new_pos);
                }
            }
            EditAction::InsertLines { start, lines } => {
                self.insert_lines_at(*start, lines.clone());
                //get additive position to get new cursor pos at end of insertion
                let last_line_len = lines.last().map(|s| s.len()).unwrap_or(0);
                let additive_pos = CursorPosition {
                    x: last_line_len,
                    y: lines.iter().count(),
                };
                let end: CursorPosition = *start + additive_pos;
                self.set_cursor_position(&end);
            } //in delete lines cursor position calculated first,
            // as visual x cannot be calculated without specific y line present
            EditAction::DeleteLines { start, deleted } => {
                //get additive position to get new cursor pos at end of insertion
                let last_line_len = deleted.last().map(|s| s.len()).unwrap_or(0);
                let negated_pos = CursorPosition {
                    x: last_line_len,
                    y: deleted.iter().count(),
                };
                let end: CursorPosition = *start - negated_pos;
                self.set_cursor_position(&end);

                self.delete_lines_at(*start, deleted.len());
            }
            EditAction::InsertRange {
                start,
                end: _end,
                lines,
            } => {
                self.insert_text_at(start, lines);
                //get additive position to get new cursor pos at end of insertion
                let last_line_len = lines.last().map(|s| s.len()).unwrap_or(0);
                let additive_pos = CursorPosition {
                    x: last_line_len,
                    y: lines.iter().count(),
                };
                let end: CursorPosition = *start + additive_pos;
                self.set_cursor_position(&end);
            }
            EditAction::DeleteRange {
                start,
                end,
                deleted: _deleted,
            } => {
                self.delete_text_at_range(start, end);

                self.set_cursor_position(start);
            }
            EditAction::SplitLine { pos, left, right } => {
                // overwrite current line with the left part
                self.editor_content[pos.y] = left.clone();

                // insert the right part as a new line
                self.editor_content.insert(pos.y + 1, right.clone());

                //at start of last line
                let new_pos = CursorPosition { x: 0, y: pos.y + 1 };

                self.set_cursor_position(&new_pos);
            }
            EditAction::JoinLine { pos, merged } => {
                // overwrite current line with the merged part
                self.editor_content[pos.y] = merged.clone();

                // remove next line
                self.editor_content.remove(pos.y + 1);

                self.set_cursor_position(&pos);
            }
            EditAction::Bulk(actions) => {
                for sub_action in actions {
                    self.apply_action(sub_action); // Recursive call
                }
            }
        }
    }

    //copy, cut and paste

    ///base function for copy that copies if text is selected
    pub fn copy(&mut self) -> Result<(), EditorError> {
        match self.copy_selected_text() {
            Ok(selected_text) => {
                //copy to clipboard
                self.clipboard.copy(&*selected_text);
                //reset text selection
                self.reset_text_selection_cursor();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    ///copies text within bound of text selected to copied_text
    pub fn copy_selected_text(&mut self) -> Result<Vec<String>, EditorError> {
        if let (Some(start), Some(end)) = (self.text_selection_start, self.text_selection_end) {
            let mut selected_text: Vec<String> = Vec::new();
            let lines = &self.editor_content[start.y..=end.y];

            if lines.len() > 1 {
                for (y, line) in lines.iter().enumerate() {
                    let mut line_chars: Vec<char> = line.chars().collect();
                    let extracted_text: String;

                    //if first line drain all from start x,
                    // else if last line drain to end .x, else drain all
                    if y == 0 {
                        extracted_text = line_chars.drain(start.x..).collect();
                    } else if y == lines.len() - 1 {
                        extracted_text = line_chars.drain(..end.x).collect();
                    } else {
                        extracted_text = line_chars.into_iter().collect();
                    }

                    selected_text.push(extracted_text);
                }
            } else {
                let mut line_chars: Vec<char> = self.editor_content[start.y].chars().collect();
                let extracted_text: String = line_chars.drain(start.x..end.x).collect();
                selected_text.push(extracted_text);
            }

            Ok(selected_text)
        } else {
            Err(TextSelectionFailure(TextSelectionError::NoTextSelected))
        }
    }

    ///base function for cut that cuts if text is selected
    pub fn cut(&mut self) -> Result<(), EditorError> {
        match self.cut_selected_text() {
            Ok(selected_text) => {
                //copy to clipboard
                self.clipboard.copy(&*selected_text);
                //reset text selection
                self.reset_text_selection_cursor();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    ///cuts text within bound of text selected to copied_text
    pub fn cut_selected_text(&mut self) -> Result<Vec<String>, EditorError> {
        if let (Some(start), Some(end)) = (self.text_selection_start, self.text_selection_end) {
            let mut selected_text: Vec<String> = Vec::new();
            let mut lines_to_remove: Vec<usize> = Vec::new(); //lines that should be removed
            let lines = self.editor_content[start.y..=end.y].as_mut();
            let line_length = lines.len();
            if lines.len() > 1 {
                // Multi-line cut
                for (y, line) in lines.iter_mut().enumerate() {
                    let mut line_chars: Vec<char> = line.as_mut().chars().collect();
                    let extracted_text: String;

                    //if first line drain all from start x,
                    // else if last line drain to end .x, else drain all

                    if y == 0 {
                        extracted_text = line_chars.drain(start.x..).collect();
                    } else if y == line_length - 1 {
                        extracted_text = line_chars.drain(..end.x).collect();
                    } else {
                        extracted_text = line_chars.drain(..).collect();
                    }
                    //if line manipulated is empty,
                    // means we want to delete it as well
                    if line_chars.is_empty() {
                        lines_to_remove.push(start.y + y);
                    }

                    selected_text.push(extracted_text);
                    *line = line_chars.into_iter().collect();
                }
            } else {
                // single-line cut
                let line = &mut self.editor_content[start.y];
                let mut line_chars: Vec<char> = line.as_mut().chars().collect();
                let extracted_text: String = line_chars.drain(start.x..end.x).collect();
                selected_text.push(extracted_text);

                *line = line_chars.into_iter().collect();
            }

            // remove fully cut lines, from last to first to avoid index shift
            for &y in lines_to_remove.iter().rev() {
                self.editor_content.remove(y);
            }

            //move content of last line selected to first line start point,
            // if any lines to remove
            if lines_to_remove.len() != 0 {
                // compute line index safely
                let merged_y = end.y.saturating_sub(lines_to_remove.len()) + 1;

                // ensure we do not access outside bounds
                if merged_y < self.editor_content.len() && start.y < self.editor_content.len() {
                    let line = self.editor_content.remove(merged_y);
                    if self.editor_content.len() <= start.y {
                        self.editor_content.push(line);
                    } else {
                        self.editor_content[start.y].push_str(&line);
                    }
                }
            }

            // record undo (DeleteRange)
            self.undo_redo_manager.record_undo(EditAction::DeleteRange {
                start,
                end,
                deleted: selected_text.clone(),
            });

            Ok(selected_text)
        } else {
            Err(TextSelectionFailure(TextSelectionError::NoTextSelected))
        }
    }

    ///base function for paste
    pub fn paste(&mut self) -> Result<(), EditorError> {
        match self.paste_selected_text(self.clipboard.copied_text.clone()) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    ///pastes text from copied text to editor content
    pub fn paste_selected_text(&mut self, copied_text: Vec<String>) -> Result<(), EditorError> {
        //if no text in copied text
        if copied_text.is_empty() {
            return Err(ClipboardFailure(ClipboardError::NoCopiedText));
        }

        let insert_y = self.cursor.y as usize;
        let insert_x = self.cursor.x as usize;

        while self.editor_content.len() <= insert_y {
            self.editor_content.push(String::new());
        }

        let current_line = &self.editor_content[insert_y];

        // Convert the line into a Vec<char> to handle multibyte characters correctly
        let chars: Vec<char> = current_line.chars().collect();
        let (before_cursor, after_cursor) = chars.split_at(insert_x.min(chars.len()));

        if copied_text.len() == 1 {
            // Single-line paste: correctly insert into character-safe split
            let new_line = format!(
                "{}{}{}",
                before_cursor.iter().collect::<String>(),
                copied_text[0],
                after_cursor.iter().collect::<String>()
            );
            self.editor_content[insert_y] = new_line;
        } else {
            // Multi-line paste
            let mut new_lines = Vec::new();

            // First line: insert copied text at cursor position
            new_lines.push(format!(
                "{}{}",
                before_cursor.iter().collect::<String>(),
                copied_text[0]
            ));

            // Middle lines: insert as separate lines
            for line in &copied_text[1..copied_text.len() - 1] {
                new_lines.push(line.clone());
            }

            // Last copied line + remainder of the original line
            let last_copied_line = &copied_text[copied_text.len() - 1];
            new_lines.push(format!(
                "{}{}",
                last_copied_line,
                after_cursor.iter().collect::<String>()
            ));

            // Replace the current line and insert new lines
            self.editor_content.splice(insert_y..=insert_y, new_lines);
        }

        let end: CursorPosition = CursorPosition {
            x: insert_x + copied_text.last().map(|s| s.chars().count()).unwrap_or(0),
            y: insert_y + copied_text.iter().count() - 1,
        };
        // record undo (InsertRange)
        self.undo_redo_manager.record_undo(EditAction::InsertRange {
            start: CursorPosition {
                y: insert_y,
                x: insert_x,
            },
            end,
            lines: copied_text.clone(),
        });

        Ok(())
    }

    //editor writing
    ///writes char to y position line, with x position
    pub fn write_char(&mut self, c: char) {
        //creating lines until y position of cursor
        while self.editor_content.len() <= self.cursor.y as usize {
            self.editor_content.push(String::new());
        }

        let line = &mut self.editor_content[self.cursor.y as usize];

        let char_count = line.chars().count();
        //position cursor to line end in chars count
        if char_count < self.cursor.x as usize {
            self.cursor.x = char_count as i16;
        }

        let mut line_chars_vec: Vec<char> = line.chars().collect();

        line_chars_vec.insert(self.cursor.x as usize, c);

        *line = line_chars_vec.into_iter().collect();
        //record undo action (action done)
        self.undo_redo_manager.record_undo(EditAction::Insert {
            pos: CursorPosition {
                x: self.cursor.x as usize,
                y: self.cursor.y as usize,
            },
            c,
        });

        self.move_cursor(1, 0);
    }

    ///replaces all selected text with char to y position line, with x position
    pub fn write_char_text_is_selected(&mut self, c: char) {
        let mut selected_text: Vec<String> = Vec::new();
        let start = self.text_selection_start.unwrap();
        let end = self.text_selection_end.unwrap();
        let lines = &mut self.editor_content[start.y..=end.y];
        let lines_length = lines.len();
        if lines_length > 1 {
            let mut line_indexes_to_remove: Vec<u16> = vec![];
            for (y, line) in lines.iter_mut().enumerate() {
                let mut line_chars_vec: Vec<char> = line.chars().collect();
                let deleted_text: String;
                //first line
                if y == 0 {
                    deleted_text = line_chars_vec.drain(start.x..).collect();
                    line_chars_vec.insert(start.x, c); //write chat to start position
                } else if y == lines_length - 1 {
                    //last line selected
                    deleted_text = line_chars_vec.drain(0..end.x).collect();
                } else {
                    deleted_text = line_chars_vec.drain(0..).collect();
                    line_indexes_to_remove.push((start.y + y) as u16);
                }
                selected_text.push(deleted_text);
                *line = line_chars_vec.into_iter().collect();
            }
            // remove the lines that became empty in reverse order
            for &i in line_indexes_to_remove.iter().rev() {
                self.editor_content.remove(i as usize);
            }
            //move content of last line selected to first line start point
            let line = &mut self
                .editor_content
                .remove(end.y - line_indexes_to_remove.len());
            self.editor_content[start.y].push_str(line);
        } else {
            let line = &mut self.editor_content[start.y];
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            let deleted_text: String = line_chars_vec.drain(start.x..end.x).collect();
            selected_text.push(deleted_text);
            line_chars_vec.insert(start.x, c);
            *line = line_chars_vec.into_iter().collect();
        }
        self.cursor.x = self.text_selection_start.unwrap().x as i16;
        self.cursor.y = self.text_selection_start.unwrap().y as i16;
        self.reset_text_selection_cursor();
        self.move_cursor(1, 0);

        // record undo (ReplaceRange)
        self.undo_redo_manager
            .record_undo(EditAction::ReplaceRange {
                start,
                end,
                old: selected_text.clone(),
                new: vec![c.to_string()],
            });
    }

    //editor tab
    /// Smart tab - auto-indents to match previous line or inserts tab
    /// by writing \t or spaces to editor content.
    pub fn tab(&mut self) {
        // If at start of line and previous line has indentation, match it
        if self.cursor.x == 0 && self.cursor.y > 0 {
            let prev_line = &self.editor_content[(self.cursor.y - 1) as usize];
            let indent = self.get_line_indent(prev_line);

            if !indent.is_empty() {
                let line = &mut self.editor_content[self.cursor.y as usize];
                line.insert_str(0, &indent);
                let len_of_indent = indent.chars().count();

                self.undo_redo_manager.record_undo(EditAction::InsertRange {
                    start: CursorPosition {
                        x: 0,
                        y: self.cursor.y as usize,
                    },
                    end: CursorPosition {
                        x: len_of_indent,
                        y: self.cursor.y as usize,
                    },
                    lines: vec![indent],
                });
                self.move_cursor(len_of_indent as i16, 0);
                return;
            }
        }

        // Otherwise, insert tab/spaces
        self.insert_tab_character();
    }

    fn insert_tab_character(&mut self) {
        let line = &mut self.editor_content[self.cursor.y as usize];
        let insert_pos = self.cursor.x as usize;
        let tab_width = self.editor_config.tab_width;
        if self.editor_config.use_spaces {
            let spaces = " ".repeat(tab_width as usize);
            line.insert_str(insert_pos, &spaces);

            self.undo_redo_manager.record_undo(EditAction::InsertRange {
                start: CursorPosition {
                    x: insert_pos,
                    y: self.cursor.y as usize,
                },
                end: CursorPosition {
                    x: insert_pos + spaces.len(),
                    y: self.cursor.y as usize,
                },
                lines: vec![spaces],
            });
            self.move_cursor(tab_width as i16, 0);
        } else {
            //since \t is special char, make sure to edit line based on chars in it not str
            //collect chars from line, insert tab char, and collect back to line again
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            line_chars_vec.insert(self.cursor.x as usize, '\t');
            *line = line_chars_vec.into_iter().collect();

            self.undo_redo_manager.record_undo(EditAction::Insert {
                pos: CursorPosition {
                    x: insert_pos,
                    y: self.cursor.y as usize,
                },
                c: '\t',
            });

            self.move_cursor(1, 0);
        }
    }

    /// Get indentation from line: leading whitespaces
    fn get_line_indent(&self, line: &str) -> String {
        line.chars().take_while(|c| c.is_whitespace()).collect()
    }

    //TODO ENTER SHOULD MOVE SCROLL OFFSET IF making new line at bottom defined by offset in config
    //editor enter
    ///handles enter new line, with possible move of text
    pub fn enter(&mut self) {
        let line = &mut self.editor_content[self.cursor.y as usize];
        //if at end of line len, then just move cursor and make new line, else move text too
        if self.cursor.x >= line.chars().count() as i16 {
            self.editor_content
                .insert(self.cursor.y as usize + 1, String::new());
            //record undo
            self.undo_redo_manager.record_undo(EditAction::InsertLines {
                start: CursorPosition {
                    x: self.cursor.x as usize,
                    y: self.cursor.y as usize + 1, //+1 y to insert after current index
                },
                lines: vec![String::new()],
            });
            self.move_cursor(0, 1);
        } else {
            //split current line and remove split part
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            let line_end = line_chars_vec.split_off(self.cursor.x as usize);
            let left: String = line_chars_vec.into_iter().collect();
            let right: String = line_end.clone().into_iter().collect();

            *line = left.clone();

            // insert new line under, move cursor and insert split line to line
            self.editor_content
                .insert(self.cursor.y as usize + 1, String::new());
            self.move_cursor(0, 1);
            self.editor_content[self.cursor.y as usize] = right.clone();
            //enter to split line, should go to start of line
            self.cursor.x = 0;
            self.visual_cursor_x = self.calculate_visual_x() as i16;
            // record undo
            self.undo_redo_manager.record_undo(EditAction::SplitLine {
                pos: CursorPosition {
                    x: left.len(),
                    y: (self.cursor.y - 1) as usize, // original line before split
                },
                left,
                right,
            });
        }
    }

    //editor backspace
    ///handles backspace in editor, removes char at y line x position and sets new cursor position
    pub fn backspace(&mut self) {
        let deleted_char: Option<char>;
        let y = self.cursor.y as usize;
        let x = self.cursor.x as usize;
        let line_char_count = self.editor_content[y].chars().count();
        //if x is more than 0 and less than max line index : should delete char and move back
        // else if y is more than 0, move line up
        if x > 0 && x <= line_char_count {
            let line = &mut self.editor_content[y];
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            let char = line_chars_vec.remove(x - 1);
            deleted_char = Some(char);

            *line = line_chars_vec.into_iter().collect();
            self.move_cursor(-1, 0);

            //record undo if any deleted char
            if let Some(char) = deleted_char {
                self.undo_redo_manager.record_undo(EditAction::Delete {
                    pos: CursorPosition {
                        x: self.cursor.x as usize,
                        y: self.cursor.y as usize,
                    },
                    deleted_char: char.clone(),
                });
            }
        } else if y > 0 {
            let line = &mut self.editor_content.remove(self.cursor.y as usize);
            let new_x_value = self.editor_content[(self.cursor.y - 1) as usize]
                .chars()
                .count() as i16;
            self.cursor.x = new_x_value;
            self.move_cursor(0, -1);
            self.editor_content[self.cursor.y as usize].push_str(line);
            let merged_line: String = self.editor_content[self.cursor.y as usize].clone();
            // Record the join action for undo
            self.undo_redo_manager.record_undo(EditAction::JoinLine {
                pos: CursorPosition {
                    x: self.cursor.x as usize,
                    y: self.cursor.y as usize,
                },
                merged: merged_line,
            });
        }
    }

    ///handles backspace in editor, removes char at y line x position and sets new cursor position
    pub fn backspace_text_is_selected(&mut self) {
        let mut selected_text: Vec<String> = Vec::new();
        let start = self.text_selection_start.unwrap();
        let end = self.text_selection_end.unwrap();
        let lines = &mut self.editor_content[start.y..=end.y];
        let lines_length = lines.len();
        if lines_length > 1 {
            let mut line_indexes_to_remove: Vec<u16> = vec![];
            for (y, line) in lines.iter_mut().enumerate() {
                let mut line_chars_vec: Vec<char> = line.chars().collect();
                let deleted_text: String;
                //first line
                if y == 0 {
                    deleted_text = line_chars_vec.drain(start.x..).collect();
                } else if y == lines_length - 1 {
                    //last line selected
                    deleted_text = line_chars_vec.drain(0..end.x).collect();
                } else {
                    deleted_text = line_chars_vec.drain(0..).collect();
                    line_indexes_to_remove.push((start.y + y) as u16);
                }

                selected_text.push(deleted_text);
                *line = line_chars_vec.into_iter().collect();
            }
            // remove the lines that became empty in reverse order
            for &i in line_indexes_to_remove.iter().rev() {
                self.editor_content.remove(i as usize);
            }
            //move content of last line selected to first line start point
            let line = &mut self
                .editor_content
                .remove(end.y - line_indexes_to_remove.len());
            self.editor_content[start.y].push_str(line);
        } else {
            let line = &mut self.editor_content[start.y];
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            let deleted_text: String = line_chars_vec.drain(start.x..end.x).collect();
            selected_text.push(deleted_text);

            *line = line_chars_vec.into_iter().collect();
        }
        self.cursor.x = self.text_selection_start.unwrap().x as i16;
        self.cursor.y = self.text_selection_start.unwrap().y as i16;
        self.reset_text_selection_cursor();
        //replace visual cursor
        self.visual_cursor_x = self.calculate_visual_x() as i16;

        // record undo (DeleteRange)
        self.undo_redo_manager.record_undo(EditAction::DeleteRange {
            start,
            end,
            deleted: selected_text.clone(),
        });
    }

    //editor delete functions

    ///handles DELETE action, of deleting char in editor at x +1 position
    pub(crate) fn delete(&mut self) {
        let current_line_len = self.editor_content[self.cursor.y as usize].chars().count() as i16;

        if current_line_len == 0 {
            return;
        }
        //if at line end, move line below up, else if current line length is bigger than current cursor x pos, remove char
        if self.cursor.x >= current_line_len - 1
            && self.editor_content.len() > (self.cursor.y + 1) as usize
        {
            let line = &mut self.editor_content.remove((self.cursor.y + 1) as usize);
            self.editor_content[self.cursor.y as usize].push_str(line);
            let merged_line: String = self.editor_content[self.cursor.y as usize].clone();
            // Record the join action for undo
            self.undo_redo_manager.record_undo(EditAction::JoinLine {
                pos: CursorPosition {
                    x: self.cursor.x as usize,
                    y: self.cursor.y as usize,
                },
                merged: merged_line,
            });
        } else if current_line_len > (self.cursor.x + 1) {
            let line = &mut self.editor_content[self.cursor.y as usize];
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            let char = line_chars_vec.remove(self.cursor.x as usize + 1);

            self.undo_redo_manager.record_undo(EditAction::Delete {
                pos: CursorPosition {
                    x: self.cursor.x as usize + 1,
                    y: self.cursor.y as usize,
                },
                deleted_char: char.clone(),
            });

            *line = line_chars_vec.into_iter().collect();
        }
    }

    ///handles delete in editor, removes char at y line x position and sets new cursor position
    pub fn delete_text_is_selected(&mut self) {
        let mut selected_text: Vec<String> = Vec::new();
        let start = self.text_selection_start.unwrap();
        let end = self.text_selection_end.unwrap();
        let lines = &mut self.editor_content[start.y..=end.y];
        let lines_length = lines.len();
        if lines_length > 1 {
            for (y, line) in lines.iter_mut().enumerate() {
                let mut line_chars_vec: Vec<char> = line.chars().collect();
                let deleted_text: String;
                //first line
                if y == 0 {
                    deleted_text = line_chars_vec.drain(start.x..).collect();
                } else if y == lines_length - 1 {
                    //last line selected
                    //line_chars_vec.drain(0..end.x);   this takes away the chars
                    //this solution replaces with whitespace
                    let mut deleted_chars: Vec<char> = vec![];
                    for i in 0..end.x.min(line_chars_vec.len()) {
                        deleted_chars.push(line_chars_vec[i]);
                        line_chars_vec[i] = ' ';
                    }
                    deleted_text = deleted_chars.clone().into_iter().collect();
                } else {
                    deleted_text = line_chars_vec.drain(0..).collect::<String>();
                }

                selected_text.push(deleted_text);
                *line = line_chars_vec.into_iter().collect();
            }
        } else {
            let line = &mut self.editor_content[start.y];
            let mut line_chars_vec: Vec<char> = line.chars().collect();
            let deleted_text: String = line_chars_vec[start.x..end.x].into_iter().collect();
            selected_text.push(deleted_text);

            line_chars_vec[start.x..end.x].fill(' ');
            *line = line_chars_vec.into_iter().collect();
        }
        self.cursor.x = self.text_selection_end.unwrap().x as i16;
        self.cursor.y = self.text_selection_end.unwrap().y as i16;
        self.reset_text_selection_cursor();
        //replace visual cursor
        self.visual_cursor_x = self.calculate_visual_x() as i16;

        let old_replaced_with_whitespaces: Vec<String> = selected_text
            .iter()
            .map(|line| {
                std::iter::repeat(' ')
                    .take(line.chars().count())
                    .collect::<String>()
            })
            .collect();
        // record undo (ReplaceRange)
        self.undo_redo_manager
            .record_undo(EditAction::ReplaceRange {
                start,
                end,
                old: selected_text.clone(),
                new: old_replaced_with_whitespaces,
            });
    }

    //editor cursor moving

    /// Moves the cursor in relation to editor content
    pub fn move_cursor(&mut self, x: i16, y: i16) {
        if self.cursor.y == 0 && y == -1 {
            return;
        }
        //if wanting to go beyond current length of editor
        while self.editor_content.len() <= (self.cursor.y + y) as usize {
            self.editor_content.push(String::new());
            //record undo
            self.undo_redo_manager.record_undo(EditAction::InsertLines {
                start: CursorPosition {
                    x: self.cursor.x as usize,
                    y: self.cursor.y as usize + 1, //+1 y to insert after current index
                },
                lines: vec![String::new()],
            });
        }

        let max_x_pos = self.editor_content[(self.cursor.y + y) as usize]
            .chars()
            .count() as i16;
        //let current_line = &self.editor.editor_content[self.editor.cursor.y as usize];

        // Moving Right →
        if x > 0 && self.cursor.x < max_x_pos {
            self.cursor.x += x;
        } else if x == 1
            && self.cursor.x >= self.editor_content[self.cursor.y as usize].chars().count() as i16
            && self.editor_content.len() > self.cursor.y as usize + 1
        {
            //else if end of line and more lines
            self.cursor.y += 1;
            self.cursor.x = 0;
            self.visual_cursor_x = self.calculate_visual_x() as i16;
            return;
        }

        // Moving Left ←
        if x < 0 && self.cursor.x > 0 {
            self.cursor.x += x;
        } else if self.cursor.x == 0 && x == -1 && self.cursor.y != 0 {
            //else if start of line and more lines
            self.cursor.y -= 1;
            self.cursor.x = self.editor_content[self.cursor.y as usize].chars().count() as i16;
            self.visual_cursor_x = self.calculate_visual_x() as i16;
            return;
        }

        /* let (top, bottom) = self.is_cursor_top_or_bottom_of_editor();
        //to offset scroll
        if (y == 1 && bottom) || (y == -1 && top) {
            self.scroll_offset = (self.scroll_offset + y).clamp(0, i16::MAX);
            return;
        }*/

        self.cursor.x = self.cursor.x.clamp(0, max_x_pos);
        self.cursor.y = (self.cursor.y + y).clamp(0, i16::MAX);
        self.visual_cursor_x = self.calculate_visual_x() as i16;
    }

    /// Moves selection cursor
    pub(crate) fn move_selection_cursor(&mut self, x: i16, y: i16) {
        let old_x = self.cursor.x;
        let old_y = self.cursor.y;
        self.move_cursor(x, y);
        let new_x = self.cursor.x;
        let new_y = self.cursor.y;

        let new_pos = CursorPosition {
            x: new_x as usize,
            y: new_y as usize,
        };
        let old_pos = CursorPosition {
            x: old_x as usize,
            y: old_y as usize,
        };

        if self.text_selection_start.is_none() {
            // Initialize selection start on first move
            self.text_selection_start = Some(old_pos);
        }

        if self.text_selection_end.is_none() {
            // Initialize selection end on first move
            self.text_selection_end = Some(old_pos);
        }

        let (at_start, at_end) = self.is_selection_cursor_start_or_end(old_pos);

        if x > 0 || y > 0 {
            // Moving right/down → Extend selection

            if at_start && !at_end {
                //is at start pos and should move start instead of end
                self.text_selection_start = Some(new_pos);
            } else if at_end && !at_start {
                //is at end pos and should move end instead of start
                self.text_selection_end = Some(new_pos);
            } else {
                //is at both start and end, should move end
                self.text_selection_end = Some(new_pos);
            }
        } else if x < 0 || y < 0 {
            // Moving left/up → Adjust start instead of resetting
            if at_start && !at_end {
                //is at start pos and should move start instead of end
                self.text_selection_start = Some(new_pos);
            } else if at_end && !at_start {
                //is at end pos and should move end instead of start
                self.text_selection_end = Some(new_pos);
            } else {
                //is at both start and end, should move start
                self.text_selection_start = Some(new_pos);
            }
        }
    }

    fn is_selection_cursor_start_or_end(&self, current_pos: CursorPosition) -> (bool, bool) {
        let start = current_pos.x == self.text_selection_start.unwrap().x
            && current_pos.y == self.text_selection_start.unwrap().y;
        let end = current_pos.x == self.text_selection_end.unwrap().x
            && current_pos.y == self.text_selection_end.unwrap().y;
        (start, end)
    }

    //SCROLL
    /// moves scroll offset and config defined scroll amount and scrolloff
    pub fn move_scroll_offset(&mut self, direction: i16) {
        let scroll_amount = (self.editor_config.scroll_lines as i16) * direction.signum();
        let scrolloff = self.editor_config.scrolloff as i16;
        let last_file_line = (self.editor_content.len() as i16 - 1).max(0);

        // Calculate viewport bounds with bottom margin
        let viewport_height = self.editor_height as i16;
        let max_scroll = self.calculate_max_scroll();

        // Calculate cursor position relative to viewport
        let cursor_viewport_pos = self.cursor.y - self.scroll_offset;

        // if direction > 0 = scrolling down
        if direction > 0 {
            // Check if cursor is near bottom of viewport
            if cursor_viewport_pos >= viewport_height - scrolloff - 1
                || self.cursor.y == last_file_line
            {
                let new_cursor_y =
                    (self.cursor.y + scroll_amount).min(self.editor_content.len() as i16 - 1);
                self.cursor.y = new_cursor_y;

                self.scroll_offset = (self.scroll_offset + scroll_amount).clamp(0, max_scroll);
            } else {
                self.cursor.y =
                    (self.cursor.y + scroll_amount).min(self.editor_content.len() as i16 - 1);
            }
        // if direction < 0 = scrolling up
        } else if direction < 0 {
            // Check if cursor is near top of viewport
            if cursor_viewport_pos <= scrolloff {
                let new_cursor_y = (self.cursor.y + scroll_amount).max(0);
                self.cursor.y = new_cursor_y;

                // Adjust scroll to keep cursor in view with scrolloff
                let desired_scroll = self.cursor.y - scrolloff;
                self.scroll_offset = desired_scroll.clamp(0, max_scroll);
            } else {
                self.cursor.y = (self.cursor.y + scroll_amount).max(0);
            }
        }
        // Clamping

        self.scroll_offset = self.scroll_offset.clamp(0, max_scroll);
        self.clamp_cursor_to_line();
    }

    /// Sets scroll offset to provided offset
    pub fn set_scroll_offset(&mut self, offset: i16) {
        self.scroll_offset = offset;
    }

    /// Adjusts view scroll offset to show cursor considering margin and scrolloff
    pub fn adjust_view_to_cursor(&mut self) {
        let scrolloff = self.editor_config.scrolloff as i16;
        let viewport_height = self.editor_height as i16;
        let cursor_v_pos = self.cursor.y - self.scroll_offset;

        if cursor_v_pos < scrolloff {
            self.scroll_offset = (self.cursor.y - scrolloff).max(0);
        } else if cursor_v_pos >= viewport_height - scrolloff {
            let max_scroll = self.calculate_max_scroll();
            self.scroll_offset = (self.cursor.y - viewport_height + scrolloff + 1).min(max_scroll);
        }
    }

    /// Calculate the maximum scroll offset with bottom margin
    fn calculate_max_scroll(&self) -> i16 {
        let viewport_height = self.editor_height as i16;
        let content_height = self.editor_content.len() as i16;
        let bottom_margin = self.editor_config.scroll_margin_bottom as i16;

        // Maximum scroll is content height minus viewport height, plus bottom margin
        // This allows scrolling past the end to show empty space
        (content_height - viewport_height + bottom_margin).max(0)
    }

    /// Ensure cursor X is within the current line bounds
    fn clamp_cursor_to_line(&mut self) {
        if self.cursor.y >= 0 && (self.cursor.y as usize) < self.editor_content.len() {
            let line_len = self.editor_content[self.cursor.y as usize].len() as i16;
            self.cursor.x = self.cursor.x.min(line_len);
        }
    }

    ///calculates the visual position of the cursor
    fn calculate_visual_x(&mut self) -> usize {
        let line = &self.editor_content[self.cursor.y as usize];
        let cursor_x = self.cursor.x as usize;
        let tab_width = self.editor_config.tab_width as usize;
        let mut visual_x = 0;
        for (i, c) in line.chars().enumerate() {
            if i == cursor_x {
                break;
            }

            if c == '\t' {
                visual_x += tab_width - (visual_x % tab_width);
            } else {
                visual_x += 1;
            }
        }

        visual_x
    }
}

//██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
//██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
//███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
//██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
//██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
//╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝

impl Editor {
    ///function to check if some text is selected
    fn is_text_selected(&self) -> bool {
        self.text_selection_start.is_some() && self.text_selection_end.is_some()
    }

    /// sets cursor position to specified position
    pub(crate) fn set_cursor_position(&mut self, pos: &CursorPosition) {
        //clamp set position to at maximum go to last position available
        //y is len of editor -1
        self.cursor.y = (pos.y as i16).clamp(0, self.editor_content.len() as i16 - 1);
        //length of cursor.y line , aka nr. of chars on line.
        let line_len = self
            .editor_content
            .get(self.cursor.y as usize)
            .map(|s| s.len())
            .unwrap_or(0);
        self.cursor.x = (pos.x as i16).clamp(0, line_len as i16);
        //calculate visual x pos again.
        self.visual_cursor_x = self.calculate_visual_x() as i16;
    }

    /// resets text selection cursor to none
    pub(crate) fn reset_text_selection_cursor(&mut self) {
        self.text_selection_start = None;
        self.text_selection_end = None;
    }
    /// Insert a character at the specified position (buffer-only: does not touch undo/redo, does not move main cursor)
    pub(crate) fn insert_char_at(&mut self, pos: CursorPosition, c: char) {
        // Ensure the target line exists
        while self.editor_content.len() <= pos.y {
            self.editor_content.push(String::new());
        }
        let line = &mut self.editor_content[pos.y];
        let mut chars: Vec<char> = line.chars().collect();
        // Clamp to actual line length
        let x = pos.x.min(chars.len());
        chars.insert(x, c);
        *line = chars.into_iter().collect();
        // cursor not updated
    }

    /// Delete a character at the specified position (buffer-only)
    pub(crate) fn delete_char_at(&mut self, pos: CursorPosition) {
        if let Some(line) = self.editor_content.get_mut(pos.y) {
            let mut chars: Vec<char> = line.chars().collect();
            if pos.x < chars.len() {
                chars.remove(pos.x);
                *line = chars.into_iter().collect();
            }
        }
    }

    /// Replace a text selection (from start to end) with new char
    pub(crate) fn replace_selection_with_char(
        &mut self,
        start: CursorPosition,
        end: CursorPosition,
        new: char,
    ) {
        if start.y == end.y {
            if let Some(line) = self.editor_content.get_mut(start.y) {
                let mut chars: Vec<char> = line.chars().collect();
                // Clamp positions
                let start_x = start.x.min(chars.len());
                let end_x = end.x.min(chars.len());
                chars.drain(start_x..end_x);
                chars.insert(start_x, new);
                *line = chars.into_iter().collect();
            }
        }
    }

    /// Replace a selection with lines of String
    fn replace_selection_with_lines(
        &mut self,
        start: CursorPosition,
        _end: CursorPosition,
        old_lines: Vec<String>,
        new_lines: Vec<String>,
    ) {
        // Safety check for empty document
        if self.editor_content.is_empty() {
            if !new_lines.is_empty() {
                self.editor_content = new_lines;
            } else {
                self.editor_content.push(String::new());
            }
            return;
        }

        // Clamp to valid indices
        let max_index = self.editor_content.len().saturating_sub(1);
        let start_y = start.y.min(max_index);
        //let end_y = end.y.min(max_index);

        // UTF-8 safe string splitter
        let split_line = |line: &str, index: usize| -> (String, String) {
            let chars: Vec<char> = line.chars().collect();
            let safe_index = index.min(chars.len());
            let (left, right) = chars.split_at(safe_index);
            (left.iter().collect(), right.iter().collect())
        };

        // === USE OLD CONTENT TO DETERMINE CONTEXTS ===

        let left_context: String;
        let right_context: String;

        if old_lines.is_empty() {
            // No old content - this is an insertion
            // Get contexts from current editor content
            let line = &self.editor_content[start_y];
            let (left, right) = split_line(line, start.x);
            left_context = left;
            right_context = right;
        } else if old_lines.len() == 1 {
            // Single line old content
            let current_line = &self.editor_content[start_y];
            let (left, _) = split_line(current_line, start.x);
            left_context = left;

            // Right context: current line after (start.x + old_line.len())
            let old_len = old_lines[0].chars().count();
            let (_, right) = split_line(current_line, start.x + old_len);
            right_context = right;
        } else {
            // Multi-line old content
            let first_line = &self.editor_content[start_y];
            let (left, _) = split_line(first_line, start.x);
            left_context = left;

            // Right context from end line
            let last_old_len = old_lines.last().map(|s| s.chars().count()).unwrap_or(0);
            let end_line_idx = start_y + old_lines.len() - 1;
            if end_line_idx < self.editor_content.len() {
                let end_line = &self.editor_content[end_line_idx];
                let (_, right) = split_line(end_line, last_old_len);
                right_context = right;
            } else {
                right_context = String::new();
            }
        }

        // Build the replacement lines
        let mut result = Vec::new();

        if new_lines.is_empty() {
            // Pure deletion - merge contexts
            result.push(format!("{}{}", left_context, right_context));
        } else if new_lines.len() == 1 {
            // Single line replacement
            result.push(format!("{}{}{}", left_context, new_lines[0], right_context));
        } else {
            // Multi-line replacement
            result.push(format!("{}{}", left_context, new_lines[0]));

            for i in 1..new_lines.len() - 1 {
                result.push(new_lines[i].clone());
            }

            result.push(format!(
                "{}{}",
                new_lines[new_lines.len() - 1],
                right_context
            ));
        }

        // Calculate how many lines to remove
        let lines_to_remove = if old_lines.is_empty() {
            // Insertion: split one line into multiple
            1
        } else {
            // Replacement: remove old_lines.len() lines (or the span from start_y to calculated end)
            old_lines.len()
        };

        // Remove the old lines (in reverse to avoid index issues)
        let end_remove = (start_y + lines_to_remove - 1).min(self.editor_content.len() - 1);
        for y in (start_y..=end_remove).rev() {
            if y < self.editor_content.len() {
                self.editor_content.remove(y);
            }
        }

        // Insert the new lines
        for (i, line) in result.into_iter().enumerate() {
            self.editor_content.insert(start_y + i, line);
        }
    }

    ///insert text lines at position without just inserting as lines,
    ///but if necessary, between already present text
    fn insert_text_at(&mut self, start: &CursorPosition, lines: &Vec<String>) {
        if lines.is_empty() {
            return;
        }

        // Ensure there's at least one line at start.y
        if start.y >= self.editor_content.len() {
            self.editor_content.resize(start.y + 1, String::new());
        }

        let original_line = self.editor_content[start.y].clone();

        // Special case: multi-line insert at column 0
        if lines.len() > 1 && start.x == 0 {
            for (i, s) in lines.iter().enumerate() {
                self.editor_content.insert(start.y + i, s.clone());
            }
            return;
        }

        // Split the original line
        let mut chars: Vec<char> = original_line.chars().collect();
        let x = start.x.min(chars.len());
        let first_line_right_half: String = chars.split_off(x).into_iter().collect();
        let first_line_left_half: String = chars.into_iter().collect();

        if lines.len() == 1 {
            // Single-line insert: left + insert + right
            self.editor_content[start.y] = format!(
                "{}{}{}",
                first_line_left_half, lines[0], first_line_right_half
            );
        } else {
            // Multi-line insert

            // Replace current line with left + first inserted line
            self.editor_content[start.y] = format!("{}{}", first_line_left_half, lines[0]);

            // Insert all remaining lines, merging the right half onto the last one
            let n_lines = lines.len();
            for i in 1..n_lines {
                let insertion_index = start.y + i;
                let line_to_insert = lines[i].clone();

                // if last line being inserted
                if i == n_lines - 1 {
                    // append the right half of the split original line
                    let final_content = format!("{}{}", line_to_insert, first_line_right_half);
                    self.editor_content.insert(insertion_index, final_content);
                } else {
                    // middle lines, insert directly
                    self.editor_content.insert(insertion_index, line_to_insert);
                }
            }
        }
    }

    ///delete text lines at start to end,
    fn delete_text_at_range(&mut self, start: &CursorPosition, end: &CursorPosition) {
        if start.y == end.y {
            // deleting within a single line
            if let Some(line) = self.editor_content.get_mut(start.y) {
                let mut chars: Vec<char> = line.chars().collect();
                let range_start = start.x.min(chars.len());
                let range_end = end.x.min(chars.len());
                if range_start < range_end {
                    chars.drain(range_start..range_end);
                    *line = chars.into_iter().collect();
                }
            }
        } else {
            // deleting across multiple lines

            // get suffix of last line
            let suffix = if let Some(last_line) = self.editor_content.get(end.y) {
                last_line.chars().skip(end.x).collect::<String>()
            } else {
                String::new()
            };

            if start.x == 0 {
                // delete the first line entirely
                self.editor_content.drain(start.y..=end.y);
                if !suffix.is_empty() {
                    self.editor_content.insert(start.y, suffix);
                }
            } else {
                // truncate first line
                if let Some(first_line) = self.editor_content.get_mut(start.y) {
                    let mut first_chars: Vec<char> = first_line.chars().collect();
                    first_chars.truncate(start.x);
                    *first_line = first_chars.into_iter().collect();
                    // append suffix from last line
                    first_line.push_str(&suffix);
                }

                // remove all lines in between start.y and end.y (excluding start.y)
                if end.y > start.y {
                    self.editor_content.drain(start.y + 1..=end.y);
                }
            }
        }
    }

    /// Insert multiple lines at a position
    pub(crate) fn insert_lines_at(&mut self, start: CursorPosition, lines: Vec<String>) {
        //calculate start y to insert
        let y = start.y.min(self.editor_content.len());
        for (i, line) in lines.into_iter().enumerate() {
            self.editor_content.insert(y + i, line);
        }
    }

    /// Delete lines starting at a position
    pub(crate) fn delete_lines_at(&mut self, start: CursorPosition, count: usize) {
        let y = start.y;
        for _ in 0..count {
            if y < self.editor_content.len() {
                self.editor_content.remove(y);
            } else {
                break;
            }
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new(Arc::new(Config::default().editor))
    }
}

//████████╗███████╗███████╗████████╗███████╗
//╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//   ██║   █████╗  ███████╗   ██║   ███████╗
//   ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//   ██║   ███████╗███████║   ██║   ███████║
//   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
#[cfg(test)]
mod unit_editor_write_tests {
    use super::super::super::cursor::CursorPosition;
    use super::super::editor::*;

    //init functions
    fn create_editor_with_editor_content(vec: Vec<String>) -> Editor {
        let config = Config::default();
        let mut editor = Editor::new(Arc::new(config.editor));
        editor.editor_content = vec;
        editor.editor_height = 10; //since testing doesnt start ui.rs, height isnt set
        editor
    }

    #[test]
    fn test_write_char() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.write_char('a');
        assert_eq!(editor.editor_content[0], "a");
        assert_eq!(editor.cursor.x, 1);
    }

    #[test]
    fn test_write_char_normal_characters() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.write_char('a');
        editor.write_char('b');
        editor.write_char('c');
        editor.write_char('d');
        assert_eq!(editor.editor_content[0], "abcd");
        assert_eq!(editor.cursor.x, 4);
    }

    #[test]
    fn test_write_char_special_characters() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.write_char('ᚠ');
        editor.write_char('Ω');
        editor.write_char('₿');
        editor.write_char('😎');
        assert_eq!(editor.editor_content[0], "ᚠΩ₿😎");
        assert_eq!(editor.cursor.x, 4);
    }

    #[test]
    fn test_write_char_at_line_10() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.cursor.y = 10;
        editor.write_char('a');
        assert_eq!(editor.editor_content[10], "a");
        assert_eq!(editor.cursor.x, 1);
    }

    #[test]
    fn test_write_char_at_100_x() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.cursor.x = 100;
        editor.write_char('a');
        assert_eq!(editor.editor_content[0], "a");
        assert_eq!(editor.cursor.x, 1);
    }

    //Write char to editor with selected text
    #[test]
    fn test_write_char_with_selected_text() {
        let mut editor = create_editor_with_editor_content(vec!["Hello Denmark".to_string()]);
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 0 });
        editor.cursor.x = 6;
        editor.write_char_text_is_selected('W');
        assert_eq!(editor.editor_content[0], "Hello W");
        assert_eq!(editor.cursor.x, 7);
    }

    #[test]
    fn test_write_char_with_selected_text_multiple_lines() {
        let mut editor = create_editor_with_editor_content(vec![
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 1 });
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 2 });
        editor.cursor.x = 6;
        editor.write_char_text_is_selected('W');
        assert_eq!(editor.editor_content[0], "test");
        assert_eq!(editor.editor_content[1], "Hello Wland");
        assert_eq!(editor.cursor.x, 7);
    }

    #[test]
    fn test_backspace_in_editor_text_is_selected_multiple_lines_4lines_middle_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec![
            "first line".to_string(),
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 2, y: 1 }); // middle of "test"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 3 }); // End of "sudeten"
                                                                          // Call the function to simulate a backspace with text selected
        editor.backspace_text_is_selected();

        assert_eq!(editor.editor_content.len(), 2);

        assert_eq!(editor.editor_content[0], "first line");
        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[1], "teland");

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 2);
        assert_eq!(editor.cursor.y, 1);
    }

    #[test]
    fn test_write_char_with_selected_text_special_characters() {
        let mut editor = create_editor_with_editor_content(vec!["ᚠΩ₿😎".to_string()]);
        editor.text_selection_start = Some(CursorPosition { x: 1, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 2, y: 0 });
        editor.cursor.x = 1;

        editor.write_char_text_is_selected('a');
        assert_eq!(editor.editor_content[0], "ᚠa₿😎");
        assert_eq!(editor.cursor.x, 2);
    }

    //TAB in editor
    #[test]
    fn test_tab_in_editor_start_of_empty_line() {
        let config = Config::default();
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);
        editor.tab();

        assert_eq!(editor.cursor.y, 0); // Cursor should stay on line
        assert_eq!(editor.editor_content.len(), 1); // New line added
        assert_eq!(editor.visual_cursor_x, config.editor.tab_width as i16);
    }

    #[test]
    fn test_tab_in_editor_start_of_line() {
        let config = Config::default();
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);
        editor.tab();

        assert_eq!(editor.cursor.y, 0); // Cursor should stay on line
        assert_eq!(editor.editor_content.len(), 1); // New line added
        assert_eq!(editor.visual_cursor_x, config.editor.tab_width as i16);
    }

    #[test]
    fn test_tab_in_editor_mid_of_line_normal_characters() {
        let mut editor = create_editor_with_editor_content(vec!["1234".to_string()]);
        editor.cursor.x = 2;
        editor.tab();

        assert_eq!(editor.cursor.y, 0); // Cursor should stay on line
        assert_eq!(editor.editor_content.len(), 1); // New line added
        assert_eq!(editor.visual_cursor_x, 4);
        editor.move_cursor(10, 0); //move to end
        assert_eq!(editor.editor_content[0].chars().count(), 5); //should contain special plus \t char
        assert_eq!(editor.visual_cursor_x, 6); //at end of line should be 6
    }

    #[test]
    fn test_tab_in_editor_mid_of_line_special_characters() {
        let mut editor = create_editor_with_editor_content(vec!["ᚠΩ₿😎".to_string()]);
        editor.cursor.x = 2;
        editor.tab();

        assert_eq!(editor.cursor.y, 0); // Cursor should stay on line
        assert_eq!(editor.editor_content.len(), 1); // New line added
        assert_eq!(editor.visual_cursor_x, 4);
        editor.move_cursor(10, 0); //move to end
        assert_eq!(editor.editor_content[0].chars().count(), 5); //should contain special plus \t char
        assert_eq!(editor.visual_cursor_x, 6); //at end of line should be 6
    }

    //ENTER in editor

    #[test]
    fn test_enter_in_editor_at_end_of_line() {
        let mut editor = create_editor_with_editor_content(vec!["Hello World".to_string()]);
        editor.cursor.x = editor.editor_content[0].len() as i16; // Set cursor to end of line
        editor.enter();

        assert_eq!(editor.cursor.y, 1); // Cursor should move to the next line
        assert_eq!(editor.editor_content.len(), 2); // New line added
        assert_eq!(editor.editor_content[1], ""); // New line should be empty
    }

    #[test]
    fn test_enter_in_editor_mid_line() {
        let mut editor = create_editor_with_editor_content(vec!["Hello World".to_string()]);
        editor.cursor.x = 5; // Split the line at index 5
        editor.enter();

        assert_eq!(editor.cursor.y, 1); // Cursor should move to next line
        assert_eq!(editor.cursor.x, 0); // Cursor resets to start of new line
        assert_eq!(editor.editor_content[0], "Hello"); // Line before cursor is kept intact
        assert_eq!(editor.editor_content[1], " World"); // Line after cursor is moved to new line
    }
}
#[cfg(test)]
mod unit_editor_delete_tests {
    use super::super::super::cursor::CursorPosition;
    use super::super::editor::*;

    fn create_editor_with_editor_content(vec: Vec<String>) -> Editor {
        let mut editor = Editor::new(Arc::new(Config::default().editor));
        editor.editor_content = vec;
        editor.editor_height = 10; //since testing doesnt start ui.rs, height isnt set
        editor
    }

    //BACKSPACE IN EDITOR
    #[test]
    fn test_backspace_in_editor() {
        let mut editor = create_editor_with_editor_content(vec!['a'.to_string()]);
        editor.cursor.x = 1;
        editor.backspace();
        assert_eq!(editor.editor_content[0], "");
        assert_eq!(editor.cursor.x, 0);
    }

    #[test]
    fn test_backspace_in_editor_special_characters() {
        let mut editor = create_editor_with_editor_content(vec!["ᚠΩ₿😎".to_string()]);
        editor.cursor.x = 4;
        editor.backspace();
        assert_eq!(editor.editor_content[0], "ᚠΩ₿");
        assert_eq!(editor.cursor.x, 3);
    }

    #[test]
    fn test_backspace_in_editor_should_go_to_previous_line() {
        let mut editor = create_editor_with_editor_content(vec!["a".to_string(), "b".to_string()]);
        editor.cursor.y = 1;
        editor.cursor.x = 0;
        editor.backspace();
        assert_eq!(editor.editor_content[0], "ab");
        assert_eq!(editor.editor_content.len(), 1);
        assert_eq!(editor.cursor.x, 1);
        assert_eq!(editor.cursor.y, 0);
    }

    //TEXT IS SELECTED

    #[test]
    fn test_backspace_in_editor_text_is_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec!["Hello Denmark".to_string()]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 }); // Start of "Denmark"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 0 }); // End of "Denmark"
                                                                          // Call the function to simulate a backspace with text selected
        editor.backspace_text_is_selected();

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content, vec!["Hello "]);

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 6);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_backspace_in_editor_text_is_selected_multiple_lines() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec![
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 1 }); // Start of "Denmark"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 2 }); // End of "sudeten"
                                                                          // Call the function to simulate a backspace with text selected
        editor.backspace_text_is_selected();

        assert_eq!(editor.editor_content.len(), 2);

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[0], "test");
        assert_eq!(editor.editor_content[1], "Hello land");

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 6);
        assert_eq!(editor.cursor.y, 1);
    }

    #[test]
    fn test_backspace_in_editor_text_is_selected_multiple_lines_4lines_middle_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec![
            "first line".to_string(),
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 2, y: 1 }); // middle of "test"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 3 }); // End of "sudeten"
                                                                          // Call the function to simulate a backspace with text selected
        editor.backspace_text_is_selected();

        assert_eq!(editor.editor_content.len(), 2);

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[1], "teland");

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 2);
        assert_eq!(editor.cursor.y, 1);
    }

    #[test]
    fn test_backspace_in_editor_text_is_selected_empty_text() {
        // Initialize the editor with empty content
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);

        // Set a selection range (even though the text is empty)
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 0, y: 0 });

        // Call the function to simulate a backspace with empty text
        editor.backspace_text_is_selected();

        // Assert that the text remains empty
        assert_eq!(editor.editor_content, vec!["".to_string()]);

        // Assert that the selection is cleared
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor position is 0
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_backspace_in_editor_text_is_selected_full_text_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec!["Hello Denmark".to_string()]);

        // Set a selection range for the entire text
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 0 });

        // Call the function to simulate a backspace with the entire text selected
        editor.backspace_text_is_selected();

        // Assert that all text is removed
        assert_eq!(editor.editor_content, vec!["".to_string()]);

        // Assert that the selection is cleared
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor position is 0
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    //DELETE IN EDITOR
    #[test]
    fn test_delete_in_editor() {
        let mut editor = create_editor_with_editor_content(vec!["ab".to_string()]);
        editor.cursor.x = 0;
        editor.delete();
        assert_eq!(editor.editor_content[0], "a");
        assert_eq!(editor.cursor.x, 0);
    }

    #[test]
    fn test_delete_in_editor_special_characters() {
        let mut editor = create_editor_with_editor_content(vec!["ᚠΩ₿😎".to_string()]);
        editor.cursor.x = 2;
        editor.delete();
        assert_eq!(editor.editor_content[0], "ᚠΩ₿");
        assert_eq!(editor.cursor.x, 2);
    }

    #[test]
    fn test_delete_in_editor_should_go_to_previous_line() {
        let mut editor = create_editor_with_editor_content(vec!["a".to_string(), "b".to_string()]);
        editor.cursor.x = 1;
        editor.delete();
        assert_eq!(editor.editor_content[0], "ab");
        assert_eq!(editor.editor_content.len(), 1);
        assert_eq!(editor.cursor.x, 1);
    }

    //TEXT IS SELECTED

    #[test]
    fn test_delete_in_editor_text_is_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec!["Hello Denmark".to_string()]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 0 }); // Start of "Denmark"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 0 }); // End of "Denmark"
                                                                          // Call the function to simulate a backspace with text selected
        editor.delete_text_is_selected();

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[0], "Hello        ");
        assert_eq!(editor.editor_content[0].len(), 13);

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 13);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_delete_in_editor_text_is_selected_multiple_lines() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec![
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 6, y: 1 }); // Start of "Denmark"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 2 }); // End of "Denmark"
                                                                          // Call the function to simulate a backspace with text selected
        editor.delete_text_is_selected();

        assert_eq!(editor.editor_content.len(), 3);

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[0], "test");
        assert_eq!(editor.editor_content[1], "Hello ");
        assert_eq!(editor.editor_content[2].len(), 17);

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 13);
        assert_eq!(editor.cursor.y, 2);
    }

    #[test]
    fn test_delete_in_editor_text_is_selected_multiple_lines_4lines_middle_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec![
            "first line".to_string(),
            "test".to_string(),
            "Hello Denmark".to_string(),
            "Hello Sudetenland".to_string(),
        ]);

        // Set a selection range (e.g., "Denmark")
        editor.text_selection_start = Some(CursorPosition { x: 2, y: 1 }); // middle of "test"
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 3 }); // End of "sudeten"
                                                                          // Call the function to simulate a backspace with text selected
        editor.delete_text_is_selected();

        assert_eq!(editor.editor_content.len(), 4);

        // Assert that the selected text is removed
        assert_eq!(editor.editor_content[1], "te");
        assert_eq!(editor.editor_content[2], "");
        assert_eq!(editor.editor_content[3], "             land");

        // Assert that the selection is cleared after the operation
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor is moved to the correct position
        assert_eq!(editor.cursor.x, 13);
        assert_eq!(editor.cursor.y, 3);
    }

    #[test]
    fn test_delete_in_editor_text_is_selected_empty_text() {
        // Initialize the editor with empty content
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);

        // Set a selection range (even though the text is empty)
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 0, y: 0 });

        // Call the function to simulate a backspace with empty text
        editor.delete_text_is_selected();

        // Assert that the text remains empty
        assert_eq!(editor.editor_content, vec!["".to_string()]);

        // Assert that the selection is cleared
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor position is 0
        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_delete_in_editor_text_is_selected_full_text_selected() {
        // Initialize the editor with some content
        let mut editor = create_editor_with_editor_content(vec!["Hello Denmark".to_string()]);

        // Set a selection range for the entire text
        editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
        editor.text_selection_end = Some(CursorPosition { x: 13, y: 0 });

        // Call the function to simulate a backspace with the entire text selected
        editor.delete_text_is_selected();

        // Assert that all text is removed
        assert_eq!(editor.editor_content[0].len(), 13);

        // Assert that the selection is cleared
        assert!(editor.text_selection_start.is_none());
        assert!(editor.text_selection_end.is_none());

        // Assert that the cursor position is 0
        assert_eq!(editor.cursor.x, 13);
        assert_eq!(editor.cursor.y, 0);
    }
}

#[cfg(test)]
mod unit_editor_cursor_tests {
    use super::super::editor::*;

    fn create_editor_with_editor_content(vec: Vec<String>) -> Editor {
        let mut editor = Editor::new(Arc::new(Config::default().editor));
        editor.editor_content = vec;
        editor.editor_height = 10; //since testing doesnt start ui.rs, height isnt set
        editor
    }

    #[test]
    fn test_cursor_move_right_within_line() {
        let mut editor = create_editor_with_editor_content(vec!["Hello World".to_string()]);
        editor.move_cursor(1, 0);

        assert_eq!(editor.cursor.x, 1);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_cursor_move_left_at_start_should_stay() {
        let mut editor = create_editor_with_editor_content(vec!["Hello World".to_string()]);
        editor.move_cursor(-1, 0);

        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_cursor_move_right_within_empty_line_should_stay() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.move_cursor(1, 0);

        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_cursor_move_right_at_end_of_first_line_should_move_down() {
        let mut editor =
            create_editor_with_editor_content(vec!["First".to_string(), "Second".to_string()]);
        editor.cursor.x = 5;
        editor.move_cursor(1, 0);

        assert_eq!(editor.cursor.x, 0);
        assert_eq!(editor.cursor.y, 1);
    }

    #[test]
    fn test_cursor_move_right_at_end_of_first_line_should_stay() {
        let mut editor = create_editor_with_editor_content(vec!["First".to_string()]);
        editor.cursor.x = 5;
        editor.move_cursor(1, 0);

        assert_eq!(editor.cursor.x, 5);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn test_cursor_move_down() {
        let mut editor = create_editor_with_editor_content(vec!["Second Line".to_string()]);
        editor.move_cursor(0, 1);

        assert_eq!(editor.cursor.x, 0); // Cursor stays at column 0
        assert_eq!(editor.cursor.y, 1); // Moves to the second line
    }

    //SELECTION CURSOR

    #[test]
    fn test_selection_cursor_move_up_should_stay() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.move_selection_cursor(0, -1);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }

    #[test]
    fn test_selection_cursor_move_down_go_down() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.move_selection_cursor(0, 1);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 1);
    }

    #[test]
    fn test_selection_cursor_move_left_should_stay() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.move_selection_cursor(-1, 0);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }

    #[test]
    fn test_selection_cursor_move_right_should_stay() {
        let mut editor = create_editor_with_editor_content(vec![]);
        editor.move_selection_cursor(1, 0);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }

    #[test]
    fn test_selection_cursor_move_up_should_go_up() {
        let mut editor =
            create_editor_with_editor_content(vec!["First".to_string(), "Second".to_string()]);
        editor.cursor.y = 1;
        editor.move_selection_cursor(0, -1);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 1);
    }

    #[test]
    fn test_selection_cursor_move_down_should_go_down() {
        let mut editor =
            create_editor_with_editor_content(vec!["First".to_string(), "Second".to_string()]);
        editor.move_selection_cursor(0, 1);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 0);
        assert_eq!(editor.text_selection_end.unwrap().y, 1);
    }

    #[test]
    fn test_selection_cursor_move_left_should_go_left() {
        let mut editor = create_editor_with_editor_content(vec!["First".to_string()]);
        editor.cursor.x = 1;
        editor.move_selection_cursor(-1, 0);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 1);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }

    #[test]
    fn test_selection_cursor_move_right_should_go_right() {
        let mut editor = create_editor_with_editor_content(vec!["First".to_string()]);
        editor.move_selection_cursor(1, 0);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 1);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }

    #[test]
    fn test_selection_cursor_move_right_thrice_should_go_right() {
        let mut editor = create_editor_with_editor_content(vec!["First".to_string()]);
        editor.move_selection_cursor(1, 0);
        editor.move_selection_cursor(1, 0);
        editor.move_selection_cursor(1, 0);

        assert_eq!(editor.text_selection_start.unwrap().x, 0);
        assert_eq!(editor.text_selection_start.unwrap().y, 0);
        assert_eq!(editor.text_selection_end.unwrap().x, 3);
        assert_eq!(editor.text_selection_end.unwrap().y, 0);
    }
}
#[cfg(test)]
mod unit_editor_cutcopy_tests {
    use crate::config::Config;
    use crate::core::cursor::CursorPosition;
    use crate::core::editor::Editor;
    use std::sync::Arc;

    fn create_editor_with_editor_content(vec: Vec<String>) -> Editor {
        let mut editor = Editor::new(Arc::new(Config::default().editor));
        editor.editor_content = vec;
        editor.editor_height = 10; //since testing doesnt start ui.rs, height isnt set
        editor
    }

    //copy selected text
    #[test]
    fn test_copy_single_line_selection() {
        let mut app = create_editor_with_editor_content(vec!["Hello, world!".to_string()]);
        app.text_selection_start = Some(CursorPosition { x: 7, y: 0 });
        app.text_selection_end = Some(CursorPosition { x: 12, y: 0 });

        let result = app.copy();

        assert!(result.is_ok());
        assert_eq!(app.clipboard.copied_text, vec!["world".to_string()]);
    }

    #[test]
    fn test_copy_multi_line_selection() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello,".to_string(),
            " world!".to_string(),
            " Rust".to_string(),
        ]);
        app.text_selection_start = Some(CursorPosition { x: 4, y: 0 });
        app.text_selection_end = Some(CursorPosition { x: 3, y: 2 });

        let result = app.copy();

        assert!(result.is_ok());
        assert_eq!(
            app.clipboard.copied_text,
            vec!["o,", " world!", " Ru"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_copy_no_selection() {
        let mut app = create_editor_with_editor_content(vec!["Hello, world!".to_string()]);
        app.reset_text_selection_cursor();

        let result = app.copy();

        assert!(result.is_err());
        assert!(app.clipboard.copied_text.is_empty());
    }

    //cut selected text
    #[test]
    fn test_cut_single_line_selection() {
        let mut app = create_editor_with_editor_content(vec!["Hello, world!".to_string()]);
        app.text_selection_start = Some(CursorPosition { x: 7, y: 0 });
        app.text_selection_end = Some(CursorPosition { x: 12, y: 0 });

        let result = app.cut();

        assert!(result.is_ok());
        assert_eq!(app.clipboard.copied_text, vec!["world".to_string()]);
        assert!(app.text_selection_start.is_none());
        assert!(app.text_selection_end.is_none());
    }

    #[test]
    fn test_cut_multi_line_selection() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello,".to_string(),
            " world!".to_string(),
            " Rust".to_string(),
        ]);
        app.text_selection_start = Some(CursorPosition { x: 4, y: 0 });
        app.text_selection_end = Some(CursorPosition { x: 3, y: 2 });

        let result = app.cut();

        assert!(result.is_ok());
        assert_eq!(
            app.clipboard.copied_text,
            vec!["o,", " world!", " Ru"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>()
        );
        assert!(app.text_selection_start.is_none());
        assert!(app.text_selection_end.is_none());
    }

    #[test]
    fn test_cut_no_selection() {
        let mut app = create_editor_with_editor_content(vec!["Hello, world!".to_string()]);
        app.reset_text_selection_cursor();

        let result = app.cut();

        assert!(result.is_err());
        assert!(app.clipboard.copied_text.is_empty());
        assert!(app.text_selection_start.is_none());
        assert!(app.text_selection_end.is_none());
    }

    //paste selected text
    #[test]
    fn test_paste_single_line() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["PASTED".to_string()]);
        app.cursor.x = 8;
        app.cursor.y = 0;

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, wPASTEDorld!".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_multiline() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard
            .copy(&vec!["First".to_string(), "Second ".to_string()]);
        app.cursor.x = 5;
        app.cursor.y = 1;

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This First".to_string(),
                "Second is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_single_line_special_characters() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, wᚠᚠᚠᚠorld!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["PASTED".to_string()]);
        app.cursor.x = 10;
        app.cursor.y = 0;

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, wᚠᚠPASTEDᚠᚠorld!".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_multiline_special_charaters() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This ᚠᚠᚠᚠis a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard
            .copy(&vec!["First".to_string(), "Second ".to_string()]);
        app.cursor.x = 7;
        app.cursor.y = 1;

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This ᚠᚠFirst".to_string(),
                "Second ᚠᚠis a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_at_start_of_line() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["NewStart".to_string()]);
        app.cursor.x = 0;
        app.cursor.y = 2;

        app.paste().unwrap();

        assert_eq!(
            app.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This is a test.".to_string(),
                "NewStartAnother line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_at_end_of_line() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec!["END".to_string()]);
        app.cursor.x = 13;
        app.cursor.y = 0;

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, world!END".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_with_empty_copied_text() {
        let mut app = create_editor_with_editor_content(vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "Another line.".to_string(),
        ]);
        app.clipboard.copy(&vec![]);
        app.cursor.x = 5;
        app.cursor.y = 1;

        assert!(app.paste().is_err());
        assert_eq!(
            app.editor_content,
            vec![
                "Hello, world!".to_string(),
                "This is a test.".to_string(),
                "Another line.".to_string(),
            ]
        );
    }

    #[test]
    fn test_paste_into_empty_editor() {
        let mut app = create_editor_with_editor_content(vec![]);
        app.clipboard
            .copy(&vec!["Hello".to_string(), "World".to_string()]);

        app.paste().unwrap();
        assert_eq!(
            app.editor_content,
            vec!["Hello".to_string(), "World".to_string()]
        );
    }
}
#[cfg(test)]
mod unit_editor_undoredo_tests {
    use super::super::super::cursor::CursorPosition;
    use super::super::editor::EditAction;
    use super::super::editor::Editor;
    use crate::config::Config;
    use std::sync::Arc;

    //init functions
    fn create_editor_with_editor_content(vec: Vec<String>) -> Editor {
        let mut editor = Editor::new(Arc::new(Config::default().editor));
        editor.editor_content = vec;
        editor.editor_height = 10; //since testing doesnt start ui.rs, height isnt set
        editor
    }
    // ========== Insert ==========
    #[test]
    fn undo_redo_insert_at_start() {
        let mut editor = create_editor_with_editor_content(vec!["xyz".to_string()]);
        editor.cursor.x = 0;
        editor.write_char('A');
        assert_eq!(editor.editor_content[0], "Axyz");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "xyz");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "Axyz");
    }

    #[test]
    fn undo_redo_insert_at_end() {
        let mut editor = create_editor_with_editor_content(vec!["foo".to_string()]);
        editor.cursor.x = 3;
        editor.write_char('B');
        assert_eq!(editor.editor_content[0], "fooB");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "foo");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "fooB");
    }

    #[test]
    fn undo_redo_multiple_insert_sequence() {
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);
        for ch in ['h', 'e', 'l', 'l', 'o'] {
            editor.write_char(ch);
        }
        assert_eq!(editor.editor_content[0], "hello");
        for _ in 0..5 {
            editor.undo().unwrap();
        }
        assert_eq!(editor.editor_content[0], "");
        for _ in 0..5 {
            editor.redo().unwrap();
        }
        assert_eq!(editor.editor_content[0], "hello");
    }

    // ========== Delete ==========
    #[test]
    fn undo_redo_delete_middle_char() {
        let mut editor = create_editor_with_editor_content(vec!["abcde".to_string()]);
        editor.cursor.x = 3;
        editor.backspace(); // remove 'c'
        assert_eq!(editor.editor_content[0], "abde");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "abcde");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "abde");
    }

    #[test]
    fn undo_redo_delete_last_char() {
        let mut editor = create_editor_with_editor_content(vec!["test".to_string()]);
        editor.cursor.x = 4;
        editor.backspace(); // remove 't'
        assert_eq!(editor.editor_content[0], "tes");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "test");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "tes");
    }

    #[test]
    fn undo_redo_delete_first_char() {
        let mut editor = create_editor_with_editor_content(vec!["tak".to_string()]);
        editor.cursor.x = 1;
        editor.backspace(); // remove 't'
        assert_eq!(editor.editor_content[0], "ak");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "tak");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "ak");
    }

    // ========== Replace ==========
    #[test]
    fn undo_redo_single_replace() {
        let mut editor = create_editor_with_editor_content(vec!["foo".to_string()]);
        // Simulate replace: overwrite 'o' (at 2..3) with 'x'
        let start = CursorPosition { x: 2, y: 0 };
        let end = CursorPosition { x: 3, y: 0 };
        let old = 'o';
        let new = 'x';
        editor.undo_redo_manager.record_undo(EditAction::Replace {
            start,
            end,
            old: old.clone(),
            new: new.clone(),
        });
        editor.editor_content[0].replace_range(2..3, "x");
        assert_eq!(editor.editor_content[0], "fox");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "foo");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "fox");
    }

    // ========== InsertLines ==========
    #[test]
    fn undo_redo_insert_lines_middle() {
        let mut editor =
            create_editor_with_editor_content(vec!["zero".to_string(), "three".to_string()]);
        let lines = vec!["one".to_string(), "two".to_string()];
        let pos = CursorPosition { x: 0, y: 1 };
        editor
            .undo_redo_manager
            .record_undo(EditAction::InsertLines {
                start: pos,
                lines: lines.clone(),
            });
        editor.editor_content.splice(1..1, lines.clone());
        assert_eq!(editor.editor_content, vec!["zero", "one", "two", "three"]);
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["zero", "three"]);
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["zero", "one", "two", "three"]);
    }

    #[test]
    fn undo_redo_insert_lines_at_start_and_end() {
        let mut editor = create_editor_with_editor_content(vec!["mid".to_string()]);
        let start_lines = vec!["a".to_string(), "b".to_string()];
        let end_lines = vec!["x".to_string(), "y".to_string()];
        // Insert at start
        let pos_start = CursorPosition { x: 0, y: 0 };
        editor
            .undo_redo_manager
            .record_undo(EditAction::InsertLines {
                start: pos_start,
                lines: start_lines.clone(),
            });
        editor.editor_content.splice(0..0, start_lines.clone());
        // Insert at end
        let pos_end = CursorPosition { x: 0, y: 3 };
        editor
            .undo_redo_manager
            .record_undo(EditAction::InsertLines {
                start: pos_end,
                lines: end_lines.clone(),
            });
        editor.editor_content.splice(3..3, end_lines.clone());
        assert_eq!(editor.editor_content, vec!["a", "b", "mid", "x", "y"]);
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "b", "mid"]);
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["mid"]);
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "b", "mid"]);
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "b", "mid", "x", "y"]);
    }

    // ========== DeleteLines ==========
    #[test]
    fn undo_redo_delete_lines_range() {
        let mut editor = create_editor_with_editor_content(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ]);
        // Remove b,c,d
        let removed = vec!["b".to_string(), "c".to_string(), "d".to_string()];
        let pos = CursorPosition { x: 0, y: 1 };
        editor
            .undo_redo_manager
            .record_undo(EditAction::DeleteLines {
                start: pos,
                deleted: removed.clone(),
            });
        editor.editor_content.drain(1..4);
        assert_eq!(editor.editor_content, vec!["a", "e"]);
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "b", "c", "d", "e"]);
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "e"]);
    }

    #[test]
    fn undo_redo_delete_all_lines() {
        let mut editor = create_editor_with_editor_content(vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
        ]);
        let removed = editor.editor_content.clone();
        let pos = CursorPosition { x: 0, y: 0 };
        editor
            .undo_redo_manager
            .record_undo(EditAction::DeleteLines {
                start: pos,
                deleted: removed.clone(),
            });

        editor.editor_content.clear();
        assert_eq!(editor.editor_content, Vec::<String>::new());
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["1", "2", "3"]);
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, Vec::<String>::new());
    }

    // ========== InsertRange ==========

    #[test]
    fn undo_redo_insert_range_single_line() {
        let mut editor = create_editor_with_editor_content(vec!["hello world".to_string()]);
        let pos = CursorPosition { x: 6, y: 0 }; // after "hello "
        let text = vec!["beautiful ".to_string()];

        editor
            .undo_redo_manager
            .record_undo(EditAction::InsertRange {
                start: pos,
                end: CursorPosition {
                    x: pos.x + 10,
                    y: 0,
                }, // "beautiful ".len()
                lines: text.clone(),
            });

        // apply insertion manually
        editor.editor_content[0].insert_str(6, &text[0]);
        assert_eq!(editor.editor_content, vec!["hello beautiful world"]);

        // undo -> original
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["hello world"]);

        // redo -> insertion again
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["hello beautiful world"]);
    }

    #[test]
    fn undo_redo_insert_range_multi_line() {
        let mut editor = create_editor_with_editor_content(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ]);
        let start = CursorPosition { x: 0, y: 1 };
        let end = CursorPosition { x: 1, y: 2 };
        let new = vec!["b".to_string(), "c".to_string()];

        editor
            .undo_redo_manager
            .record_undo(EditAction::InsertRange {
                start,
                end,
                lines: new.clone(),
            });

        // apply manually: result should be a, b, c, d

        assert_eq!(editor.editor_content, vec!["a", "b", "c", "d"]);

        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "d"]);

        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["a", "b", "c", "d"]);
    }

    // ========== DeleteRange ==========

    #[test]
    fn undo_redo_delete_range_single_line() {
        let mut editor = create_editor_with_editor_content(vec!["abcdef".to_string()]);
        let start = CursorPosition { x: 2, y: 0 };
        let end = CursorPosition { x: 5, y: 0 };

        let old = vec!["cde".to_string()];
        editor
            .undo_redo_manager
            .record_undo(EditAction::DeleteRange {
                start,
                end,
                deleted: old.clone(),
            });

        // delete manually
        editor.editor_content[0].replace_range(2..5, "");
        assert_eq!(editor.editor_content, vec!["abf"]);

        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["abcdef"]);

        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["abf"]);
    }

    #[test]
    fn undo_redo_delete_range_multi_line() {
        let mut editor = create_editor_with_editor_content(vec![
            "abc".to_string(),
            "def".to_string(),
            "ghi".to_string(),
        ]);
        let start = CursorPosition { x: 1, y: 0 }; // after 'a'
        let end = CursorPosition { x: 2, y: 2 }; // inside 'ghi'

        let old = vec![
            "bc".to_string(),  // from first line
            "def".to_string(), // middle line
            "gh".to_string(),  // from last line
        ];
        editor
            .undo_redo_manager
            .record_undo(EditAction::DeleteRange {
                start,
                end,
                deleted: old.clone(),
            });

        // apply manually: "abc", "def", "ghi" => delete selection => "ai"
        editor.editor_content = vec!["ai".to_string()];
        assert_eq!(editor.editor_content, vec!["ai"]);

        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["abc", "def", "ghi"]);

        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["ai"]);
    }

    // ========== SplitLine & JoinLine ==========

    #[test]
    fn undo_redo_split_line_middle() {
        let mut editor = create_editor_with_editor_content(vec!["hello world".to_string()]);
        let pos = CursorPosition { x: 5, y: 0 };

        // Record SplitLine
        let left = "hello".to_string();
        let right = " world".to_string();
        editor.undo_redo_manager.record_undo(EditAction::SplitLine {
            pos,
            left: left.clone(),
            right: right.clone(),
        });

        // Apply manually
        editor.editor_content[0] = left.clone();
        editor.editor_content.insert(1, right.clone());

        assert_eq!(editor.editor_content, vec!["hello", " world"]);

        // Undo should merge back
        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["hello world"]);

        // Redo should split again
        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["hello", " world"]);
    }

    #[test]
    fn undo_redo_split_line_start() {
        let mut editor = create_editor_with_editor_content(vec!["abc".to_string()]);
        let pos = CursorPosition { x: 0, y: 0 };

        let left = "".to_string();
        let right = "abc".to_string();
        editor.undo_redo_manager.record_undo(EditAction::SplitLine {
            pos,
            left: left.clone(),
            right: right.clone(),
        });

        editor.editor_content[0] = left.clone();
        editor.editor_content.insert(1, right.clone());

        assert_eq!(editor.editor_content, vec!["", "abc"]);

        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["abc"]);

        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["", "abc"]);
    }

    #[test]
    fn undo_redo_split_line_end() {
        let mut editor = create_editor_with_editor_content(vec!["abc".to_string()]);
        let pos = CursorPosition { x: 3, y: 0 };

        let left = "abc".to_string();
        let right = "".to_string();
        editor.undo_redo_manager.record_undo(EditAction::SplitLine {
            pos,
            left: left.clone(),
            right: right.clone(),
        });

        editor.editor_content[0] = left.clone();
        editor.editor_content.insert(1, right.clone());

        assert_eq!(editor.editor_content, vec!["abc", ""]);

        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["abc"]);

        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["abc", ""]);
    }

    #[test]
    fn undo_redo_join_line() {
        let mut editor =
            create_editor_with_editor_content(vec!["foo".to_string(), "bar".to_string()]);
        let pos = CursorPosition { x: 3, y: 0 };

        let merged = "foobar".to_string();
        editor.undo_redo_manager.record_undo(EditAction::JoinLine {
            pos,
            merged: merged.clone(),
        });

        editor.editor_content[0] = merged.clone();
        editor.editor_content.remove(1);

        assert_eq!(editor.editor_content, vec!["foobar"]);

        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["foo", "bar"]);

        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["foobar"]);
    }

    // ========== Additional SplitLine & JoinLine Tests ==========

    #[test]
    fn undo_redo_split_empty_line() {
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);
        let pos = CursorPosition { x: 0, y: 0 };

        let left = "".to_string();
        let right = "".to_string();
        editor.undo_redo_manager.record_undo(EditAction::SplitLine {
            pos,
            left: left.clone(),
            right: right.clone(),
        });

        editor.editor_content[0] = left.clone();
        editor.editor_content.insert(1, right.clone());

        assert_eq!(editor.editor_content, vec!["", ""]);

        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec![""]);

        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["", ""]);
    }

    #[test]
    fn undo_redo_join_empty_lines() {
        let mut editor = create_editor_with_editor_content(vec!["".to_string(), "".to_string()]);
        let pos = CursorPosition { x: 0, y: 0 };

        let merged = "".to_string();
        editor.undo_redo_manager.record_undo(EditAction::JoinLine {
            pos,
            merged: merged.clone(),
        });

        editor.editor_content[0] = merged.clone();
        editor.editor_content.remove(1);

        assert_eq!(editor.editor_content, vec![""]);

        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["", ""]);

        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec![""]);
    }

    #[test]
    fn undo_redo_split_line_multiple_lines() {
        let mut editor =
            create_editor_with_editor_content(vec!["first".to_string(), "second".to_string()]);
        let pos = CursorPosition { x: 3, y: 1 };

        let left = "sec".to_string();
        let right = "ond".to_string();
        editor.undo_redo_manager.record_undo(EditAction::SplitLine {
            pos,
            left: left.clone(),
            right: right.clone(),
        });

        editor.editor_content[1] = left.clone();
        editor.editor_content.insert(2, right.clone());

        assert_eq!(editor.editor_content, vec!["first", "sec", "ond"]);

        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["first", "second"]);

        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["first", "sec", "ond"]);
    }

    #[test]
    fn undo_redo_join_lines_with_whitespace() {
        let mut editor =
            create_editor_with_editor_content(vec!["foo ".to_string(), " bar".to_string()]);
        let pos = CursorPosition { x: 4, y: 0 };

        let merged = "foo  bar".to_string();
        editor.undo_redo_manager.record_undo(EditAction::JoinLine {
            pos,
            merged: merged.clone(),
        });

        editor.editor_content[0] = merged.clone();
        editor.editor_content.remove(1);

        assert_eq!(editor.editor_content, vec!["foo  bar"]);

        editor.undo().unwrap();
        assert_eq!(editor.editor_content, vec!["foo ", " bar"]);

        editor.redo().unwrap();
        assert_eq!(editor.editor_content, vec!["foo  bar"]);
    }

    #[test]
    fn undo_redo_split_and_join_sequential() {
        let mut editor = create_editor_with_editor_content(vec!["abc".to_string()]);

        // Split
        let split_pos = CursorPosition { x: 1, y: 0 };
        let left = "a".to_string();
        let right = "bc".to_string();
        editor.undo_redo_manager.record_undo(EditAction::SplitLine {
            pos: split_pos,
            left: left.clone(),
            right: right.clone(),
        });
        editor.editor_content[0] = left.clone();
        editor.editor_content.insert(1, right.clone());

        // Join immediately after
        let join_pos = CursorPosition { x: 1, y: 0 };
        let merged = "abc".to_string();
        editor.undo_redo_manager.record_undo(EditAction::JoinLine {
            pos: join_pos,
            merged: merged.clone(),
        });
        editor.editor_content[0] = merged.clone();
        editor.editor_content.remove(1);

        assert_eq!(editor.editor_content, vec!["abc"]);

        editor.undo().unwrap(); // Undo join
        assert_eq!(editor.editor_content, vec!["a", "bc"]);

        editor.undo().unwrap(); // Undo split
        assert_eq!(editor.editor_content, vec!["abc"]);

        editor.redo().unwrap(); // Redo split
        assert_eq!(editor.editor_content, vec!["a", "bc"]);

        editor.redo().unwrap(); // Redo join
        assert_eq!(editor.editor_content, vec!["abc"]);
    }

    // ========== Edge and Stack Cases ==========
    #[test]
    fn undo_redo_stack_behavior() {
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);
        // Undo & redo stack empty
        assert!(editor.undo().is_err());
        assert!(editor.redo().is_err());
        // Normal sequence
        editor.write_char('t');
        editor.undo().unwrap();
        assert!(editor.redo().is_ok());
        // After new action, redo stack cleared
        editor.write_char('z');
        assert!(editor.redo().is_err());
    }

    #[test]
    fn alternating_undo_redo_variety() {
        let mut editor = create_editor_with_editor_content(vec!["".to_string()]);
        editor.write_char('a');
        editor.write_char('b');
        editor.write_char('c');
        assert_eq!(editor.editor_content[0], "abc");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "ab");
        editor.write_char('Z');
        assert_eq!(editor.editor_content[0], "abZ");
        assert!(editor.redo().is_err()); // Redo stack cleared
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "ab");
        editor.undo().unwrap();
        assert_eq!(editor.editor_content[0], "a");
        editor.redo().unwrap();
        assert_eq!(editor.editor_content[0], "ab");
    }
}
