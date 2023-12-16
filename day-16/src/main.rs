use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let contraption = parse(&content)?;

    let e = energized_tiles(&contraption, 0, 0, Dir::East);
    println!("{e} tiles are energized");

    let e_max = maximize_energized(&contraption);
    println!("With an optimal starting point, {e_max} tiles are energized");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Tile {
    Empty,
    MirrorSW,
    MirrorNW,
    SplitterH,
    SplitterV,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Dir {
    North,
    East,
    South,
    West,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Contraption {
    tiles: Vec<Tile>,
    width: usize,
}

impl Contraption {
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

fn parse(input: &str) -> Result<Contraption, String> {
    // assumption: all lines have the same length
    let width = input
        .lines()
        .next()
        .ok_or_else(|| "input has no lines".to_string())?
        .len();

    let tiles = input
        .chars()
        .filter(|c| *c != '\n')
        .map(|c| match c {
            '.' => Ok(Tile::Empty),
            '\\' => Ok(Tile::MirrorSW),
            '/' => Ok(Tile::MirrorNW),
            '|' => Ok(Tile::SplitterH),
            '-' => Ok(Tile::SplitterV),
            _ => Err(format!("unknown tile: '{c}'")),
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Contraption { tiles, width })
}

fn energized_tiles(
    contraption: &Contraption,
    start_x: usize,
    start_y: usize,
    start_dir: Dir,
) -> usize {
    let mut queue: Vec<(usize, usize, Dir)> = Vec::with_capacity(contraption.tiles.len());
    let mut visited: HashSet<(usize, usize, Dir)> = HashSet::with_capacity(contraption.tiles.len());
    let mut energized: HashSet<(usize, usize)> = HashSet::with_capacity(contraption.tiles.len());
    if contraption.tiles.is_empty() {
        return 0;
    }

    queue.push((start_x, start_y, start_dir));
    while let Some((x, y, dir)) = queue.pop() {
        if visited.contains(&(x, y, dir)) {
            continue;
        }
        if let Some(tile) = contraption.get(x, y) {
            visited.insert((x, y, dir));
            energized.insert((x, y));
            match dir {
                Dir::North => match tile {
                    Tile::Empty | Tile::SplitterH => {
                        if y > 0 {
                            queue.push((x, y - 1, Dir::North));
                        }
                    }
                    Tile::MirrorSW => {
                        if x > 0 {
                            queue.push((x - 1, y, Dir::West));
                        }
                    }
                    Tile::MirrorNW => {
                        if x + 1 < contraption.width {
                            queue.push((x + 1, y, Dir::East));
                        }
                    }
                    Tile::SplitterV => {
                        if x > 0 {
                            queue.push((x - 1, y, Dir::West));
                        }
                        if x + 1 < contraption.width {
                            queue.push((x + 1, y, Dir::East));
                        }
                    }
                },
                Dir::South => match tile {
                    Tile::Empty | Tile::SplitterH => {
                        if y + 1 < contraption.height() {
                            queue.push((x, y + 1, Dir::South));
                        }
                    }
                    Tile::MirrorSW => {
                        if x + 1 < contraption.width {
                            queue.push((x + 1, y, Dir::East));
                        }
                    }
                    Tile::MirrorNW => {
                        if x > 0 {
                            queue.push((x - 1, y, Dir::West));
                        }
                    }
                    Tile::SplitterV => {
                        if x > 0 {
                            queue.push((x - 1, y, Dir::West));
                        }
                        if x + 1 < contraption.width {
                            queue.push((x + 1, y, Dir::East));
                        }
                    }
                },
                Dir::West => match tile {
                    Tile::Empty | Tile::SplitterV => {
                        if x > 0 {
                            queue.push((x - 1, y, Dir::West));
                        }
                    }
                    Tile::MirrorSW => {
                        if y > 0 {
                            queue.push((x, y - 1, Dir::North));
                        }
                    }
                    Tile::MirrorNW => {
                        if y + 1 < contraption.height() {
                            queue.push((x, y + 1, Dir::South));
                        }
                    }
                    Tile::SplitterH => {
                        if y > 0 {
                            queue.push((x, y - 1, Dir::North));
                        }
                        if y + 1 < contraption.height() {
                            queue.push((x, y + 1, Dir::South));
                        }
                    }
                },
                Dir::East => match tile {
                    Tile::Empty | Tile::SplitterV => {
                        if x + 1 < contraption.width {
                            queue.push((x + 1, y, Dir::East));
                        }
                    }
                    Tile::MirrorSW => {
                        if y + 1 < contraption.height() {
                            queue.push((x, y + 1, Dir::South));
                        }
                    }
                    Tile::MirrorNW => {
                        if y > 0 {
                            queue.push((x, y - 1, Dir::North));
                        }
                    }
                    Tile::SplitterH => {
                        if y > 0 {
                            queue.push((x, y - 1, Dir::North));
                        }
                        if y + 1 < contraption.height() {
                            queue.push((x, y + 1, Dir::South));
                        }
                    }
                },
            }
        }
    }

    energized.len()
}

fn maximize_energized(contraption: &Contraption) -> usize {
    (0..contraption.width)
        .flat_map(|x| {
            [
                energized_tiles(contraption, x, 0, Dir::South),
                energized_tiles(contraption, x, contraption.height() - 1, Dir::North),
            ]
        })
        .chain((0..contraption.height()).flat_map(|y| {
            [
                energized_tiles(contraption, 0, y, Dir::East),
                energized_tiles(contraption, contraption.width - 1, y, Dir::West),
            ]
        }))
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#".|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|....
"#;

    #[test]
    fn energized_tiles_works_for_example() {
        // given
        let contraption = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let e = energized_tiles(&contraption, 0, 0, Dir::East);

        // then
        assert_eq!(e, 46);
    }

    #[test]
    fn maximize_energized_works_for_example() {
        // given
        let contraption = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let e = maximize_energized(&contraption);

        // then
        assert_eq!(e, 51);
    }
}
