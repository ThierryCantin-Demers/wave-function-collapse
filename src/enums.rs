use bmp::Pixel;
use rand::Rng;

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum Tile {
    Red,
    Green,
    Blue,
}

impl Tile {
    pub fn minify(&self) -> char {
        match self {
            Tile::Red => 'R',
            Tile::Green => 'G',
            Tile::Blue => 'B',
        }
    }
}

impl From<Tile> for Pixel {
    fn from(tile: Tile) -> Self {
        match tile {
            Tile::Red => Pixel { r: 255, g: 0, b: 0 },
            Tile::Green => Pixel { r: 0, g: 255, b: 0 },
            Tile::Blue => Pixel { r: 0, g: 0, b: 255 },
        }
    }
}

impl From<Pixel> for Tile {
    fn from(pixel: Pixel) -> Self {
        if pixel.r == 255 && pixel.g == 0 && pixel.b == 0 {
            Tile::Red
        } else if pixel.r == 0 && pixel.g == 255 && pixel.b == 0 {
            Tile::Green
        } else if pixel.r == 0 && pixel.g == 0 && pixel.b == 255 {
            Tile::Blue
        } else {
            panic!("Invalid pixel color: {:?}", pixel)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn offset(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

pub fn generate_color(x: u32, y: u32) -> Tile {
    let res = ((x + y) * rand::thread_rng().gen::<u32>()) % 3;
    match res {
        0 => Tile::Red,
        1 => Tile::Green,
        2 => Tile::Blue,
        _ => Tile::Red,
    }
}
