use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::{collections::HashSet, hash::Hash, vec};

use bmp::{Image, Pixel};
use rand::prelude::SliceRandom;
use rand::Rng;

fn delete_files_in_dir(path: &str) -> io::Result<()> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                // Check if it's a file
                if entry.metadata()?.is_file() {
                    // Delete the file
                    fs::remove_file(entry.path())?;
                }
            }
        }
    }
    Ok(())
}

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
pub type Rule = (Tile, Tile, Direction);

fn generate_color(x: u32, y: u32) -> Tile {
    let res = ((x + y) * rand::thread_rng().gen::<u32>()) % 3;
    match res {
        0 => Tile::Red,
        1 => Tile::Green,
        2 => Tile::Blue,
        _ => Tile::Red,
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub possible_vals: Vec<Vec<HashSet<Tile>>>,
    pub width: usize,
    pub height: usize,
    pub curr_file_index: u32,
    pub rules: HashSet<Rule>,
}

impl State {
    pub fn new(w: usize, h: usize, all_tiles_types: &HashSet<Tile>, rules: HashSet<Rule>) -> Self {
        State {
            possible_vals: vec![vec![all_tiles_types.clone(); w]; h],
            curr_file_index: 0,
            width: w,
            height: h,
            rules,
        }
    }

    pub fn save_into_file(&mut self, end: &str) {
        let mut file_path = PathBuf::new();
        file_path.push("imgs");
        file_path.push("output");
        file_path.push(&format!("state_{}_{}", self.curr_file_index, end));
        file_path.set_extension("txt");
        println!("Saving into file: {:?}", file_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create directories");
        }
        let file = std::fs::File::create(file_path).unwrap();
        self.curr_file_index += 1;
        let mut w = std::io::BufWriter::new(file);
        let max_col = self.possible_vals[0].len();
        for col in 0..max_col {
            for row in &self.possible_vals {
                if let Some(tile) = row.get(col) {
                    for t in tile {
                        let _ = write!(w, "{}", t.minify());
                    }
                    let _ = write!(w, "\t");
                }
            }
            let _ = write!(w, "\n");
        }
    }

    pub fn save_rules_into_file(&self) {
        let mut file_path = PathBuf::new();
        file_path.push("imgs");
        file_path.push("output");
        file_path.push("rules");
        file_path.set_extension("txt");
        println!("Saving rules into file: {:?}", file_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create directories");
        }
        let file = std::fs::File::create(file_path).unwrap();
        let mut w = std::io::BufWriter::new(file);
        for rule in &self.rules {
            let _ = write!(w, "{:?} can be at {:?} of {:?}\n", rule.0, rule.2, rule.1);
        }
    }

    pub fn get(&self, x: usize, y: usize) -> HashSet<Tile> {
        self.possible_vals[x][y].clone()
    }

    pub fn get_total_entropy(&self) -> usize {
        self.possible_vals.iter().flatten().flatten().count()
    }
}

pub fn generate_bitmap(w: u32, h: u32) -> Image {
    let mut img = Image::new(w, h);
    for (x, y) in img.coordinates() {
        img.set_pixel(x, y, generate_color(x, y).into());
    }
    img
}

pub fn save_bitmap(img: Image, file_name: &str) {
    let _ = img.save(file_name);
}

pub fn read_bitmap(file_name: &str) -> Image {
    let img = bmp::open(file_name).unwrap();
    println!("read_bitmap: {:?}", img);
    img
}

pub fn get_image_adjacent_pixels(
    img: &Image,
    x: u32,
    y: u32,
) -> (Option<Pixel>, Option<Pixel>, Option<Pixel>, Option<Pixel>) {
    let w = img.get_width();
    let h = img.get_height();
    let up = if y > 0 {
        Some(img.get_pixel(x, y - 1))
    } else {
        None
    };
    let down = if y < h - 1 {
        Some(img.get_pixel(x, y + 1))
    } else {
        None
    };
    let left = if x > 0 {
        Some(img.get_pixel(x - 1, y))
    } else {
        None
    };
    let right = if x < w - 1 {
        Some(img.get_pixel(x + 1, y))
    } else {
        None
    };
    (up, down, left, right)
}

pub fn get_possibilities_adjacent_pixels(
    possible_vals: &Vec<Vec<HashSet<Tile>>>,
    x: usize,
    y: usize,
) -> (
    Option<HashSet<Tile>>,
    Option<HashSet<Tile>>,
    Option<HashSet<Tile>>,
    Option<HashSet<Tile>>,
    Option<HashSet<Tile>>,
) {
    let w = possible_vals.len();
    let h = possible_vals[0].len();

    let curr = possible_vals[x as usize][y as usize].clone();
    let up = if y > 0 {
        Some(possible_vals[x as usize][y as usize - 1].clone())
    } else {
        None
    };
    let down = if y < h - 1 {
        Some(possible_vals[x as usize][y as usize + 1].clone())
    } else {
        None
    };
    let left = if x > 0 {
        Some(possible_vals[x as usize - 1][y as usize].clone())
    } else {
        None
    };
    let right = if x < w - 1 {
        Some(possible_vals[x as usize + 1][y as usize].clone())
    } else {
        None
    };
    (Some(curr), up, down, left, right)
}

pub fn extract_rules(img: &Image) -> HashSet<Rule> {
    let mut rules = HashSet::new();
    for (x, y) in img.coordinates() {
        let curr_tile = img.get_pixel(x, y);
        let (up, down, left, right) = get_image_adjacent_pixels(img, x, y);

        if up.is_some() {
            rules.insert((curr_tile.into(), up.unwrap().into(), Direction::Up));
        }
        if down.is_some() {
            rules.insert((curr_tile.into(), down.unwrap().into(), Direction::Down));
        }
        if left.is_some() {
            rules.insert((curr_tile.into(), left.unwrap().into(), Direction::Left));
        }
        if right.is_some() {
            rules.insert((curr_tile.into(), right.unwrap().into(), Direction::Right));
        }
    }

    rules
}

pub fn apply_rules(curr_state: &mut State, rules: &HashSet<Rule>) {
    let mut new_state = curr_state.possible_vals.clone();
    let w = curr_state.width;
    let h = curr_state.height;

    for x in 0..w {
        for y in 0..h {
            let possibilities = curr_state.get(x, y);
            if possibilities.len() == 1 {
                continue;
            }

            for rule in rules {
                let (curr_tile, adj_tile, direction) = rule;
                if !possibilities.contains(curr_tile) {
                    continue;
                }
                let (dx, dy) = direction.offset();
                let adj_x = x as i32 + dx;
                let adj_y = y as i32 + dy;
                if adj_x < 0 || adj_x >= w as i32 || adj_y < 0 || adj_y >= h as i32 {
                    continue;
                }
                let adj_x = adj_x as usize;
                let adj_y = adj_y as usize;
                let adj_possibilities = curr_state.get(adj_x, adj_y);
                if !adj_possibilities.contains(adj_tile) {
                    new_state[x][y].remove(curr_tile);
                    if possibilities.len() == 1 {
                        break;
                    }
                }
            }
        }
    }

    curr_state.possible_vals = new_state.clone();
}

pub fn contains_invalid_tiles(state: &Vec<Vec<HashSet<Tile>>>) -> bool {
    for row in state {
        for tile in row {
            if tile.is_empty() {
                return true;
            }
        }
    }
    false
}

pub fn get_image_from_possible_vals(state: &State) -> Option<Image> {
    let w = state.width as u32;
    let h = state.height as u32;
    let mut img = Image::new(w, h);
    for x in 0..w {
        for y in 0..h {
            if state.get(x as usize, y as usize).len() != 1 {
                return None;
            }
            img.set_pixel(
                x,
                y,
                state
                    .get(x as usize, y as usize)
                    .iter()
                    .next()
                    .unwrap()
                    .clone()
                    .into(),
            );
        }
    }

    Some(img)
}

fn get_lowest_entropy_tile(possible_vals: &Vec<Vec<HashSet<Tile>>>) -> Option<(usize, usize)> {
    let mut min_entropy = usize::MAX;
    let mut min_entropy_tiles = Vec::new();

    for (i, row) in possible_vals.iter().enumerate() {
        for (j, tile) in row.iter().enumerate() {
            let entropy = tile.len();
            if entropy == 1 {
                continue;
            }
            if entropy < min_entropy {
                min_entropy = entropy;
                min_entropy_tiles.clear();
                min_entropy_tiles.push((i, j));
            } else if entropy == min_entropy {
                min_entropy_tiles.push((i, j));
            }
        }
    }

    min_entropy_tiles.choose(&mut rand::thread_rng()).cloned()
}

pub trait HashSetExt<T> {
    fn with(self, value: T) -> HashSet<T>;
}

impl<T: std::hash::Hash + Eq + Clone> HashSetExt<T> for HashSet<T> {
    fn with(mut self: Self, value: T) -> HashSet<T> {
        self.insert(value);
        self
    }
}

pub fn print_tile_possibilities_and_adjacents(
    possible_vals: &Vec<Vec<HashSet<Tile>>>,
    x: usize,
    y: usize,
) {
    let (curr, up, down, left, right) = get_possibilities_adjacent_pixels(possible_vals, x, y);
    println!("Current: {:?}", curr);
    println!("Up: {:?}", up);
    println!("Down: {:?}", down);
    println!("Left: {:?}", left);
    println!("Right: {:?}", right);
}

pub fn generate_image(w: u32, h: u32, rules: &HashSet<Rule>) -> Option<Image> {
    delete_files_in_dir("imgs/output").expect("Failed to delete files");
    let mut all_tiles_types = HashSet::new();
    for rule in rules {
        all_tiles_types.insert(rule.0.clone());
        all_tiles_types.insert(rule.1.clone());
    }

    let mut state: State = State::new(w as usize, h as usize, &all_tiles_types, rules.clone());
    state.save_rules_into_file();
    state.save_into_file("initial");

    println!(
        "Initial entropy : {:?}",
        state.possible_vals.iter().flatten().flatten().count()
    );

    let next_tile_coord = get_lowest_entropy_tile(&state.possible_vals);
    if next_tile_coord.is_none() {
        return None;
    }

    while state
        .possible_vals
        .iter()
        .any(|row| row.iter().any(|tile| tile.len() > 1))
    {
        let mut old_state = state.clone();
        apply_rules(&mut state, rules);
        while contains_invalid_tiles(&state.possible_vals) {
            state = old_state.clone();
            apply_rules(&mut state, rules);
        }

        while old_state.get_total_entropy() != state.get_total_entropy() {
            old_state = state.clone();
            apply_rules(&mut state, rules);
            state.save_into_file("after_rule");
        }

        let next_tile_coord = get_lowest_entropy_tile(&state.possible_vals);
        if next_tile_coord.is_none() {
            break;
        }
        let next_tile_coord = next_tile_coord.unwrap();
        println!("Next tile coord: {:?}", next_tile_coord);
        let next_tile_color = (*state.possible_vals[next_tile_coord.0 as usize]
            [next_tile_coord.1 as usize]
            .iter()
            .collect::<Vec<_>>()
            .choose(&mut rand::thread_rng())
            .unwrap())
        .clone();
        state.possible_vals[next_tile_coord.0 as usize][next_tile_coord.1 as usize] =
            HashSet::new().with(next_tile_color.clone());
        print_tile_possibilities_and_adjacents(
            &state.possible_vals,
            next_tile_coord.0,
            next_tile_coord.1,
        );
    }

    return if let Some(img) = get_image_from_possible_vals(&state) {
        Some(img)
    } else {
        None
    };
}

fn main() {
    let file_name = "imgs/noel2.bmp";
    let final_file_name = "imgs/noel_final.bmp";
    let input_img = read_bitmap(file_name);
    let rules = extract_rules(&input_img);
    let res_img = generate_image(16, 16, &rules);
    println!("Generated image: {:?}", res_img);
    if let Some(img) = res_img {
        save_bitmap(img, final_file_name);
    }
}
