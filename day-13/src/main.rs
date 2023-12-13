use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let patterns = parse(&content)?;

    let sum = reflection_line_sum(&patterns);
    println!("The weighted sum of reflection line positions is {sum}");

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Tile {
    Ash,
    Rock,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Pattern {
    width: usize,
    tiles: Vec<Tile>,
}

impl Pattern {
    fn height(&self) -> usize {
        self.tiles.len() / self.width
    }
    fn get(&self, x: usize, y: usize) -> Option<Tile> {
        if x < self.width {
            self.tiles.get(x + y * self.width).copied()
        } else {
            None
        }
    }
}

fn reflection_line_sum(patterns: &[Pattern]) -> usize {
    patterns
        .iter()
        .filter_map(|pattern| {
            v_reflection_line(pattern).or_else(|| h_reflection_line(pattern).map(|y| y * 100))
        })
        .sum()
}

fn v_reflection_line(pattern: &Pattern) -> Option<usize> {
    (1..pattern.width).find(|ref_x| v_reflects_at(pattern, *ref_x))
}

fn v_reflects_at(pattern: &Pattern, ref_x: usize) -> bool {
    if ref_x == 0 || ref_x >= pattern.width {
        return false;
    }
    let mut matches = true;
    if ref_x <= pattern.width / 2 {
        for x in 0..ref_x {
            for y in 0..pattern.height() {
                matches = matches && pattern.get(x, y) == pattern.get(ref_x * 2 - x - 1, y);
            }
        }
    } else {
        for x in ref_x..pattern.width {
            for y in 0..pattern.height() {
                matches = matches && pattern.get(x, y) == pattern.get(ref_x * 2 - x - 1, y)
            }
        }
    }
    matches
}

fn h_reflection_line(pattern: &Pattern) -> Option<usize> {
    (1..pattern.height()).find(|ref_y| h_reflects_at(pattern, *ref_y))
}

fn h_reflects_at(pattern: &Pattern, ref_y: usize) -> bool {
    if ref_y == 0 || ref_y >= pattern.height() {
        return false;
    }
    let mut matches = true;
    if ref_y <= pattern.height() / 2 {
        for y in 0..ref_y {
            for x in 0..pattern.width {
                matches = matches && pattern.get(x, y) == pattern.get(x, ref_y * 2 - y - 1);
            }
        }
    } else {
        for y in ref_y..pattern.height() {
            for x in 0..pattern.width {
                matches = matches && pattern.get(x, y) == pattern.get(x, ref_y * 2 - y - 1)
            }
        }
    }
    matches
}

fn parse(input: &str) -> Result<Vec<Pattern>, String> {
    input.split("\n\n").map(parse_pattern).collect()
}

fn parse_pattern(block: &str) -> Result<Pattern, String> {
    // assumption: block is rectangular
    let width = block
        .lines()
        .next()
        .ok_or_else(|| "expected lines in block".to_owned())?
        .len();
    let tiles = block
        .chars()
        .filter_map(|c| match c {
            '.' => Some(Tile::Ash),
            '#' => Some(Tile::Rock),
            _ => None,
        })
        .collect();
    Ok(Pattern { width, tiles })
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#
"#;

    const REV_EXAMPLE: &str = r#".##..##.#
.#.##.#..
#......##
#......##
.#.##.#..
.##..##..
.#.##.#.#

#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#
#...##..#
"#;

    const LARGE_EXAMPLE: &str = r#"##.##.####.##.#..
..#.#..#..##.#..#
.###...###.##.#..
.#..#.###.#....#.
###.##.#...##.##.
#####.##.###..###
#.##..#.#.#.#.#..
#.##..#.#.#.#.#..
#####.##.###..###
###.##.#...##.##.
.#.##.###.#....#.
.###...###.##.#..
..#.#..#..##.#..#
##.##.####.##.#..
##.##.####.##.#..
"#;

    #[test]
    fn reflection_line_sum_works_for_example() {
        // given
        let patterns = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let sum = reflection_line_sum(&patterns);

        // then
        assert_eq!(sum, 405);
    }

    #[test]
    fn reflection_line_sum_works_for_reverse_example() {
        // given
        let patterns = parse(REV_EXAMPLE).expect("expected successful parsing");

        // when
        let sum = reflection_line_sum(&patterns);

        // then
        assert_eq!(sum, 304);
    }

    #[test]
    fn h_reflection_line_works_for_large_example() {
        // given
        let pattern = parse_pattern(LARGE_EXAMPLE).expect("expected successful parsing");

        // when
        let line = h_reflection_line(&pattern);

        // then
        assert_eq!(line, Some(14));
    }
}
