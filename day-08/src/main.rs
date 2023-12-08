use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let (dirs, nodes) = parse(&content)?;

    let steps = find_path_length(&dirs, &nodes)?;
    println!("{steps} steps are required to reach 'ZZZ'");

    let ghost_steps = find_ghost_path_length(&dirs, &nodes)?;
    println!("{ghost_steps} ghost steps are required to reach an end state");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Dir {
    Left,
    Right,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Node<'a> {
    from: &'a str,
    left: &'a str,
    right: &'a str,
}

fn parse_node(line: &str) -> Result<Node, String> {
    let (from, to) = line
        .split_once(" = ")
        .ok_or_else(|| format!("unable to split line '{line}'"))?;
    let (left, right) = to
        .strip_prefix('(')
        .and_then(|t| t.strip_suffix(')'))
        .and_then(|t| t.split_once(", "))
        .ok_or_else(|| format!("invalid format for RHS in line '{line}'"))?;

    Ok(Node { from, left, right })
}

fn parse(input: &str) -> Result<(Vec<Dir>, HashMap<&str, Node>), String> {
    let mut lines = input.lines();
    let dirs = lines
        .next()
        .ok_or_else(|| "no first line in input".to_string())?;

    let dirs: Vec<Dir> = dirs
        .chars()
        .map(|c| match c {
            'L' => Ok(Dir::Left),
            'R' => Ok(Dir::Right),
            _ => Err(format!("unknown direction: '{c}'")),
        })
        .collect::<Result<Vec<Dir>, String>>()?;

    if lines.next() != Some("") {
        return Err("missing blank line between directions and nodes".to_string());
    }

    let nodes = lines
        .map(parse_node)
        .map(|nr| {
            let node = nr?;
            Ok((node.from, node))
        })
        .collect::<Result<HashMap<&str, Node>, String>>()?;

    Ok((dirs, nodes))
}

fn find_path_length(dirs: &[Dir], nodes: &HashMap<&str, Node>) -> Result<usize, String> {
    let mut steps: usize = 0;
    let mut current_node: &str = "AAA";
    while current_node != "ZZZ" {
        let node = nodes
            .get(current_node)
            .ok_or_else(|| format!("unable to find directions for node '{current_node}'"))?;
        current_node = match dirs[steps % dirs.len()] {
            Dir::Left => node.left,
            Dir::Right => node.right,
        };
        steps += 1
    }
    Ok(steps)
}

fn find_any_end_path_length(
    dirs: &[Dir],
    nodes: &HashMap<&str, Node>,
    start_node: &str,
) -> Result<u64, String> {
    let mut steps: u64 = 0;
    let mut current_node: &str = start_node;
    while !current_node.ends_with('Z') {
        let node = nodes
            .get(current_node)
            .ok_or_else(|| format!("unable to find directions for node '{current_node}'"))?;
        current_node = match dirs[steps as usize % dirs.len()] {
            Dir::Left => node.left,
            Dir::Right => node.right,
        };
        steps += 1
    }
    Ok(steps)
}

fn find_ghost_path_length(dirs: &[Dir], nodes: &HashMap<&str, Node>) -> Result<u64, String> {
    // note: this only works if there is only _one_ cycle length in the paths
    let paths_to_end: Vec<u64> = nodes
        .keys()
        .filter(|node| node.ends_with('A'))
        .map(|node| find_any_end_path_length(dirs, nodes, node))
        .collect::<Result<Vec<u64>, String>>()?;
    paths_to_end
        .iter()
        .copied()
        .reduce(lcm)
        .ok_or_else(|| "no starting nodes found".to_string())
}

fn gcd(mut m: u64, mut n: u64) -> u64 {
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    while n != 0 {
        let h = m % n;
        m = n;
        n = h;
    }
    m
}

fn lcm(m: u64, n: u64) -> u64 {
    (m * n) / gcd(m, n)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_1: &str = r#"RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, DDD)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)
"#;

    const EXAMPLE_2: &str = r#"LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)
"#;

    #[test]
    fn find_path_length_works_for_example_1() {
        // given
        let (dirs, nodes) = parse(EXAMPLE_1).expect("expected successful parsing");

        // when
        let steps = find_path_length(&dirs, &nodes);

        // then
        assert_eq!(steps, Ok(2));
    }

    #[test]
    fn find_path_length_works_for_example_2() {
        // given
        let (dirs, nodes) = parse(EXAMPLE_2).expect("expected successful parsing");

        // when
        let steps = find_path_length(&dirs, &nodes);

        // then
        assert_eq!(steps, Ok(6));
    }
    const EXAMPLE_3: &str = r#"LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)
"#;
    #[test]
    fn find_ghost_path_length_works_for_example() {
        // given
        let (dirs, nodes) = parse(EXAMPLE_3).expect("expected successful parsing");

        // when
        let steps = find_ghost_path_length(&dirs, &nodes);

        // then
        assert_eq!(steps, Ok(6));
    }
}
