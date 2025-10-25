use calliglyph::core::editor::Editor;
use calliglyph::input::input_action::{Direction, InputAction};
use calliglyph::core::cursor::CursorPosition;

/// Helper to create an editor with some starting text.
fn create_editor_with_content(lines: Vec<&str>) -> Editor {
    let mut editor = Editor::new();
    editor.editor_content = lines.into_iter().map(String::from).collect();
    editor.editor_height = 10;
    editor
}

#[test]
fn test_write_and_backspace_single_char() {
    let mut editor = create_editor_with_content(vec!["Hello"]);
    editor.cursor.x = 6;
    // Type a character
    editor.handle_input_action(InputAction::WriteChar('!')).unwrap();
    assert_eq!(editor.editor_content[0], "Hello!");

    // Remove it with backspace
    editor.handle_input_action(InputAction::BACKSPACE).unwrap();
    assert_eq!(editor.editor_content[0], "Hello");
}

#[test]
fn test_enter_splits_line_and_backspace_joins() {
    let mut editor = create_editor_with_content(vec!["Hello world"]);
    editor.cursor.x = 6;
    editor.cursor.y = 0;
    
    // Press ENTER (split line)
    editor.handle_input_action(InputAction::ENTER).unwrap();
    assert_eq!(editor.editor_content.len(), 2, "Line count should be 2 after split");
    assert_eq!(
        editor.editor_content[0], "Hello ",
        "First line should contain the left half of the split"
    );
    assert_eq!(
        editor.editor_content[1], "world",
        "Second line should contain the right half of the split"
    );

    // Join back
    editor.cursor.x = 0;
    editor.cursor.y = 1;

    editor.handle_input_action(InputAction::BACKSPACE).unwrap();
    assert_eq!(editor.editor_content.len(), 1);
    assert_eq!(editor.editor_content[0], "Hello world");
}

#[test]
fn test_tab_inserts_spaces() {
    let mut editor = create_editor_with_content(vec![""]);
    editor.handle_input_action(InputAction::TAB).unwrap();
    assert!(editor.editor_content[0].starts_with("\t")); // assume 4-space tab
}

#[test]
fn test_move_cursor_left_and_right() {
    let mut editor = create_editor_with_content(vec!["abcdef"]);
    editor.cursor.x = 6;

    editor
        .handle_input_action(InputAction::MoveCursor(Direction::Left))
        .unwrap();
    let left_x = editor.cursor.x;
    assert!(left_x < 6);

    editor
        .handle_input_action(InputAction::MoveCursor(Direction::Right))
        .unwrap();
    assert!(editor.cursor.x > left_x);
}

#[test]
fn test_move_cursor_up_and_down() {
    let mut editor = create_editor_with_content(vec!["line1", "line2", "line3"]);
    editor.cursor.y = 1;

    editor
        .handle_input_action(InputAction::MoveCursor(Direction::Up))
        .unwrap();
    assert_eq!(editor.cursor.y, 0);

    editor
        .handle_input_action(InputAction::MoveCursor(Direction::Down))
        .unwrap();
    assert_eq!(editor.cursor.y, 1);
}

#[test]
fn test_write_undo_and_redo() {
    let mut editor = create_editor_with_content(vec!["foo"]);
    editor.cursor.x = 3;
    // Type '!'
    editor.handle_input_action(InputAction::WriteChar('!')).unwrap();
    assert_eq!(editor.editor_content[0], "foo!");

    // Undo it
    editor.handle_input_action(InputAction::UNDO).unwrap();
    assert_eq!(editor.editor_content[0], "foo");

    // Redo it
    editor.handle_input_action(InputAction::REDO).unwrap();
    assert_eq!(editor.editor_content[0], "foo!");
}

#[test]
fn test_multiple_writes_and_undo_redo_chain() {
    let mut editor = create_editor_with_content(vec!["abc"]);
    editor.cursor.x = 3;
    editor.cursor.y = 0;
    for c in ['d', 'e', 'f'] {
        editor.handle_input_action(InputAction::WriteChar(c)).unwrap();
    }
    assert_eq!(editor.editor_content[0], "abcdef");

    // Undo all
    for _ in 0..3 {
        editor.handle_input_action(InputAction::UNDO).unwrap();
    }
    assert_eq!(editor.editor_content[0], "abc");

    // Redo all
    for _ in 0..3 {
        editor.handle_input_action(InputAction::REDO).unwrap();
    }
    assert_eq!(editor.editor_content[0], "abcdef");
}

#[test]
fn test_multi_line_editing_with_undo() {
    let mut editor = create_editor_with_content(vec!["first", "second"]);
    editor.cursor.y = 0;
    editor.handle_input_action(InputAction::WriteChar('A')).unwrap();

    editor.cursor.x = 0;
    editor.cursor.y = 1;
    editor.handle_input_action(InputAction::WriteChar('B')).unwrap();

    assert_eq!(editor.editor_content[0], "Afirst");
    assert_eq!(editor.editor_content[1], "Bsecond");

    // Undo last change (B)
    editor.handle_input_action(InputAction::UNDO).unwrap();
    assert_eq!(editor.editor_content[1], "second");

    // Undo first change (A)
    editor.handle_input_action(InputAction::UNDO).unwrap();
    assert_eq!(editor.editor_content[0], "first");
}

#[test]
fn test_text_selection_delete_range() {
    let mut editor = create_editor_with_content(vec!["delete this"]);
    editor.text_selection_start = Some(CursorPosition { x: 0, y: 0 });
    editor.text_selection_end = Some(CursorPosition { x: 11, y: 0 });

    editor.handle_input_action(InputAction::BACKSPACE).unwrap();
    assert_eq!(editor.editor_content[0], "");
}

#[test]
fn test_scroll_offset_and_cursor_bounds() {
    let mut editor = create_editor_with_content(vec![
        "line1", "line2", "line3", "line4", "line5", "line6",
    ]);
    editor.editor_height = 3;
    editor.scroll_offset = 2;
    editor.cursor.y = 5;

    editor
        .handle_input_action(InputAction::MoveCursor(Direction::Up))
        .unwrap();
    assert!(editor.scroll_offset <= 2);
}
