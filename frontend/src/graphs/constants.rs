use plotters::style::RGBColor;

// pink, purple, green, blue, sand, red, bulma blue
pub static COLORS: [RGBColor; 7] = [RGBColor(255,181,232), RGBColor(178,141,255), RGBColor(178,248,219),
    RGBColor(175,203,255), RGBColor(255,245,186), RGBColor(255,171,171), RGBColor(112,157,231)];

pub static MAX_COLS: u64 = 500;
pub static MAX_HISTORY: u64 = 30_000;