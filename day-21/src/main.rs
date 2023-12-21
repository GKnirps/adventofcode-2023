use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let (start_x, start_y, garden) = parse(&content)?;
    let n = possible_plots(start_x, start_y, &garden, 64);
    println!("With exactly 64 steps, the elf can reach {n} plots.");

    let n_inf = possible_plots_on_infinite_garden(start_x, start_y, &garden, 26501365)?;
    println!("On the infinite garden, there are {n_inf} plots the elf can reach in 26501365 steps");

    Ok(())
}

fn possible_plots_on_infinite_garden(
    start_x: usize,
    start_y: usize,
    garden: &Garden,
    steps: u128,
) -> Result<u128, String> {
    // idea: get the distances to the corner plots and extrapolate from there
    // assumption: the tiles on the edge are always garden plots (they are for my input and for the
    // example)
    // another assumption: the input is quadratic
    if garden.width != garden.height() {
        return Err("assumed that the garden is a square, but it is not".to_string());
    }
    for x in 0..garden.width {
        if garden.get(x, 0) != Some(Tile::Plot)
            || garden.get(0, x) != Some(Tile::Plot)
            || garden.get(x, garden.width - 1) != Some(Tile::Plot)
            || garden.get(garden.width - 1, x) != Some(Tile::Plot)
        {
            return Err("assumed that the border of the input is all walkable tiles".to_string());
        }
    }
    if garden.width % 2 != 1 {
        return Err("assumed odd width".to_string());
    }
    // yet another assumption: the starting point is in the exact centet, and the direct line from
    // the starting point to the end has no rocks on it
    if start_x != garden.width / 2 || start_y != garden.width / 2 {
        return Err("assumed the starting point to be in the center".to_string());
    }
    if !(0..garden.width).all(|d| {
        garden.get(d, garden.width / 2) == Some(Tile::Plot)
            && garden.get(garden.width / 2, d) == Some(Tile::Plot)
    }) {
        return Err("assumed straight lines from starting point to the edges".to_string());
    }

    // shortest paths starting at the starting point
    let shortest_path_start = explore(start_x, start_y, garden);
    // shortest paths when the start is in the north-west corner (and north-east, south-west and
    // south-east respectively)
    let shortest_path_nw: HashMap<(usize, usize), usize> = explore(0, 0, garden);
    let shortest_path_ne: HashMap<(usize, usize), usize> = explore(garden.width - 1, 0, garden);
    let shortest_path_sw: HashMap<(usize, usize), usize> = explore(0, garden.width - 1, garden);
    let shortest_path_se: HashMap<(usize, usize), usize> =
        explore(garden.width - 1, garden.width - 1, garden);

    // shortest path when the start is in the middle of the northern edge (and easter, southern,
    // western respectively)
    let shortest_path_n: HashMap<(usize, usize), usize> = explore(garden.width / 2, 0, garden);
    let shortest_path_e: HashMap<(usize, usize), usize> =
        explore(garden.width - 1, garden.width / 2, garden);
    let shortest_path_s: HashMap<(usize, usize), usize> =
        explore(garden.width / 2, garden.width - 1, garden);
    let shortest_path_w: HashMap<(usize, usize), usize> = explore(0, garden.width / 2, garden);

    let corner_offset = garden.width as u128 - 1;
    let corner_range = (steps.saturating_sub(corner_offset + 2)) / (garden.width as u128);

    let (nw_even, nw_odd) = count_even_or_odd(&shortest_path_se, steps as usize);
    let (ne_even, ne_odd) = count_even_or_odd(&shortest_path_sw, steps as usize);
    let (sw_even, sw_odd) = count_even_or_odd(&shortest_path_ne, steps as usize);
    let (se_even, se_odd) = count_even_or_odd(&shortest_path_nw, steps as usize);
    let (n_even, n_odd) = count_even_or_odd(&shortest_path_s, steps as usize);
    let (e_even, e_odd) = count_even_or_odd(&shortest_path_w, steps as usize);
    let (s_even, s_odd) = count_even_or_odd(&shortest_path_n, steps as usize);
    let (w_even, w_odd) = count_even_or_odd(&shortest_path_e, steps as usize);

    let garden_range_straight =
        steps.saturating_sub(garden.width as u128 / 2 + 1) / garden.width as u128;
    let straight_steps = steps.saturating_sub(
        garden.width as u128 / 2 + 1 + garden.width as u128 * garden_range_straight,
    );
    let straight_partial = if straight_steps > 0 && straight_steps % 2 == 0 {
        count_even_or_odd(&shortest_path_s, straight_steps as usize).0
            + count_even_or_odd(&shortest_path_w, straight_steps as usize).0
            + count_even_or_odd(&shortest_path_n, straight_steps as usize).0
            + count_even_or_odd(&shortest_path_e, straight_steps as usize).0
    } else if straight_steps > 0 {
        count_even_or_odd(&shortest_path_s, straight_steps as usize).1
            + count_even_or_odd(&shortest_path_w, straight_steps as usize).1
            + count_even_or_odd(&shortest_path_n, straight_steps as usize).1
            + count_even_or_odd(&shortest_path_e, straight_steps as usize).1
    } else if garden_range_straight * garden.width as u128 + garden.width as u128 / 2 < steps {
        4
    } else {
        0
    };

    let (start_even, start_odd) = count_even_or_odd(&shortest_path_start, steps as usize);
    // possibilities in the starting garden
    Ok(if steps % 2 == 0 {
        start_even
    } else {
        start_odd
    } +
    // possibilities in the triangles (north-east, north-west, south-east, south-west)
    corner_full(corner_range, nw_even, nw_odd, steps)
        + corner_part(corner_range, garden.width as u128, corner_offset, steps, &shortest_path_se) +
    corner_full(corner_range, ne_even, ne_odd, steps)
        + corner_part(corner_range, garden.width as u128, corner_offset, steps, &shortest_path_sw) +
    corner_full(corner_range, sw_even, sw_odd, steps)
        + corner_part(corner_range, garden.width as u128, corner_offset, steps, &shortest_path_ne) +
    corner_full(corner_range, se_even, se_odd, steps)
        + corner_part(corner_range, garden.width as u128, corner_offset, steps, &shortest_path_nw) +
        if steps % 2 == 0 {
    (n_even + e_even + s_even + w_even) * (garden_range_straight/2 + garden_range_straight%2) + (n_odd + e_odd + s_odd + w_odd) * (garden_range_straight/2) + straight_partial
        } else {
    (n_odd + e_odd + s_odd + w_odd) * (garden_range_straight/2 + garden_range_straight%2) + (n_even + e_even + s_even + w_even) * (garden_range_straight/2) + straight_partial
        })
}

fn corner_full(range: u128, even: u128, odd: u128, total_steps: u128) -> u128 {
    let range = range.saturating_sub(1);
    let n_total = ((range + 1) * range) / 2;
    let n_odd = (range / 2 + 1) * (range / 2);
    let n_even = n_total - n_odd;
    if total_steps % 2 == 0 {
        n_even * even + n_odd * odd
    } else {
        n_even * odd + n_odd * even
    }
}

fn corner_part(
    range: u128,
    width: u128,
    offset: u128,
    total_steps: u128,
    distances: &HashMap<(usize, usize), usize>,
) -> u128 {
    if total_steps <= range * width + offset + 1 {
        return 0;
    }
    ({
        let remaining_steps = total_steps.saturating_sub(range * width + offset + 2) as usize;
        let (even, odd) = count_even_or_odd(distances, remaining_steps);
        (if remaining_steps % 2 == 0 { even } else { odd } * (range + 1))
    } + if range > 0 {
        let range = range - 1;
        let remaining_steps = total_steps.saturating_sub(range * width + offset + 2) as usize;
        let (even, odd) = count_even_or_odd(distances, remaining_steps);
        (if remaining_steps % 2 == 0 { even } else { odd } * (range + 1))
    } else {
        0
    })
}

fn count_even_or_odd(distances: &HashMap<(usize, usize), usize>, steps: usize) -> (u128, u128) {
    let mut even = 0;
    let mut odd = 0;
    for d in distances.values() {
        if *d <= steps {
            if d % 2 == 0 {
                even += 1
            } else {
                odd += 1
            }
        }
    }
    (even, odd)
}

fn possible_plots(start_x: usize, start_y: usize, garden: &Garden, steps: usize) -> u128 {
    let (even, odd) = count_even_or_odd(&explore(start_x, start_y, garden), steps);
    if steps % 2 == 0 {
        even
    } else {
        odd
    }
}

fn explore(start_x: usize, start_y: usize, garden: &Garden) -> HashMap<(usize, usize), usize> {
    let mut queue: VecDeque<(usize, usize, usize)> =
        VecDeque::with_capacity(garden.tiles.len() * 4);
    let mut visited: HashMap<(usize, usize), usize> = HashMap::with_capacity(garden.tiles.len());

    queue.push_back((start_x, start_y, 0));
    while let Some((x, y, steps)) = queue.pop_front() {
        if visited.contains_key(&(x, y)) {
            continue;
        }
        visited.insert((x, y), steps);
        if x > 0
            && garden
                .get(x - 1, y)
                .map(|tile| tile == Tile::Plot)
                .unwrap_or(false)
        {
            queue.push_back((x - 1, y, steps + 1));
        }
        if y > 0
            && garden
                .get(x, y - 1)
                .map(|tile| tile == Tile::Plot)
                .unwrap_or(false)
        {
            queue.push_back((x, y - 1, steps + 1));
        }
        if garden
            .get(x + 1, y)
            .map(|tile| tile == Tile::Plot)
            .unwrap_or(false)
        {
            queue.push_back((x + 1, y, steps + 1));
        }
        if garden
            .get(x, y + 1)
            .map(|tile| tile == Tile::Plot)
            .unwrap_or(false)
        {
            queue.push_back((x, y + 1, steps + 1));
        }
    }

    visited
}

fn parse(input: &str) -> Result<(usize, usize, Garden), String> {
    // assumption: input is rectangular
    let width = input
        .lines()
        .next()
        .ok_or_else(|| "no lines in input".to_string())?
        .len();
    if width == 0 {
        return Err("first input line is empty".to_string());
    }
    let tiles: Vec<Tile> = input
        .chars()
        .filter(|c| *c != '\n')
        .map(|c| match c {
            '.' | 'S' => Ok(Tile::Plot),
            '#' => Ok(Tile::Rock),
            _ => Err(format!("unknown tile: '{c}'")),
        })
        .collect::<Result<_, _>>()?;

    let start_i = input
        .chars()
        .filter(|c| *c != '\n')
        .position(|c| c == 'S')
        .ok_or_else(|| "no starting position in input".to_owned())?;

    Ok((start_i % width, start_i / width, Garden { width, tiles }))
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Tile {
    Plot,
    Rock,
}

#[derive(Clone, PartialEq, Hash, Debug)]
struct Garden {
    width: usize,
    tiles: Vec<Tile>,
}

impl Garden {
    fn height(&self) -> usize {
        self.tiles.len() / self.width
    }
    fn get(&self, x: usize, y: usize) -> Option<Tile> {
        if x >= self.width {
            None
        } else {
            self.tiles.get(x + y * self.width).copied()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
...........
"#;

    #[test]
    fn possible_plots_works_for_example() {
        // given
        let (sx, sy, garden) = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let n = possible_plots(sx, sy, &garden, 6);

        // then
        assert_eq!(n, 16);
    }

    #[test]
    fn possible_plots_on_infinite_garden_works_same_as_possible_plots_with_expanded_empty_area() {
        // given
        let mini = Garden {
            width: 3,
            tiles: vec![Tile::Plot; 9],
        };
        let expanded = Garden {
            width: 21,
            tiles: vec![Tile::Plot; 21 * 21],
        };

        for steps in 1..=10 {
            println!("steps: {steps}");
            // when
            let n_infinite = possible_plots_on_infinite_garden(1, 1, &mini, steps as u128);
            let n_expanded = possible_plots(10, 10, &expanded, steps);

            // then
            assert_eq!(n_infinite, Ok(n_expanded));
        }
    }

    #[test]
    fn possible_plots_on_infinite_garden_works_same_as_possible_plots_with_expanded_rocky_area() {
        // given
        let mini = Garden {
            width: 5,
            tiles: {
                let mut t = vec![Tile::Plot; 25];
                t[3 + 5 * 3] = Tile::Rock;
                t
            },
        };
        let expanded = Garden {
            width: 25,
            tiles: {
                let mut t = vec![Tile::Plot; 25 * 25];
                for y in 0..5 {
                    for x in 0..5 {
                        t[3 + x * 5 + (3 + y * 5) * 25] = Tile::Rock;
                    }
                }
                t
            },
        };

        for steps in 1..=12 {
            println!("steps: {steps}");
            // when
            let n_infinite = possible_plots_on_infinite_garden(2, 2, &mini, steps as u128);
            let n_expanded = possible_plots(12, 12, &expanded, steps);

            // then
            assert_eq!(n_infinite, Ok(n_expanded));
        }
    }
}
