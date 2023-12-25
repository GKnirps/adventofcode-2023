use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let connections = parse(&content)?;

    if let Some(n) = find_cut_3(&connections) {
        println!("The product of the size of the two groups is {n}");
    } else {
        println!(
            "Could not find a way to split the components by removing only three connections."
        );
    }

    Ok(())
}

fn find_cut_3(connections: &HashMap<&str, Vec<&str>>) -> Option<usize> {
    let from = connections.keys().next()?;
    for to in connections.keys().filter(|to| to != &from) {
        let (cut, from_size) = min_cut(from, to, connections);
        if cut <= 3 {
            return Some(from_size * (connections.len() - from_size));
        }
    }
    None
}

// search for a min cut between from and to
// return min cut and size of the part tha from belongs to
fn min_cut(from: &str, to: &str, connections: &HashMap<&str, Vec<&str>>) -> (u32, usize) {
    // simplified Ford & Fulkerson, since we have unit capacities
    // I did not check this for correctness, so it may not work for all inputs
    let mut used_connections: HashSet<(&str, &str)> = HashSet::with_capacity(connections.len() * 6);

    let mut flow: u32 = 0;
    let mut found_path = true;
    let mut queue: VecDeque<(&str, Option<&str>)> = VecDeque::with_capacity(connections.len());
    let mut visited: HashMap<&str, Option<&str>> = HashMap::with_capacity(connections.len());
    while found_path {
        found_path = false;
        queue.clear();
        visited.clear();
        queue.push_back((from, None));

        while let Some((v, pred)) = queue.pop_front() {
            if visited.contains_key(&v) {
                continue;
            }
            visited.insert(v, pred);
            if v == to {
                break;
            }
            for neighbour in connections.get(&v).iter().flat_map(|c| c.iter()) {
                if !used_connections.contains(&(v, neighbour)) {
                    queue.push_back((neighbour, Some(v)));
                }
            }
        }

        if visited.contains_key(&to) {
            found_path = true;
            flow += 1;
        }
        let mut backtrack = to;
        while let Some(pred) = visited.get(&backtrack).copied().flatten() {
            used_connections.insert((pred, backtrack));
            used_connections.insert((backtrack, pred));
            backtrack = pred;
        }
    }
    (flow, visited.len())
}

fn parse(input: &str) -> Result<HashMap<&str, Vec<&str>>, String> {
    let mut connections = HashMap::with_capacity(2048);

    for line in input.lines() {
        let (from, to) = line
            .split_once(": ")
            .ok_or_else(|| format!("unable to split line '{line}'"))?;
        for to in to.split_whitespace() {
            connections
                .entry(from)
                .or_insert(Vec::with_capacity(8))
                .push(to);
            connections
                .entry(to)
                .or_insert(Vec::with_capacity(8))
                .push(from);
        }
    }

    Ok(connections)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"jqt: rhn xhk nvd
rsh: frs pzl lsr
xhk: hfx
cmg: qnr nvd lhk bvb
rhn: xhk bvb hfx
bvb: xhk hfx
pzl: lsr hfx nvd
qnr: nvd
ntq: jqt hfx bvb xhk
nvd: lhk
lsr: lhk
rzs: qnr cmg lsr rsh
frs: qnr lhk lsr
"#;

    #[test]
    fn find_cut_3_works_for_example() {
        // given
        let connections = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let n = find_cut_3(&connections);

        // then
        assert_eq!(n, Some(54));
    }
}
