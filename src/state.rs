use std::{collections::HashSet, path::PathBuf};

use bmp::Image;

use crate::{enums::Tile, files::delete_files_in_dir, rules::{apply_rules, get_possibilities_adjacent_pixels, Rule}};
use std::io::Write;
use rand::prelude::SliceRandom;

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

        w.flush().expect("Should be able to flush writer buffer.");
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
        w.flush().expect("Should be able to flush writer buffer.");
    }

    pub fn get(&self, x: usize, y: usize) -> HashSet<Tile> {
        self.possible_vals[x][y].clone()
    }

    pub fn get_total_entropy(&self) -> usize {
        self.possible_vals.iter().flatten().flatten().count()
    }
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

fn get_lowest_entropy_tiles(possible_vals: &Vec<Vec<HashSet<Tile>>>) -> Option<(usize, usize)> {
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

    while state
        .possible_vals
        .iter()
        .any(|row| row.iter().any(|tile| tile.len() > 1))
        {
        let mut old_state = state.clone();
        let next_tile_coord = get_lowest_entropy_tiles(&state.possible_vals);
        if next_tile_coord.is_none() {
            break;
        }
        let next_tile_coord = next_tile_coord.unwrap();
        let next_tile_color = (*state.possible_vals[next_tile_coord.0 as usize]
            [next_tile_coord.1 as usize]
            .iter()
            .collect::<Vec<_>>()
            .choose(&mut rand::thread_rng())
            .unwrap())
        .clone();
        state.possible_vals[next_tile_coord.0 as usize][next_tile_coord.1 as usize] =
            HashSet::new().with(next_tile_color.clone());

        apply_rules(&mut state, rules);
        while contains_invalid_tiles(&state.possible_vals) {
            state.possible_vals = old_state.possible_vals.clone();
            let next_tile_coord = get_lowest_entropy_tiles(&state.possible_vals);
            if next_tile_coord.is_none() {
                break;
            }
            let next_tile_coord = next_tile_coord.unwrap();
            let next_tile_color = (*state.possible_vals[next_tile_coord.0 as usize]
                [next_tile_coord.1 as usize]
                .iter()
                .collect::<Vec<_>>()
                .choose(&mut rand::thread_rng())
                .unwrap())
            .clone();
            state.possible_vals[next_tile_coord.0 as usize][next_tile_coord.1 as usize] =
                HashSet::new().with(next_tile_color.clone());
            apply_rules(&mut state, rules);
            state.save_into_file("after_rule");
        }

        while old_state.get_total_entropy() != state.get_total_entropy() {
            old_state = state.clone();
            apply_rules(&mut state, rules);
        }
    }

    return if let Some(img) = get_image_from_possible_vals(&state) {
        Some(img)
    } else {
        None
    };
}


#[cfg(test)]
mod tests {
    mod get_lowest_entropy_tiles {
        #[test]
        pub fn test1() {
            assert_eq!(1+1, 2);
        }
    }
}