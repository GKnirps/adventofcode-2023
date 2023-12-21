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

    Ok(())
}

fn possible_plots(start_x: usize, start_y: usize, garden: &Garden, steps: usize) -> usize {
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
        .values()
        .filter(|s| **s <= steps && *s % 2 == 0)
        .count()
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
}
