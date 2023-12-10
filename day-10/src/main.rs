use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let (start, edges) = parse(&content)?;

    if let Some(distance) = loop_distance(start, &edges) {
        println!("The point farthest from the starting point is {distance} steps away.");
    } else {
        println!("I got lost in the tubes…");
    }

    Ok(())
}

type Node = (usize, usize);
type Edges = HashMap<Node, (Node, Node)>;

fn parse(input: &str) -> Result<(Node, Edges), String> {
    // assumption: all lines have the same length (and they all end with a newline, including the
    // last line)
    let width = 1 + input
        .lines()
        .next()
        .map(|l| l.len())
        .ok_or_else(|| "no lines in the input".to_string())?;
    let input = input.as_bytes();

    let mut edges: Edges = HashMap::with_capacity(input.len());
    let mut start: Node = (0, 0);

    for (i, tile) in input.iter().copied().enumerate() {
        let x = i % width;
        let y = i / width;
        match tile {
            b'S' => {
                // assumption: there is only one start
                start = (x, y);
                // assumption: just by looking at the neighbours, there are exactly two tiles
                // that connect to S, so there is only one possible tile for S
                let dirs = [
                    connects_north(input, width, x, y),
                    connects_east(input, width, x, y),
                    connects_south(input, width, x, y),
                    connects_west(input, width, x, y),
                ];
                let mut connections = dirs.iter().filter_map(|d| *d);
                let edge = (
                    connections
                        .next()
                        .ok_or_else(|| "no connection for starting point".to_string())?,
                    connections
                        .next()
                        .ok_or_else(|| "no connection for starting point".to_string())?,
                );
                if connections.next().is_some() {
                    return Err("ambiguous connection for starting point".to_string());
                }
                edges.insert((x, y), edge);
            }
            b'|' => {
                if let Some(north) = connects_north(input, width, x, y) {
                    if let Some(south) = connects_south(input, width, x, y) {
                        edges.insert((x, y), (north, south));
                    }
                }
            }
            b'-' => {
                if let Some(east) = connects_east(input, width, x, y) {
                    if let Some(west) = connects_west(input, width, x, y) {
                        edges.insert((x, y), (east, west));
                    }
                }
            }
            b'L' => {
                if let Some(north) = connects_north(input, width, x, y) {
                    if let Some(east) = connects_east(input, width, x, y) {
                        edges.insert((x, y), (north, east));
                    }
                }
            }
            b'J' => {
                if let Some(north) = connects_north(input, width, x, y) {
                    if let Some(west) = connects_west(input, width, x, y) {
                        edges.insert((x, y), (north, west));
                    }
                }
            }
            b'7' => {
                if let Some(south) = connects_south(input, width, x, y) {
                    if let Some(west) = connects_west(input, width, x, y) {
                        edges.insert((x, y), (south, west));
                    }
                }
            }
            b'F' => {
                if let Some(south) = connects_south(input, width, x, y) {
                    if let Some(east) = connects_east(input, width, x, y) {
                        edges.insert((x, y), (south, east));
                    }
                }
            }
            b'.' | b'\n' => (),
            _ => {
                return Err(format!("unknown tile: '{tile}'"));
            }
        }
    }
    Ok((start, edges))
}

fn connects_north(input: &[u8], width: usize, x: usize, y: usize) -> Option<Node> {
    if y == 0 {
        None
    } else if [b'|', b'7', b'F', b'S'].contains(input.get(x + (y - 1) * width)?) {
        Some((x, y - 1))
    } else {
        None
    }
}

fn connects_east(input: &[u8], width: usize, x: usize, y: usize) -> Option<Node> {
    if x + 1 >= width {
        None
    } else if [b'-', b'7', b'J', b'S'].contains(input.get((x + 1) + y * width)?) {
        Some((x + 1, y))
    } else {
        None
    }
}

fn connects_south(input: &[u8], width: usize, x: usize, y: usize) -> Option<Node> {
    if [b'|', b'L', b'J', b'S'].contains(input.get(x + (y + 1) * width)?) {
        Some((x, y + 1))
    } else {
        None
    }
}

fn connects_west(input: &[u8], width: usize, x: usize, y: usize) -> Option<Node> {
    if x == 0 {
        None
    } else if [b'-', b'L', b'F', b'S'].contains(input.get((x - 1) + y * width)?) {
        Some((x - 1, y))
    } else {
        None
    }
}

fn loop_distance(start: Node, edges: &Edges) -> Option<usize> {
    let (mut dir1, mut dir2) = edges.get(&start)?;
    let mut steps = 1;
    let (mut prev1, mut prev2) = (start, start);

    while dir1 != dir2 {
        steps += 1;
        let (next1, next2) = edges.get(&dir1)?;
        if prev1 == *next1 {
            prev1 = dir1;
            dir1 = *next2;
        } else {
            prev1 = dir1;
            dir1 = *next1;
        }
        if dir1 == dir2 {
            break;
        }
        let (next1, next2) = edges.get(&dir2)?;
        if prev2 == *next1 {
            prev2 = dir2;
            dir2 = *next2;
        } else {
            prev2 = dir2;
            dir2 = *next1;
        }
    }
    Some(steps)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"..F7.
.FJ|.
SJ.L7
|F--J
LJ...
"#;

    #[test]
    fn loop_distances_works_for_example() {
        // given
        let (start, edges) = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let result = loop_distance(start, &edges);

        // then
        assert_eq!(result, Some(8));
    }
}
