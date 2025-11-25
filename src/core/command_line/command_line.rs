use super::super::cursor::Cursor;
use crate::input::input_action::InputAction;

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
            InputAction::MoveCursor(direction) => {
                let (x, _y) = direction.to_vector();
                self.move_cursor(x);
            }
            InputAction::BACKSPACE => self.backspace(),
            InputAction::DELETE => self.delete(),
            InputAction::WriteChar(c) => {
                self.write_char(c);
            }
            InputAction::NoOp => {}
            _ => {}
        }
    }

    ///to split command line text into a command and arguments
    pub fn split_command_bind_and_args(&mut self) -> Result<(String, Vec<String>), String> {
        let mut command_bind: Option<String> = None;
        let mut command_args = vec![];
        let mut parts = self.input.split_whitespace();

        if let Some(first) = parts.next() {
            if let Some(':') = first.chars().next() {
                command_bind = Some(first.chars().skip(1).collect());
            }
        }

        if let Some(ref cmd) = command_bind {
            command_args.extend(parts.map(String::from));
            return Ok((cmd.clone(), command_args));
        }

        Err("No valid command found".to_string())
    }

    //writing
    ///writes char to line, with x position
    pub fn write_char(&mut self, c: char) {
        let line = &mut self.input;

        // cursor calc based on character count for multibyte chars
        let char_len = line.chars().count();
        if (self.cursor.x as usize) > char_len {
            self.cursor.x = char_len as i16;
        }

        let char_idx = self.cursor.x as usize;

        let byte_idx = Self::get_byte_idx(char_idx, line);

        // Insert the char at the calculated byte index.
        line.insert(byte_idx, c);
        self.move_cursor(1);
    }
    ///backspaces on x position
    pub fn backspace(&mut self) {
        let line = &mut self.input;
        if self.cursor.x > 0 && self.cursor.x <= line.len() as i16 {
            let char_idx = self.cursor.x as usize;

            let byte_idx = Self::get_byte_idx(char_idx - 1, line);

            line.remove(byte_idx);
            self.move_cursor(-1);
        }
    }

    ///deletes on x position
    pub fn delete(&mut self) {
        let line = &mut self.input;
        if line.len() > 0 && self.cursor.x < line.len() as i16 {
            line.remove(self.cursor.x as usize);
        }
    }

    //cursor
    ///moves cursor by x amounts in commandline
    pub fn move_cursor(&mut self, x: i16) {
        let max_x_pos: i16 = self.input.chars().count() as i16;
        self.cursor.x = (self.cursor.x + x).clamp(0, max_x_pos);
    }

    /// Find the byte index corresponding to the char index.
    fn get_byte_idx(char_idx: usize, line: &str) -> usize {
        // Find the byte index corresponding to the char index.
        line.char_indices()
            .nth(char_idx)
            .map(|(idx, _)| idx)
            .unwrap_or(line.len()) // If char_idx is at the end, use the total byte length
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

    //DELETE in commandline

    #[test]
    fn test_delete_at_start() {
        let mut command_line = create_command_line_with_command_input("".to_string());
        command_line.cursor.x = 0;
        command_line.delete();

        assert_eq!(command_line.input, "");
        assert_eq!(command_line.cursor.x, 0);
    }

    #[test]
    fn test_delete_in_middle() {
        let mut command_line = create_command_line_with_command_input("Test".to_string());
        command_line.cursor.x = 2;
        command_line.delete();

        assert_eq!(command_line.input, "Tet");
        assert_eq!(command_line.cursor.x, 2);
    }
}
