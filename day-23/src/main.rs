use std::collections::{BTreeSet, HashMap, HashSet};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let map = parse(&content)?;

    if let Some(path_length) = longest_path(&map) {
        println!("The longest path is {path_length}");
    } else {
        println!("I searched for the longest path, but I got lost…");
    }

    if let Some(path_length) = longest_path_ignore_slopes(&map) {
        println!("If you ignore slopes, the longest path is {path_length}");
    } else {
        println!("I tried to climb up slopes, but I got lost…");
    }

    Ok(())
}

type Point = (usize, usize);

fn longest_path_ignore_slopes(map: &Map) -> Option<u32> {
    // Wikipedia says this problem is NP hard, but the number of branches should be low, so this is
    // doable I guess?
    // Note from after running it: Yes, it works, but it takes a lot of time (~50s with --release
    // on my machine) and an awful amount of memory
    // But hey, it works, so no need to spend a lot of time looking for patterns in the input to
    // optimize the solution

    // I don't want to do this branch exploration again, but I can just take the directed graph and
    // transform it
    let edges = derive_directed(map)?;
    let edges: HashMap<Point, Vec<(Point, u32)>> = edges
        .iter()
        .flat_map(|((from, to), distance)| [(*from, (*to, *distance)), (*to, (*from, *distance))])
        .fold(
            HashMap::with_capacity(edges.len() * 2 + 2),
            |mut agg, (from, to)| {
                agg.entry(from)
                    .or_insert_with(|| Vec::with_capacity(4))
                    .push(to);
                agg
            },
        );

    let mut visited: HashMap<(Point, BTreeSet<Point>), u32> = HashMap::with_capacity(edges.len());
    let mut queue: Vec<(Point, BTreeSet<Point>, u32)> = Vec::with_capacity(edges.len());
    queue.push(((1, 0), BTreeSet::new(), 0));

    while let Some((edge, ancestors, distance)) = queue.pop() {
        let child_ancestors = {
            let mut a = ancestors.clone();
            a.insert(edge);
            a
        };
        let seen_distance = visited.entry((edge, ancestors)).or_insert(distance);
        if *seen_distance > distance {
            continue;
        }
        *seen_distance = distance;

        for (child, child_distance) in edges.get(&edge).into_iter().flatten() {
            if !child_ancestors.contains(child) {
                queue.push((*child, child_ancestors.clone(), distance + child_distance));
            }
        }
    }

    visited
        .iter()
        .filter(|((edge, _), _)| *edge == (map.width - 2, map.height() - 1))
        .map(|(_, distance)| *distance)
        .max()
}

fn longest_path(map: &Map) -> Option<u32> {
    // fun thing is: I have so many places here where I inefficiently loop over data instead of
    // using approriate data structures, I have lots of potential for optimization should the need
    // arise.
    let edges = derive_directed(map)?;
    let topsort_vertices = sort_topological(&edges)?;

    let mut longest_paths: HashMap<Point, u32> = HashMap::with_capacity(topsort_vertices.len());

    for v in topsort_vertices {
        // this, for example, is one of the optimizable things mentioned above
        let dist: u32 = edges
            .iter()
            .filter(|((_, to), _)| *to == v)
            .map(|((from, _), dist)| {
                longest_paths
                    .get(from)
                    .expect("vertices should be topologically sorted!")
                    + dist
            })
            .max()
            .unwrap_or(0);
        longest_paths.insert(v, dist);
    }

    longest_paths
        .get(&(map.width - 2, map.height() - 1))
        .copied()
}

fn sort_topological(edges: &HashMap<(Point, Point), u32>) -> Option<Vec<Point>> {
    let mut predecessors: HashMap<Point, Vec<Point>> = HashMap::with_capacity(edges.len() * 4);
    for (from, to) in edges.keys() {
        predecessors
            .entry(*to)
            .or_insert_with(|| Vec::with_capacity(4))
            .push(*from);
        predecessors
            .entry(*from)
            .or_insert_with(|| Vec::with_capacity(4));
    }
    let mut sorted: Vec<Point> = Vec::with_capacity(edges.len() + 1);

    // this kind of reminds me of yesterday's puzzle
    let mut removed_something = true;
    let mut remove_from_predecessors: Vec<Point> = Vec::with_capacity(edges.len());
    while removed_something {
        removed_something = false;
        for (node, preds) in &predecessors {
            if preds.is_empty() {
                sorted.push(*node);
                removed_something = true;
                remove_from_predecessors.push(*node);
            }
        }

        // this may be slower than expected, because according to the docs:
        // “this operation takes O(capacity) time instead of O(len) because it internally visits
        // empty buckets too.”
        // should be fine for our use case, though
        predecessors.retain(|_, preds| !preds.is_empty());

        for pred in remove_from_predecessors.drain(..) {
            for l_preds in predecessors.values_mut() {
                if let Some(i) = l_preds.iter().position(|edge| *edge == pred) {
                    l_preds.swap_remove(i);
                }
            }
        }
    }

    if predecessors.is_empty() {
        Some(sorted)
    } else {
        None
    }
}

fn derive_directed(map: &Map) -> Option<HashMap<(Point, Point), u32>> {
    // idea: solving the longest path problem is generally NP-hard
    // …unless the graph is a directed acyclic graph (DAG)
    // Fortunately, both the example input and my input seem to be DAGs, because there
    // are slopes around every branch.
    // This function does not check if the resulting directed graph is acyclic yet
    let mut queue: Vec<(Point, Point, u32)> = Vec::with_capacity(map.tiles.len());
    let mut seen: HashSet<(Point, Point)> = HashSet::with_capacity(map.tiles.len());
    let mut dag_edges: HashMap<(Point, Point), u32> = HashMap::with_capacity(map.width);

    queue.push(((1, 0), (1, 0), 0));
    while let Some((node, last_branch, distance)) = queue.pop() {
        if seen.contains(&(node, last_branch)) {
            continue;
        }
        seen.insert((node, last_branch));
        let neighbours = find_neighbours(map, node);

        let slope_neighbours = neighbours
            .iter()
            .filter(|(_, tile)| tile.is_slope())
            .count();
        let path_neighbours = neighbours
            .iter()
            .filter(|(_, tile)| *tile == Tile::Path)
            .count();

        // branches must only have slopes as neighbours
        if path_neighbours > 2 || slope_neighbours > 1 && path_neighbours > 1 {
            return None;
        }

        let distance = if (slope_neighbours > 1 || slope_neighbours == 0 && path_neighbours == 1)
            && last_branch != node
        {
            dag_edges.insert((last_branch, node), distance);
            0
        } else {
            distance
        };

        let last_branch = if slope_neighbours > 1 {
            node
        } else {
            last_branch
        };

        for neighbour in neighbours.iter().filter_map(|(n, _)| *n) {
            queue.push((neighbour, last_branch, distance + 1));
        }
    }
    Some(dag_edges)
}

fn find_neighbours(map: &Map, pos: Point) -> [(Option<Point>, Tile); 4] {
    let current = map.get(pos);
    let (x, y) = pos;
    let up = if y > 0 {
        if let Some(neighbour) = map.get((x, y - 1)) {
            match current {
                Some(Tile::Path) | Some(Tile::SlopeUp) => match neighbour {
                    Tile::Forest | Tile::SlopeDown => (None, neighbour),
                    tile => (Some((x, y - 1)), tile),
                },
                Some(_) => (None, neighbour),
                None => (None, Tile::Forest),
            }
        } else {
            (None, Tile::Forest)
        }
    } else {
        (None, Tile::Forest)
    };
    let left = if x > 0 {
        if let Some(neighbour) = map.get((x - 1, y)) {
            match current {
                Some(Tile::Path) | Some(Tile::SlopeLeft) => match neighbour {
                    Tile::Forest | Tile::SlopeRight => (None, neighbour),
                    tile => (Some((x - 1, y)), tile),
                },
                Some(_) => (None, neighbour),
                None => (None, Tile::Forest),
            }
        } else {
            (None, Tile::Forest)
        }
    } else {
        (None, Tile::Forest)
    };
    let down = if let Some(neighbour) = map.get((x, y + 1)) {
        match current {
            Some(Tile::Path) | Some(Tile::SlopeDown) => match neighbour {
                Tile::Forest | Tile::SlopeUp => (None, neighbour),
                tile => (Some((x, y + 1)), tile),
            },
            Some(_) => (None, neighbour),
            None => (None, Tile::Forest),
        }
    } else {
        (None, Tile::Forest)
    };
    let right = if let Some(neighbour) = map.get((x + 1, y)) {
        match current {
            Some(Tile::Path) | Some(Tile::SlopeRight) => match neighbour {
                Tile::Forest | Tile::SlopeLeft => (None, neighbour),
                tile => (Some((x + 1, y)), tile),
            },
            Some(_) => (None, neighbour),
            None => (None, Tile::Forest),
        }
    } else {
        (None, Tile::Forest)
    };
    [up, right, down, left]
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Tile {
    Forest,
    Path,
    SlopeUp,
    SlopeRight,
    SlopeDown,
    SlopeLeft,
}

impl Tile {
    fn is_slope(self) -> bool {
        !matches!(self, Tile::Forest | Tile::Path)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Map {
    width: usize,
    tiles: Vec<Tile>,
}

impl Map {
    fn height(&self) -> usize {
        self.tiles.len() / self.width
    }

    fn get(&self, (x, y): Point) -> Option<Tile> {
        if x < self.width {
            self.tiles.get(x + y * self.width).copied()
        } else {
            None
        }
    }
}

fn parse(input: &str) -> Result<Map, String> {
    // assumption: input is rectangular
    let width = input
        .lines()
        .next()
        .ok_or_else(|| "expected lines in the input".to_string())?
        .len();

    if width == 0 {
        return Err("expected non-empty first line".to_string());
    }

    let tiles: Vec<Tile> = input
        .chars()
        .filter(|c| *c != '\n')
        .map(|c| match c {
            '.' => Ok(Tile::Path),
            '#' => Ok(Tile::Forest),
            '^' => Ok(Tile::SlopeUp),
            '>' => Ok(Tile::SlopeRight),
            'v' => Ok(Tile::SlopeDown),
            '<' => Ok(Tile::SlopeLeft),
            _ => Err(format!("unkown tile: '{c}'")),
        })
        .collect::<Result<_, _>>()?;

    Ok(Map { width, tiles })
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"#.#####################
#.......#########...###
#######.#########.#.###
###.....#.>.>.###.#.###
###v#####.#v#.###.#.###
###.>...#.#.#.....#...#
###v###.#.#.#########.#
###...#.#.#.......#...#
#####.#.#.#######.#.###
#.....#.#.#.......#...#
#.#####.#.#.#########v#
#.#...#...#...###...>.#
#.#.#v#######v###.###v#
#...#.>.#...>.>.#.###.#
#####v#.#.###v#.#.###.#
#.....#...#...#.#.#...#
#.#########.###.#.#.###
#...###...#...#...#.###
###.###.#.###v#####v###
#...#...#.#.>.>.#.>.###
#.###.###.#.###.#.#v###
#.....###...###...#...#
#####################.#
"#;

    #[test]
    fn longest_path_works_for_example() {
        // given
        let map = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let length = longest_path(&map);

        // then
        assert_eq!(length, Some(94));
    }

    #[test]
    fn longest_path_ignore_slopes_works_for_example() {
        // given
        let map = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let length = longest_path_ignore_slopes(&map);

        // then
        assert_eq!(length, Some(154));
    }
}
