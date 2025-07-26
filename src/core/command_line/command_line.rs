use crate::input::input_action::InputAction;
use super::super::cursor::Cursor;

#[derive(Debug, Default)]
pub struct CommandLine {
    pub input: String,
    pub cursor: Cursor,
}

impl CommandLine {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor: Cursor::new(),
        }
    }

    ///function to handle input action on command line,
    /// responsible for dispatching action to corret internal method.
    pub fn handle_input_action(&mut self, action: InputAction) {
        match action {
            InputAction::MoveCursor(direction) => {}
            InputAction::MoveSelectionCursor(direction) => {}
            InputAction::TAB => {}
            InputAction::ENTER => {}
            InputAction::BACKSPACE => {}
            InputAction::DELETE => {}
            InputAction::SAVE => {}
            InputAction::COPY => {}
            InputAction::CUT => {}
            InputAction::PASTE => {}
            InputAction::UNDO => {}
            InputAction::REDO => {}
            InputAction::ToggleActiveArea => {}
            InputAction::WriteChar(c) => {}
            InputAction::QUIT => {}
            InputAction::NoOp => {}
        }
    }
    
    //writing
    ///writes char to line, with x position
    pub fn write_char(&mut self, c: char) {
        let line = &mut self.input;
        if line.len() < self.cursor.x as usize {
            self.cursor.x = line.len() as i16;
        }
        line.insert(self.cursor.x as usize, c);
        self.move_cursor(1);
    }
    ///backspaces on x position
    pub fn backspace(&mut self) {
        let line = &mut self.input;
        if self.cursor.x > 0 && self.cursor.x <= line.len() as i16 {
            line.remove(self.cursor.x as usize - 1);
            self.move_cursor(-1);
        }
    }

    //cursor
    ///moves cursor by x amounts in commandline
    pub fn move_cursor(&mut self, x: i16) {
        let max_x_pos: i16 = self.input.len() as i16;
        self.cursor.x = (self.cursor.x + x).clamp(0, max_x_pos);
    }
}

//████████╗███████╗███████╗████████╗███████╗
//╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//   ██║   █████╗  ███████╗   ██║   ███████╗
//   ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//   ██║   ███████╗███████║   ██║   ███████║
//   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
#[cfg(test)]
mod unit_commandline_command_line_tests {
    use super::super::super::super::core::command_line::CommandLine;

    fn create_command_line_with_command_input(s: String) -> CommandLine {
        let mut command_line = CommandLine::new();
        command_line.input = s;
        command_line
    }

    //writing chars to command line
    #[test]
    fn test_write_char_to_command_line() {
        let mut command_line = create_command_line_with_command_input("".to_string());
        command_line.write_char('A');

        assert_eq!(command_line.input, "A");
        assert_eq!(command_line.cursor.x, 1);
    }

    #[test]
    fn test_write_char_to_command_line_mid_input() {
        let mut command_line = create_command_line_with_command_input("Test".to_string());
        command_line.cursor.x = 2;
        command_line.write_char('X');

        assert_eq!(command_line.input, "TeXst");
        assert_eq!(command_line.cursor.x, 3);
    }

    //BACKSPACE in commandline

    #[test]
    fn test_backspace_at_start() {
        let mut command_line = create_command_line_with_command_input("".to_string());
        command_line.cursor.x = 0;
        command_line.backspace();

        assert_eq!(command_line.input, "");
        assert_eq!(command_line.cursor.x, 0);
    }

    #[test]
    fn test_backspace_in_middle() {
        let mut command_line = create_command_line_with_command_input("Test".to_string());
        command_line.cursor.x = 3;
        command_line.backspace();

        assert_eq!(command_line.input, "Tet");
        assert_eq!(command_line.cursor.x, 2);
    }
}
