use std::{collections::HashSet, path::PathBuf};

use bmp::Image;

use crate::{
    enums::Tile,
    files::delete_files_in_dir,
    rules::{apply_rules, get_possibilities_adjacent_pixels, Rule},
};
use rand::prelude::SliceRandom;
use std::fmt::Debug;
use std::io::Write;

#[derive(Clone)]
pub struct PossibleVals {
    pub inner: Vec<Vec<HashSet<Tile>>>,
}

impl PossibleVals {
    pub fn set(&mut self, x: usize, y: usize, value: HashSet<Tile>) {
        self.inner[x][y] = value;
    }

    pub fn get(&self, x: usize, y: usize) -> HashSet<Tile> {
        self.inner[x][y].clone()
    }

    pub fn size(&self) -> Option<(usize, usize)> {
        let w = self.inner.len();
        if w == 0 {
            return None;
        }
        let h = self.inner[0].len();
        Some((w, h))
    }
}

impl From<Vec<Vec<HashSet<Tile>>>> for PossibleVals {
    fn from(inner: Vec<Vec<HashSet<Tile>>>) -> Self {
        PossibleVals { inner }
    }
}

impl Debug for PossibleVals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Calculate the maximum length of the string representation of any value in each column
        let mut max_lengths = self.inner[0].iter().map(|_| 0).collect::<Vec<_>>();
        for row in self.inner.iter() {
            for (tile, max_length) in row.iter().zip(max_lengths.iter_mut()) {
                *max_length = (*max_length).max(format!("{:?}", tile).len());
            }
        }

        writeln!(f)?;
        for row in self.inner.iter() {
            for (tile, &max_length) in row.iter().zip(max_lengths.iter()) {
                // Use the maximum length as the width specifier
                let mut sorted = tile.iter().collect::<Vec<_>>();
                sorted.sort_unstable();
                let s = format!("{:width$}", format!("{:?}", sorted), width = max_length);
                write!(f, "{} ", s)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub possible_vals: PossibleVals,
    pub width: usize,
    pub height: usize,
    pub curr_file_index: u32,
}

impl State {
    pub fn new(w: usize, h: usize, all_tiles_types: &HashSet<Tile>) -> Self {
        State {
            possible_vals: PossibleVals {
                inner: vec![vec![all_tiles_types.clone(); w]; h],
            },
            curr_file_index: 0,
            width: w,
            height: h,
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
        let max_col = self.possible_vals.inner[0].len();
        for col in 0..max_col {
            for row in &self.possible_vals.inner {
                if let Some(tile) = row.get(col) {
                    for t in tile {
                        let _ = write!(w, "{}", t.minify());
                    }
                    let _ = write!(w, "\t");
                }
            }
            let _ = writeln!(w);
        }

        w.flush().expect("Should be able to flush writer buffer.");
    }

    pub fn save_rules_into_file(&self, rules: &HashSet<Rule>) {
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
        for rule in rules {
            let _ = writeln!(
                w,
                "{:?} can be at {:?} of {:?}",
                rule.curr_tile, rule.direction, rule.adj_tile
            );
        }
        w.flush().expect("Should be able to flush writer buffer.");
    }

    pub fn get(&self, x: usize, y: usize) -> HashSet<Tile> {
        self.possible_vals.inner[x][y].clone()
    }

    pub fn get_total_entropy(&self) -> usize {
        self.possible_vals.inner.iter().flatten().flatten().count()
    }

    pub fn with_possibilities(&self, possible_vals: PossibleVals) -> Self {
        let mut new_state = self.clone();
        new_state.possible_vals = possible_vals;
        new_state
    }
}

pub fn contains_invalid_tiles(possible_vals: &PossibleVals) -> bool {
    for row in possible_vals.inner.iter() {
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

fn get_lowest_entropy_tile(possible_vals: &PossibleVals) -> Option<(usize, usize)> {
    let mut min_entropy = usize::MAX;
    let mut min_entropy_tiles = Vec::new();

    for (i, row) in possible_vals.inner.iter().enumerate() {
        for (j, tile) in row.iter().enumerate() {
            let entropy = tile.len();

            if entropy == 1 {
                continue;
            }

            match entropy.cmp(&min_entropy) {
                std::cmp::Ordering::Less => {
                    min_entropy = entropy;
                    min_entropy_tiles.clear();
                    min_entropy_tiles.push((i, j));
                }
                std::cmp::Ordering::Equal => {
                    min_entropy_tiles.push((i, j));
                }
                std::cmp::Ordering::Greater => {}
            }
        }
    }

    min_entropy_tiles.choose(&mut rand::thread_rng()).cloned()
}

pub trait HashSetExt<T> {
    fn with(self, value: T) -> HashSet<T>;
    fn with_all(self, values: Vec<T>) -> HashSet<T>;
    fn from_all(values: Vec<T>) -> HashSet<T>;
}

impl<T: std::hash::Hash + Eq + Clone> HashSetExt<T> for HashSet<T> {
    fn with(mut self, value: T) -> HashSet<T> {
        self.insert(value);
        self
    }

    fn with_all(mut self, values: Vec<T>) -> HashSet<T> {
        for value in values {
            self.insert(value);
        }
        self
    }

    fn from_all(values: Vec<T>) -> HashSet<T> {
        let mut new_set = HashSet::new();
        for value in values {
            new_set.insert(value);
        }
        new_set
    }
}

pub fn print_tile_possibilities_and_adjacents(possible_vals: &PossibleVals, x: usize, y: usize) {
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
        all_tiles_types.insert(rule.curr_tile.clone());
        all_tiles_types.insert(rule.adj_tile.clone());
    }

    let mut state: State = State::new(w as usize, h as usize, &all_tiles_types);
    state.save_rules_into_file(rules);
    state.save_into_file("initial");

    while state
        .possible_vals
        .inner
        .iter()
        .any(|row| row.iter().any(|tile| tile.len() > 1))
    {
        let mut old_state = state.clone();
        let next_tile_coord = get_lowest_entropy_tile(&state.possible_vals);
        if next_tile_coord.is_none() {
            break;
        }
        let next_tile_coord = next_tile_coord.unwrap();
        let next_tile_color = (*state.possible_vals.inner[next_tile_coord.0 as usize]
            [next_tile_coord.1 as usize]
            .iter()
            .collect::<Vec<_>>()
            .choose(&mut rand::thread_rng())
            .unwrap())
        .clone();
        state.possible_vals.inner[next_tile_coord.0 as usize][next_tile_coord.1 as usize] =
            HashSet::new().with(next_tile_color.clone());

        apply_rules(&state, rules);
        while contains_invalid_tiles(&state.possible_vals) {
            state.possible_vals = old_state.possible_vals.clone();
            let next_tile_coord = get_lowest_entropy_tile(&state.possible_vals);
            if next_tile_coord.is_none() {
                break;
            }
            let next_tile_coord = next_tile_coord.unwrap();
            let next_tile_color = (*state.possible_vals.inner[next_tile_coord.0 as usize]
                [next_tile_coord.1 as usize]
                .iter()
                .collect::<Vec<_>>()
                .choose(&mut rand::thread_rng())
                .unwrap())
            .clone();
            state.possible_vals.inner[next_tile_coord.0 as usize][next_tile_coord.1 as usize] =
                HashSet::new().with(next_tile_color.clone());
            apply_rules(&state, rules);
            state.save_into_file("after_rule");
        }

        while old_state.get_total_entropy() != state.get_total_entropy() {
            old_state = state.clone();
            apply_rules(&state, rules);
        }
    }

    get_image_from_possible_vals(&state)
}

#[cfg(test)]
mod tests {

    mod get_lowest_entropy_tile {
        use rstest::{fixture, rstest};
        use std::collections::HashSet;

        use crate::{
            enums::Tile,
            state::{get_lowest_entropy_tile, HashSetExt, PossibleVals},
        };

        #[fixture]
        fn one_at_2() -> PossibleVals {
            PossibleVals {
                inner: vec![
                    vec![
                        HashSet::new().with(Tile::Blue).with(Tile::Green),
                        HashSet::new()
                            .with(Tile::Blue)
                            .with(Tile::Green)
                            .with(Tile::Red),
                    ],
                    vec![
                        HashSet::new()
                            .with(Tile::Blue)
                            .with(Tile::Green)
                            .with(Tile::Red),
                        HashSet::new()
                            .with(Tile::Blue)
                            .with(Tile::Green)
                            .with(Tile::Red),
                    ],
                ],
            }
        }

        #[fixture]
        fn one_at_2_and_1() -> PossibleVals {
            PossibleVals {
                inner: vec![
                    vec![
                        HashSet::new().with(Tile::Blue),
                        HashSet::new().with(Tile::Blue).with(Tile::Green),
                    ],
                    vec![
                        HashSet::new()
                            .with(Tile::Blue)
                            .with(Tile::Green)
                            .with(Tile::Red),
                        HashSet::new()
                            .with(Tile::Blue)
                            .with(Tile::Green)
                            .with(Tile::Red),
                    ],
                ],
            }
        }

        #[fixture]
        fn one_at_1() -> PossibleVals {
            PossibleVals {
                inner: vec![
                    vec![
                        HashSet::new().with(Tile::Blue),
                        HashSet::new().with(Tile::Blue).with(Tile::Green),
                    ],
                    vec![
                        HashSet::new().with(Tile::Blue).with(Tile::Green),
                        HashSet::new().with(Tile::Blue).with(Tile::Green),
                    ],
                ],
            }
        }

        #[fixture]
        fn all_at_1() -> PossibleVals {
            PossibleVals {
                inner: vec![
                    vec![
                        HashSet::new().with(Tile::Blue),
                        HashSet::new().with(Tile::Green),
                    ],
                    vec![
                        HashSet::new().with(Tile::Red),
                        HashSet::new().with(Tile::Blue),
                    ],
                ],
            }
        }

        #[rstest]
        pub fn test1(one_at_2: PossibleVals) {
            println!("{:?}", one_at_2.inner[0][0]);
            let res = get_lowest_entropy_tile(&one_at_2);
            assert_eq!(res, Some((0, 0)));
        }

        #[rstest]
        pub fn test2(one_at_2_and_1: PossibleVals) {
            let res = get_lowest_entropy_tile(&one_at_2_and_1);
            assert_eq!(res, Some((0, 1)));
        }

        #[rstest]
        pub fn test3(one_at_1: PossibleVals) {
            for _ in 0..1000 {
                let res = get_lowest_entropy_tile(&one_at_1);
                assert_ne!(res, Some((0, 0)));
            }
        }

        #[rstest]
        pub fn test4(all_at_1: PossibleVals) {
            let res = get_lowest_entropy_tile(&all_at_1);
            assert_eq!(res, None);
        }
    }

    mod contains_invalid_tiles {
        use rstest::{fixture, rstest};
        use std::collections::HashSet;

        use crate::{
            enums::Tile,
            state::{contains_invalid_tiles, HashSetExt, PossibleVals},
        };

        #[fixture]
        fn all_ok() -> PossibleVals {
            PossibleVals {
                inner: vec![
                    vec![
                        HashSet::new().with(Tile::Blue).with(Tile::Green),
                        HashSet::new()
                            .with(Tile::Blue)
                            .with(Tile::Green)
                            .with(Tile::Red),
                    ],
                    vec![
                        HashSet::new()
                            .with(Tile::Blue)
                            .with(Tile::Green)
                            .with(Tile::Red),
                        HashSet::new()
                            .with(Tile::Blue)
                            .with(Tile::Green)
                            .with(Tile::Red),
                    ],
                ],
            }
        }

        #[fixture]
        fn one_not_ok() -> PossibleVals {
            PossibleVals {
                inner: vec![
                    vec![
                        HashSet::new(),
                        HashSet::new()
                            .with(Tile::Blue)
                            .with(Tile::Green)
                            .with(Tile::Red),
                    ],
                    vec![
                        HashSet::new()
                            .with(Tile::Blue)
                            .with(Tile::Green)
                            .with(Tile::Red),
                        HashSet::new()
                            .with(Tile::Blue)
                            .with(Tile::Green)
                            .with(Tile::Red),
                    ],
                ],
            }
        }

        #[fixture]
        fn all_not_ok() -> PossibleVals {
            PossibleVals {
                inner: vec![
                    vec![HashSet::new(), HashSet::new()],
                    vec![HashSet::new(), HashSet::new()],
                ],
            }
        }

        #[rstest]
        pub fn test1(all_ok: PossibleVals) {
            let res = contains_invalid_tiles(&all_ok);
            assert_eq!(res, false);
        }

        #[rstest]
        pub fn test2(one_not_ok: PossibleVals) {
            let res = contains_invalid_tiles(&one_not_ok);
            assert_eq!(res, true);
        }

        #[rstest]
        pub fn test3(all_not_ok: PossibleVals) {
            let res = contains_invalid_tiles(&all_not_ok);
            assert_eq!(res, true);
        }
    }
}
