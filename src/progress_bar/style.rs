#[derive(Debug, Clone, Copy)]
pub enum Style {
    /// Classic style: [████████░░░░░░░░]
    Classic,
    /// ASCII style: [########--------]
    Ascii,
    /// Dots style: [●●●●●●●●○○○○○○○○]
    Dots,
    /// Arrows style: [>>>>>>>>--------]
    Arrows,
    /// Blocks style: [▰▰▰▰▰▰▱▱▱▱▱▱]
    Blocks,
}

impl Style {
    pub fn filled_char(&self) -> char {
        match self {
            Style::Classic => '█',
            Style::Ascii => '#',
            Style::Dots => '●',
            Style::Arrows => '>',
            Style::Blocks => '▰',
        }
    }

    pub fn empty_char(&self) -> char {
        match self {
            Style::Classic => '░',
            Style::Ascii => '-',
            Style::Dots => '○',
            Style::Arrows => '-',
            Style::Blocks => '▱',
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::Classic
    }
}
