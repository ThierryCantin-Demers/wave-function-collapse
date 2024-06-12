use std::collections::HashSet;

use bmp::{Image, Pixel};

use crate::{
    enums::{Direction, Tile},
    state::State,
};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Rule {
    pub curr_tile: Tile,
    pub adj_tile: Tile,
    pub direction: Direction,
}

impl Rule {
    pub fn new(curr_tile: Tile, adj_tile: Tile, direction: Direction) -> Self {
        Rule {
            curr_tile,
            adj_tile,
            direction,
        }
    }
}

pub fn extract_rules(img: &Image) -> HashSet<Rule> {
    let mut rules: HashSet<Rule> = HashSet::new();
    for (x, y) in img.coordinates() {
        let curr_tile = img.get_pixel(x, y);
        let (up, down, left, right) = get_image_adjacent_pixels(img, x, y);

        if up.is_some() {
            rules.insert(Rule::new(
                up.unwrap().into(),
                curr_tile.into(),
                Direction::Up,
            ));
        }
        if down.is_some() {
            rules.insert(Rule::new(
                down.unwrap().into(),
                curr_tile.into(),
                Direction::Down,
            ));
        }
        if left.is_some() {
            rules.insert(Rule::new(
                left.unwrap().into(),
                curr_tile.into(),
                Direction::Left,
            ));
        }
        if right.is_some() {
            rules.insert(Rule::new(
                right.unwrap().into(),
                curr_tile.into(),
                Direction::Right,
            ));
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
                if !possibilities.contains(&rule.curr_tile) {
                    continue;
                }
                let (dx, dy) = rule.direction.offset();
                let adj_x = x as i32 + dx;
                let adj_y = y as i32 + dy;
                if adj_x < 0 || adj_x >= w as i32 || adj_y < 0 || adj_y >= h as i32 {
                    continue;
                }
                let adj_x = adj_x as usize;
                let adj_y = adj_y as usize;
                let adj_possibilities = curr_state.get(adj_x, adj_y);
                if !adj_possibilities.contains(&rule.adj_tile) {
                    new_state[x][y].remove(&rule.curr_tile);
                    if possibilities.len() == 1 {
                        break;
                    }
                }
            }
        }
    }

    curr_state.possible_vals = new_state.clone();
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

#[cfg(test)]
mod tests {
    use crate::state::HashSetExt;

    use super::*;

    #[test]
    fn test_extract_rules() {
        let mut img = Image::new(2, 2);
        img.set_pixel(0, 0, Tile::Red.into());
        img.set_pixel(1, 0, Tile::Blue.into());
        img.set_pixel(0, 1, Tile::Red.into());
        img.set_pixel(1, 1, Tile::Red.into());

        let rules = extract_rules(&img);
        let expected = HashSet::new().with_all(vec![
            Rule::new(Tile::Red, Tile::Red, Direction::Down),
            Rule::new(Tile::Red, Tile::Red, Direction::Up),
            Rule::new(Tile::Red, Tile::Red, Direction::Right),
            Rule::new(Tile::Red, Tile::Red, Direction::Left),
            Rule::new(Tile::Red, Tile::Blue, Direction::Left),
            Rule::new(Tile::Blue, Tile::Red, Direction::Right),
            Rule::new(Tile::Red, Tile::Blue, Direction::Down),
            Rule::new(Tile::Blue, Tile::Red, Direction::Up),
        ]);

        assert_eq!(rules, expected);
    }
}
