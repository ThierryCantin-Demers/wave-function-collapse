use std::collections::HashSet;

use bmp::{Image, Pixel};

use crate::{
    enums::{Direction, Tile},
    state::{contains_invalid_tiles, PossibleVals, State},
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

pub fn apply_rules(curr_state: &State, rules: &HashSet<Rule>) -> Option<State> {
    let w = curr_state.width;
    let h = curr_state.height;
    let mut new_possibilities = curr_state.possible_vals.clone();

    for x in 0..w {
        for y in 0..h {
            let possibilities = &curr_state.possible_vals.get(x, y);
            if possibilities.len() == 1 {
                continue;
            }

            let (_, up, down, left, right) =
                get_possibilities_adjacent_pixels(&curr_state.possible_vals, x, y);
            let mut new_tile_possibilities: HashSet<Tile> = HashSet::new();
            for possibility in possibilities.iter() {
                let mut valid = true;
                for rule in rules.iter() {
                    if rule.curr_tile != *possibility {
                        continue;
                    }
                    if rule.direction == Direction::Up && up.is_some() {
                        if !up.as_ref().unwrap().contains(&rule.adj_tile) {
                            valid = false;
                            break;
                        }
                    } else if rule.direction == Direction::Down && down.is_some() {
                        if !down.as_ref().unwrap().contains(&rule.adj_tile) {
                            valid = false;
                            break;
                        }
                    } else if rule.direction == Direction::Left && left.is_some() {
                        if !left.as_ref().unwrap().contains(&rule.adj_tile) {
                            valid = false;
                            break;
                        }
                    } else if rule.direction == Direction::Right && right.is_some() {
                        if !right.as_ref().unwrap().contains(&rule.adj_tile) {
                            valid = false;
                            break;
                        }
                    }
                }
                if valid {
                    new_tile_possibilities.insert(possibility.clone());
                }
            }
            new_possibilities.set(x, y, new_tile_possibilities);
        }
    }

    if contains_invalid_tiles(&new_possibilities) {
        return None;
    }

    Some(curr_state.with_possibilities(new_possibilities))
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
    possible_vals: &PossibleVals,
    x: usize,
    y: usize,
) -> (
    Option<HashSet<Tile>>,
    Option<HashSet<Tile>>,
    Option<HashSet<Tile>>,
    Option<HashSet<Tile>>,
    Option<HashSet<Tile>>,
) {
    let size = possible_vals.size();
    if size.is_none() {
        return (None, None, None, None, None);
    }
    let (w, h) = size.unwrap();

    let curr = possible_vals.get(x as usize, y as usize).clone();
    let up = if y > 0 {
        Some(possible_vals.get(x as usize, y as usize - 1).clone())
    } else {
        None
    };
    let down = if y < h - 1 {
        Some(possible_vals.get(x as usize, y as usize + 1).clone())
    } else {
        None
    };
    let left = if x > 0 {
        Some(possible_vals.get(x as usize - 1, y as usize).clone())
    } else {
        None
    };
    let right = if x < w - 1 {
        Some(possible_vals.get(x as usize + 1, y as usize).clone())
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
        let expected = HashSet::from_all(vec![
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

    mod get_possibilities_adjacent_pixels {
        use rstest::{fixture, rstest};

        use crate::state::print_tile_possibilities_and_adjacents;

        use super::*;

        #[fixture]
        fn possible_vals_1x1() -> PossibleVals {
            PossibleVals::from(vec![vec![HashSet::from_all(vec![Tile::Red, Tile::Green])]])
        }

        #[fixture]
        fn possible_vals_1x2() -> PossibleVals {
            PossibleVals::from(vec![vec![
                HashSet::from_all(vec![Tile::Red, Tile::Green]),
                HashSet::from_all(vec![Tile::Red, Tile::Green]),
            ]])
        }

        #[fixture]
        fn possible_vals_3x3() -> PossibleVals {
            PossibleVals::from(vec![
                vec![
                    HashSet::from_all(vec![Tile::Green]),
                    HashSet::from_all(vec![Tile::Red, Tile::Green]),
                    HashSet::from_all(vec![Tile::Red]),
                ],
                vec![
                    HashSet::from_all(vec![Tile::Green, Tile::Blue]),
                    HashSet::from_all(vec![Tile::Red, Tile::Green, Tile::Blue]),
                    HashSet::from_all(vec![Tile::Red, Tile::Blue]),
                ],
                vec![
                    HashSet::from_all(vec![Tile::Blue]),
                    HashSet::from_all(vec![Tile::Green, Tile::Red]),
                    HashSet::from_all(vec![Tile::Red, Tile::Green]),
                ],
            ])
        }

        #[fixture]
        fn possible_vals_0x1() -> PossibleVals {
            PossibleVals::from(vec![])
        }

        #[rstest]
        fn test_1x1(possible_vals_1x1: PossibleVals) {
            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_1x1, 0, 0);
            assert_eq!(curr, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(up, None);
            assert_eq!(down, None);
            assert_eq!(left, None);
            assert_eq!(right, None);
        }

        #[rstest]
        fn test_1x2(possible_vals_1x2: PossibleVals) {
            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_1x2, 0, 0);
            assert_eq!(curr, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(up, None);
            assert_eq!(down, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(left, None);
            assert_eq!(right, None);

            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_1x2, 0, 1);
            assert_eq!(curr, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(up, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(down, None);
            assert_eq!(left, None);
            assert_eq!(right, None);
        }

        #[rstest]
        fn test_3x3(possible_vals_3x3: PossibleVals) {
            print_tile_possibilities_and_adjacents(&possible_vals_3x3, 0, 0);
            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_3x3, 0, 0);
            assert_eq!(curr, Some(HashSet::from_all(vec![Tile::Green])));
            assert_eq!(up, None);
            assert_eq!(down, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(left, None);
            assert_eq!(
                right,
                Some(HashSet::from_all(vec![Tile::Green, Tile::Blue]))
            );

            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_3x3, 1, 0);
            assert_eq!(curr, Some(HashSet::from_all(vec![Tile::Green, Tile::Blue])));
            assert_eq!(up, None);
            assert_eq!(
                down,
                Some(HashSet::from_all(vec![Tile::Red, Tile::Green, Tile::Blue]))
            );
            assert_eq!(left, Some(HashSet::from_all(vec![Tile::Green])));
            assert_eq!(right, Some(HashSet::from_all(vec![Tile::Blue])));

            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_3x3, 2, 0);
            assert_eq!(curr, Some(HashSet::from_all(vec![Tile::Blue])));
            assert_eq!(up, None);
            assert_eq!(down, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(left, Some(HashSet::from_all(vec![Tile::Green, Tile::Blue])));
            assert_eq!(right, None);

            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_3x3, 0, 1);
            assert_eq!(curr, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(up, Some(HashSet::from_all(vec![Tile::Green])));
            assert_eq!(down, Some(HashSet::from_all(vec![Tile::Red])));
            assert_eq!(left, None);
            assert_eq!(
                right,
                Some(HashSet::from_all(vec![Tile::Red, Tile::Green, Tile::Blue]))
            );

            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_3x3, 1, 1);
            assert_eq!(
                curr,
                Some(HashSet::from_all(vec![Tile::Red, Tile::Green, Tile::Blue]))
            );
            assert_eq!(up, Some(HashSet::from_all(vec![Tile::Green, Tile::Blue])));
            assert_eq!(down, Some(HashSet::from_all(vec![Tile::Red, Tile::Blue])));
            assert_eq!(left, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(right, Some(HashSet::from_all(vec![Tile::Green, Tile::Red])));

            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_3x3, 2, 1);
            assert_eq!(curr, Some(HashSet::from_all(vec![Tile::Green, Tile::Red])));
            assert_eq!(up, Some(HashSet::from_all(vec![Tile::Blue])));
            assert_eq!(down, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(
                left,
                Some(HashSet::from_all(vec![Tile::Red, Tile::Green, Tile::Blue]))
            );
            assert_eq!(right, None);

            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_3x3, 0, 2);
            assert_eq!(curr, Some(HashSet::from_all(vec![Tile::Red])));
            assert_eq!(up, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(down, None);
            assert_eq!(left, None);
            assert_eq!(right, Some(HashSet::from_all(vec![Tile::Red, Tile::Blue])));

            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_3x3, 1, 2);
            assert_eq!(curr, Some(HashSet::from_all(vec![Tile::Red, Tile::Blue])));
            assert_eq!(
                up,
                Some(HashSet::from_all(vec![Tile::Red, Tile::Green, Tile::Blue]))
            );
            assert_eq!(down, None);
            assert_eq!(left, Some(HashSet::from_all(vec![Tile::Red])));
            assert_eq!(right, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));

            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_3x3, 2, 2);
            assert_eq!(curr, Some(HashSet::from_all(vec![Tile::Red, Tile::Green])));
            assert_eq!(up, Some(HashSet::from_all(vec![Tile::Green, Tile::Red])));
            assert_eq!(down, None);
            assert_eq!(left, Some(HashSet::from_all(vec![Tile::Red, Tile::Blue])));
            assert_eq!(right, None);
        }

        #[rstest]
        fn test_0x1(possible_vals_0x1: PossibleVals) {
            let (curr, up, down, left, right) =
                get_possibilities_adjacent_pixels(&possible_vals_0x1, 0, 0);
            assert_eq!(curr, None);
            assert_eq!(up, None);
            assert_eq!(down, None);
            assert_eq!(left, None);
            assert_eq!(right, None);
        }
    }

    mod apply_rules {
        use rstest::{fixture, rstest};

        use crate::state::State;

        use super::*;

        #[fixture]
        fn state_1x1_rg() -> State {
            State::new(1, 1, &HashSet::from_all(vec![Tile::Red, Tile::Green]))
        }

        #[fixture]
        fn state_2x2_rg() -> State {
            State::new(2, 2, &HashSet::from_all(vec![Tile::Red, Tile::Green]))
        }

        #[fixture]
        fn state_3x3_rg() -> State {
            State::new(3, 3, &HashSet::from_all(vec![Tile::Red, Tile::Green]))
        }

        #[fixture]
        fn rules_red_green_ud() -> HashSet<Rule> {
            HashSet::from_all(vec![
                Rule::new(Tile::Red, Tile::Red, Direction::Down),
                Rule::new(Tile::Red, Tile::Red, Direction::Up),
                Rule::new(Tile::Green, Tile::Green, Direction::Down),
                Rule::new(Tile::Green, Tile::Green, Direction::Up),
            ])
        }

        #[fixture]
        fn rules_red_green_lr() -> HashSet<Rule> {
            HashSet::from_all(vec![
                Rule::new(Tile::Red, Tile::Red, Direction::Left),
                Rule::new(Tile::Red, Tile::Red, Direction::Right),
                Rule::new(Tile::Green, Tile::Green, Direction::Left),
                Rule::new(Tile::Green, Tile::Green, Direction::Right),
            ])
        }

        #[fixture]
        fn rules_red_udlr() -> HashSet<Rule> {
            HashSet::from_all(vec![
                Rule::new(Tile::Red, Tile::Red, Direction::Down),
                Rule::new(Tile::Red, Tile::Red, Direction::Up),
                Rule::new(Tile::Red, Tile::Red, Direction::Left),
                Rule::new(Tile::Red, Tile::Red, Direction::Right),
            ])
        }

        #[rstest]
        fn test_1x1(state_1x1_rg: State, rules_red_green_ud: HashSet<Rule>) {
            let new_state = apply_rules(&state_1x1_rg, &rules_red_green_ud);
            assert!(new_state.is_some());
            let new_state = new_state.unwrap();
            assert_eq!(
                new_state.possible_vals.inner[0][0],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
        }

        #[rstest]
        fn test_2x2_no_collapse(state_2x2_rg: State, rules_red_green_ud: HashSet<Rule>) {
            let new_state = apply_rules(&state_2x2_rg, &rules_red_green_ud);
            assert!(new_state.is_some());
            let new_state = new_state.unwrap();
            assert_eq!(
                new_state.possible_vals.inner[0][0],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][0],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[0][1],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][1],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
        }

        #[rstest]
        fn test_2x2_collapse(mut state_2x2_rg: State, rules_red_green_ud: HashSet<Rule>) {
            state_2x2_rg.possible_vals.inner[0][0] = HashSet::from_all(vec![Tile::Red]);

            let new_state = apply_rules(&state_2x2_rg, &rules_red_green_ud);
            assert!(new_state.is_some());
            let new_state = new_state.unwrap();

            assert_eq!(
                new_state.possible_vals.inner[0][0],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][0],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[0][1],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][1],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
        }

        #[rstest]
        fn test_3x3_collapse_vertical(mut state_3x3_rg: State, rules_red_green_ud: HashSet<Rule>) {
            state_3x3_rg.possible_vals.inner[0][0] = HashSet::from_all(vec![Tile::Red]);
            state_3x3_rg.possible_vals.inner[2][0] = HashSet::from_all(vec![Tile::Green]);

            let new_state = apply_rules(&state_3x3_rg, &rules_red_green_ud);
            assert!(new_state.is_some());
            let new_state = new_state.unwrap();

            assert_eq!(
                new_state.possible_vals.inner[0][0],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[0][1],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[0][2],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );

            assert_eq!(
                new_state.possible_vals.inner[1][0],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][1],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][2],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );

            assert_eq!(
                new_state.possible_vals.inner[2][0],
                HashSet::from_all(vec![Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][1],
                HashSet::from_all(vec![Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][2],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );

            let new_state = apply_rules(&new_state, &rules_red_green_ud);
            assert!(new_state.is_some());
            let new_state = new_state.unwrap();

            assert_eq!(
                new_state.possible_vals.inner[0][0],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[0][1],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[0][2],
                HashSet::from_all(vec![Tile::Red])
            );

            assert_eq!(
                new_state.possible_vals.inner[1][0],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][1],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][2],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );

            assert_eq!(
                new_state.possible_vals.inner[2][0],
                HashSet::from_all(vec![Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][1],
                HashSet::from_all(vec![Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][2],
                HashSet::from_all(vec![Tile::Green])
            );
        }

        #[rstest]
        fn test_3x3_collapse_horizontal(
            mut state_3x3_rg: State,
            rules_red_green_lr: HashSet<Rule>,
        ) {
            state_3x3_rg.possible_vals.inner[0][0] = HashSet::from_all(vec![Tile::Red]);
            state_3x3_rg.possible_vals.inner[0][2] = HashSet::from_all(vec![Tile::Green]);

            let new_state = apply_rules(&state_3x3_rg, &rules_red_green_lr);
            assert!(new_state.is_some());
            let new_state = new_state.unwrap();
            assert_eq!(
                new_state.possible_vals.inner[0][0],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][0],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][0],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );

            assert_eq!(
                new_state.possible_vals.inner[0][1],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][1],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][1],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );

            assert_eq!(
                new_state.possible_vals.inner[0][2],
                HashSet::from_all(vec![Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][2],
                HashSet::from_all(vec![Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][2],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );

            let new_state = apply_rules(&new_state, &rules_red_green_lr);
            assert!(new_state.is_some());
            let new_state = new_state.unwrap();

            assert_eq!(
                new_state.possible_vals.inner[0][0],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][0],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][0],
                HashSet::from_all(vec![Tile::Red])
            );

            assert_eq!(
                new_state.possible_vals.inner[0][1],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][1],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][1],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );

            assert_eq!(
                new_state.possible_vals.inner[0][2],
                HashSet::from_all(vec![Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][2],
                HashSet::from_all(vec![Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][2],
                HashSet::from_all(vec![Tile::Green])
            );
        }

        #[rstest]
        fn test_3x3_collapse_all_red(mut state_3x3_rg: State, rules_red_udlr: HashSet<Rule>) {
            state_3x3_rg.possible_vals.inner[1][1] = HashSet::from_all(vec![Tile::Red]);

            println!("Initial state: {:?}", state_3x3_rg);

            let new_state = apply_rules(&state_3x3_rg, &rules_red_udlr);
            assert!(new_state.is_some());
            let new_state = new_state.unwrap();

            println!("After first apply rule: {:?}", new_state);

            assert_eq!(
                new_state.possible_vals.inner[0][0],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][0],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][0],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );

            assert_eq!(
                new_state.possible_vals.inner[0][1],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][1],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][1],
                HashSet::from_all(vec![Tile::Red])
            );

            assert_eq!(
                new_state.possible_vals.inner[0][2],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
            assert_eq!(
                new_state.possible_vals.inner[1][2],
                HashSet::from_all(vec![Tile::Red])
            );
            assert_eq!(
                new_state.possible_vals.inner[2][2],
                HashSet::from_all(vec![Tile::Red, Tile::Green])
            );
        }
    }
}
