use super::super::errors::{RedoError, UndoError};
use super::editor::EditAction;

#[derive(Debug)]
pub struct UndoRedoManager {
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
}

impl UndoRedoManager {
    pub fn new() -> UndoRedoManager {
        Self {
            undo_stack: vec![],
            redo_stack: vec![],
        }
    }

    ///records and action done to the undo stack, and clears redo stack.
    pub fn record_undo(&mut self, action: EditAction) {
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    // UNDO AND REDO FUNCTIONALITY
    /// undo's last action of user
    pub fn undo(&mut self) -> Result<EditAction, UndoError> {
        if let Some(last_action) = self.undo_stack.pop() {
            let action_reversed = self.reverse_action(&last_action);
            //self.apply_action(editor,&action_reversed);
            self.redo_stack.push(last_action);
            Ok(action_reversed)
        } else {
            Err(UndoError::NoActionToUndo)
        }
    }

    /// redo's last action of user
    pub fn redo(&mut self) -> Result<EditAction, RedoError> {
        if let Some(last_action) = self.redo_stack.pop() {
            //self.apply_action(editor, &last_action);
            self.undo_stack.push(last_action.clone());
            Ok(last_action)
        } else {
            Err(RedoError::NoActionToRedo)
        }
    }

    /// returns the reverse action of the given action
    fn reverse_action(&mut self, action: &EditAction) -> EditAction {
        match action {
            EditAction::Insert { pos, c } => EditAction::Delete {
                pos: *pos,
                deleted_char: *c,
            },
            EditAction::Delete { pos, deleted_char } => EditAction::Insert {
                pos: *pos,
                c: *deleted_char,
            },
            EditAction::Replace {
                start,
                end,
                old,
                new,
            } => EditAction::Replace {
                start: *start,
                end: *end,
                old: new.clone(),
                new: old.clone(),
            },
            EditAction::InsertLines { start, lines } => EditAction::DeleteLines {
                start: *start,
                deleted: lines.clone(),
            },
            EditAction::DeleteLines { start, deleted } => EditAction::InsertLines {
                start: *start,
                lines: deleted.clone(),
            },
        }
    }
}
