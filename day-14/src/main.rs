use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let platform = parse(&content)?;

    let tilted_north = tilt_north(platform.clone());
    let tilted_load = load(&tilted_north);
    println!("The load of the platform after it has been tilted north is {tilted_load}");

    let cycled_load = spin_cycles(platform);
    println!("After a billion spin cycles, the load is {cycled_load}");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Tile {
    Floor,
    Cube,
    Round,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Platform {
    tiles: Vec<Tile>,
    width: usize,
}

impl Platform {
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
    fn set(&mut self, x: usize, y: usize, tile: Tile) {
        if x < self.width {
            if let Some(t) = self.tiles.get_mut(x + y * self.width) {
                *t = tile;
            }
        }
    }
}

fn parse(input: &str) -> Result<Platform, String> {
    // assumption: input is rectangular
    let width = input
        .lines()
        .next()
        .ok_or_else(|| "expected lines in input".to_owned())?
        .len();
    let tiles = input
        .chars()
        .filter_map(|c| match c {
            '.' => Some(Tile::Floor),
            '#' => Some(Tile::Cube),
            'O' => Some(Tile::Round),
            _ => None,
        })
        .collect();
    Ok(Platform { tiles, width })
}

fn tilt_north(mut platform: Platform) -> Platform {
    for x in 0..platform.width {
        let mut y = 0;
        while y < platform.height() {
            let tile = platform.get(x, y);
            if tile == Some(Tile::Floor) {
                for round_y in y + 1..platform.height() {
                    match platform.get(x, round_y) {
                        Some(Tile::Cube) => {
                            y = round_y;
                            break;
                        }
                        Some(Tile::Round) => {
                            platform.set(x, y, Tile::Round);
                            platform.set(x, round_y, Tile::Floor);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            y += 1;
        }
    }
    platform
}

fn tilt_west(mut platform: Platform) -> Platform {
    for y in 0..platform.height() {
        let mut x = 0;
        while x < platform.width {
            let tile = platform.get(x, y);
            if tile == Some(Tile::Floor) {
                for round_x in x + 1..platform.width {
                    match platform.get(round_x, y) {
                        Some(Tile::Cube) => {
                            x = round_x;
                            break;
                        }
                        Some(Tile::Round) => {
                            platform.set(x, y, Tile::Round);
                            platform.set(round_x, y, Tile::Floor);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            x += 1;
        }
    }
    platform
}

fn tilt_east(mut platform: Platform) -> Platform {
    for y in 0..platform.height() {
        let mut x = 0;
        while x < platform.width {
            let tile = platform.get(platform.width - x - 1, y);
            if tile == Some(Tile::Floor) {
                for round_x in x + 1..platform.width {
                    match platform.get(platform.width - round_x - 1, y) {
                        Some(Tile::Cube) => {
                            x = round_x;
                            break;
                        }
                        Some(Tile::Round) => {
                            platform.set(platform.width - x - 1, y, Tile::Round);
                            platform.set(platform.width - round_x - 1, y, Tile::Floor);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            x += 1;
        }
    }
    platform
}

fn tilt_south(mut platform: Platform) -> Platform {
    for x in 0..platform.width {
        let mut y = 0;
        while y < platform.height() {
            let tile = platform.get(x, platform.height() - 1 - y);
            if tile == Some(Tile::Floor) {
                for round_y in y + 1..platform.height() {
                    match platform.get(x, platform.height() - 1 - round_y) {
                        Some(Tile::Cube) => {
                            y = round_y;
                            break;
                        }
                        Some(Tile::Round) => {
                            platform.set(x, platform.height() - 1 - y, Tile::Round);
                            platform.set(x, platform.height() - 1 - round_y, Tile::Floor);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            y += 1;
        }
    }
    platform
}

const RUN_LENGTH: usize = 1_000_000_000;
fn spin_cycles(mut platform: Platform) -> usize {
    let mut seen: HashMap<Platform, usize> = HashMap::with_capacity(1024);
    let mut i: usize = 0;
    while i < RUN_LENGTH {
        platform = tilt_north(platform);
        platform = tilt_west(platform);
        platform = tilt_south(platform);
        platform = tilt_east(platform);
        if let Some(cycle_start) = seen.get(&platform) {
            let cycle_length = i - cycle_start;
            i = cycle_start + ((RUN_LENGTH - cycle_start) / cycle_length) * cycle_length + 1;
            break;
        }
        seen.insert(platform.clone(), i);
        i += 1;
    }
    for _ in i..RUN_LENGTH {
        platform = tilt_north(platform);
        platform = tilt_west(platform);
        platform = tilt_south(platform);
        platform = tilt_east(platform);
    }
    load(&platform)
}

fn load(platform: &Platform) -> usize {
    let height = platform.height();
    platform
        .tiles
        .iter()
        .enumerate()
        .filter(|(_, tile)| **tile == Tile::Round)
        .map(|(i, _)| height - (i / platform.width))
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....
"#;

    const EXAMPLE_TILTED: &str = r#"OOOO.#.O..
OO..#....#
OO..O##..O
O..#.OO...
........#.
..#....#.#
..O..#.O.O
..O.......
#....###..
#....#....
"#;

    const EXAMPLE_TILTED_WEST: &str = r#"O....#....
OOO.#....#
.....##...
OO.#OO....
OO......#.
O.#O...#.#
O....#OO..
O.........
#....###..
#OO..#....
"#;

    #[test]
    fn tilt_north_works_for_example() {
        // given
        let platform = parse(EXAMPLE).expect("expected successful parsing");
        let platform_tilted = parse(EXAMPLE_TILTED).expect("expected successful parsing");

        // when
        let result = tilt_north(platform);

        // then
        assert_eq!(result, platform_tilted);
    }

    #[test]
    fn load_works_for_example() {
        // given
        let platform = parse(EXAMPLE_TILTED).expect("expected successful parsing");

        // when
        let result = load(&platform);

        // then
        assert_eq!(result, 136);
    }

    #[test]
    fn round_trip_tilt_north_and_south_works_for_example() {
        // given
        let platform = parse(EXAMPLE).expect("expected successful parsing");
        let platform_tilted = parse(EXAMPLE_TILTED).expect("expected successful parsing");

        // when
        let platform = tilt_south(platform);
        let result = tilt_north(platform);

        // then
        assert_eq!(result, platform_tilted);
    }

    #[test]
    fn tilt_west_works_for_example() {
        // given
        let platform = parse(EXAMPLE).expect("expected successful parsing");
        let platform_tilted = parse(EXAMPLE_TILTED_WEST).expect("expected successful parsing");

        // when
        let result = tilt_west(platform);

        // then
        assert_eq!(result, platform_tilted);
    }

    #[test]
    fn round_trop_tilt_west_and_east_works_for_example() {
        // given
        let platform = parse(EXAMPLE).expect("expected successful parsing");
        let platform_tilted = parse(EXAMPLE_TILTED_WEST).expect("expected successful parsing");

        // when
        let tilted_east = tilt_east(platform);
        let result = tilt_west(tilted_east);

        // then
        assert_eq!(result, platform_tilted);
    }

    #[test]
    fn spin_cycles_works_for_example() {
        // given
        let platform = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let load = spin_cycles(platform);

        // then
        assert_eq!(load, 64);
    }
}
