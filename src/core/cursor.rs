/// handles cursor
#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    pub x: i16,
    pub y: i16,
}

impl Cursor {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct CursorPosition {
    pub(crate) x: usize,
    pub(crate) y: usize,
}
