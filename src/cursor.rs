

/// handles cursor
#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub x: i16,
    pub y: i16,
}

impl Cursor {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}

#[derive(Debug,Copy,Clone)]
pub struct CursorPosition {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl Default for CursorPosition {
    fn default() -> CursorPosition {
        CursorPosition { x: 0, y: 0 }
    }
}