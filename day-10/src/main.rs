use std::collections::{HashMap, HashSet};
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

    if let Some(area) = enclosed_area(start, &edges) {
        println!("{area} tiles are enclosed by the loop");
    } else {
        println!("I got lost between the tubes…");
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

fn filter_loop_edges_in_double_space(start: Node, edges: &Edges) -> Option<HashSet<Node>> {
    let (mut current, _) = edges.get(&start)?;
    let mut loop_edges: HashSet<Node> = HashSet::with_capacity(edges.len() * 2);
    let mut prev = start;
    loop_edges.insert((2 + start.0 * 2, 2 + start.1 * 2));
    loop_edges.insert(odd_in_between(start, current));
    while current != start {
        let (dir1, dir2) = edges.get(&current)?;
        loop_edges.insert((2 + current.0 * 2, 2 + current.1 * 2));
        if *dir1 == prev {
            loop_edges.insert(odd_in_between(current, *dir2));
            prev = current;
            current = *dir2;
        } else {
            loop_edges.insert(odd_in_between(current, *dir1));
            prev = current;
            current = *dir1;
        }
    }
    Some(loop_edges)
}

fn odd_in_between((ax, ay): Node, (bx, by): Node) -> Node {
    // assumption: a and b are neighbours (so ax == bx or ay == by)
    if ax == bx {
        (ax * 2 + 2, ay.min(by) * 2 + 3)
    } else {
        (ax.min(bx) * 2 + 3, ay * 2 + 2)
    }
}

fn enclosed_area(start: Node, edges: &Edges) -> Option<usize> {
    // idea: double all positions (to make space between pipes), then flood the outside area, then discard
    // odd positions, then subtract it from the total area
    let double_edges = filter_loop_edges_in_double_space(start, edges)?;

    let width = double_edges.iter().map(|(x, _)| x).max()? + 2;
    let height = double_edges.iter().map(|(_, y)| y).max()? + 2;

    let mut visited: HashSet<Node> = HashSet::with_capacity(width * height);

    let mut queue: Vec<Node> = Vec::with_capacity(width * height);
    queue.push((0, 0));

    while let Some((x, y)) = queue.pop() {
        visited.insert((x, y));
        if x != 0 && !double_edges.contains(&(x - 1, y)) && !visited.contains(&(x - 1, y)) {
            queue.push((x - 1, y));
        }
        if y != 0 && !double_edges.contains(&(x, y - 1)) && !visited.contains(&(x, y - 1)) {
            queue.push((x, y - 1));
        }
        if x + 1 < width && !double_edges.contains(&(x + 1, y)) && !visited.contains(&(x + 1, y)) {
            queue.push((x + 1, y));
        }
        if y + 1 < height && !double_edges.contains(&(x, y + 1)) && !visited.contains(&(x, y + 1)) {
            queue.push((x, y + 1));
        }
    }

    Some(
        (width / 2) * (height / 2)
            - visited
                .iter()
                .filter(|(x, y)| x % 2 == 0 && y % 2 == 0)
                .count()
            - double_edges
                .iter()
                .filter(|(x, y)| x % 2 == 0 && y % 2 == 0)
                .count(),
    )
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

    const ENCLOSED_EX1: &str = r#"..........
.S------7.
.|F----7|.
.||....||.
.||....||.
.|L-7F-J|.
.|..||..|.
.L--JL--J.
..........
"#;

    const ENCLOSED_EX2: &str = r#".F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...
"#;

    const ENCLOSED_EX3: &str = r#"FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L
"#;

    #[test]
    fn enclosed_area_works_for_example1() {
        // given
        let (start, edges) = parse(ENCLOSED_EX1).expect("expected successful parsing");

        // when
        let area = enclosed_area(start, &edges);

        assert_eq!(area, Some(4));
    }

    #[test]
    fn enclosed_area_works_for_example2() {
        // given
        let (start, edges) = parse(ENCLOSED_EX2).expect("expected successful parsing");

        // when
        let area = enclosed_area(start, &edges);

        assert_eq!(area, Some(8));
    }

    #[test]
    fn enclosed_area_works_for_example3() {
        // given
        let (start, edges) = parse(ENCLOSED_EX3).expect("expected successful parsing");

        // when
        let area = enclosed_area(start, &edges);

        assert_eq!(area, Some(10));
    }
}
