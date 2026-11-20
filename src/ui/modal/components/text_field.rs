use crate::input::actions::ModalAction;

#[derive(Default, Clone, Debug)]
pub struct TextField {
    value: String,
}

impl TextField {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_value(v: &str) -> Self {
        Self {
            value: v.to_string(),
        }
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
    pub fn len(&self) -> usize {
        self.value.chars().count()
    }
    pub fn clear(&mut self) {
        self.value.clear();
    }

    /// Applies Text/Backspace. Returns true if the action was an editing action.
    pub fn handle(&mut self, action: &ModalAction) -> bool {
        match action {
            ModalAction::Text(c) if !c.is_control() => {
                self.value.push(*c);
                true
            }
            ModalAction::Backspace => {
                self.value.pop();
                true
            }
            _ => false,
        }
    }

    /// Bullets, with the last `reveal_last` chars visible once the value is
    /// longer than that (so a paste can be sanity-checked).
    pub fn masked(&self, reveal_last: usize) -> String {
        let n = self.len();
        if n <= reveal_last {
            return self.value.clone();
        }
        let tail: String = self.value.chars().skip(n - reveal_last).collect();
        format!("{}{}", "•".repeat(n - reveal_last), tail)
    }
}

#[cfg(test)]
mod unit_text_field_tests {
    use super::*;

    #[test]
    fn text_and_backspace_edit_the_value() {
        let mut f = TextField::new();
        for c in "aB1".chars() {
            assert!(f.handle(&ModalAction::Text(c)));
        }
        assert_eq!(f.value(), "aB1");
        f.handle(&ModalAction::Backspace);
        assert_eq!(f.value(), "aB");
    }

    #[test]
    fn control_chars_are_ignored() {
        let mut f = TextField::new();
        f.handle(&ModalAction::Text('\n'));
        assert!(f.is_empty());
    }

    #[test]
    fn masked_reveals_only_the_tail() {
        let f = TextField::with_value("ghp_abcdef1234");
        let m = f.masked(4);
        assert!(m.ends_with("1234"));
        assert!(m.starts_with('•'));
        assert_eq!(m.chars().count(), 14);
    }

    #[test]
    fn short_values_also_mask_everything_but_reveal_last_even_when_it_is_bigger_than_char_count() {
        assert_eq!(TextField::with_value("abc").masked(4), "abc");
    }
}
