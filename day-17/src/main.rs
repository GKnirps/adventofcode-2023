use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let heat_loss_map = parse(&content)?;

    if let Some(heat_loss) = shortest_path(&heat_loss_map, 0, 3) {
        println!("The path with minimal heat loss has a heat loss of {heat_loss}");
    } else {
        println!("There is no path to the goal");
    }

    if let Some(heat_loss) = shortest_path(&heat_loss_map, 4, 10) {
        println!("The ultra-crucible path with minimal heat loss has a heat loss of {heat_loss}");
    } else {
        println!(
            "There is _no way_ to steer this frilkin' ultra crucible through this frikkin' city"
        );
    }

    Ok(())
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct HeatLossMap {
    width: usize,
    tiles: Vec<u32>,
}

impl HeatLossMap {
    fn height(&self) -> usize {
        self.tiles.len() / self.width
    }

    fn get(&self, x: usize, y: usize) -> Option<u32> {
        if x >= self.width {
            None
        } else {
            self.tiles.get(x + y * self.width).copied()
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Dir {
    North,
    East,
    South,
    West,
}

type Node = (usize, usize, u32, Dir);

#[derive(Copy, Clone, Eq, Debug)]
struct HeapItem {
    heat_loss: u32,
    node: Node,
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // by default binary heap puts the largest items on top, so we reverse the comparing
        // function here
        other.heat_loss.cmp(&self.heat_loss)
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

fn shortest_path(heat_loss_map: &HeatLossMap, min_straight: u32, max_straight: u32) -> Option<u32> {
    let mut heap: BinaryHeap<HeapItem> =
        BinaryHeap::with_capacity(heat_loss_map.tiles.len() * 4 * max_straight as usize);
    // since we have a minimum steps to go in a straight line, we have to consider both directions
    // as starting directions (there is no given starting direction in the puzzle
    heap.push(HeapItem {
        heat_loss: 0,
        node: (0, 0, 0, Dir::South),
    });
    heap.push(HeapItem {
        heat_loss: 0,
        node: (0, 0, 0, Dir::East),
    });

    let mut visited: HashSet<Node> =
        HashSet::with_capacity(heat_loss_map.tiles.len() * 4 * max_straight as usize);

    while let Some(heap_item) = heap.pop() {
        if heap_item.node.0 == heat_loss_map.width - 1
            && heap_item.node.1 == heat_loss_map.height() - 1
            && heap_item.node.2 >= min_straight
        {
            return Some(heap_item.heat_loss);
        }
        if visited.contains(&heap_item.node) {
            continue;
        }
        visited.insert(heap_item.node);

        let HeapItem {
            heat_loss,
            node: (x, y, steps, dir),
        } = heap_item;
        match dir {
            Dir::North => {
                if y > 0 && steps < max_straight {
                    if let Some(neighbour_heat_loss) = heat_loss_map.get(x, y - 1) {
                        heap.push(HeapItem {
                            heat_loss: heat_loss + neighbour_heat_loss,
                            node: (x, y - 1, steps + 1, Dir::North),
                        });
                    }
                }
                if steps >= min_straight {
                    if x > 0 {
                        if let Some(neighbour_heat_loss) = heat_loss_map.get(x - 1, y) {
                            heap.push(HeapItem {
                                heat_loss: heat_loss + neighbour_heat_loss,
                                node: (x - 1, y, 1, Dir::West),
                            });
                        }
                    }
                    if let Some(neighbour_heat_loss) = heat_loss_map.get(x + 1, y) {
                        heap.push(HeapItem {
                            heat_loss: heat_loss + neighbour_heat_loss,
                            node: (x + 1, y, 1, Dir::East),
                        });
                    }
                }
            }
            Dir::South => {
                if steps < max_straight {
                    if let Some(neighbour_heat_loss) = heat_loss_map.get(x, y + 1) {
                        heap.push(HeapItem {
                            heat_loss: heat_loss + neighbour_heat_loss,
                            node: (x, y + 1, steps + 1, Dir::South),
                        });
                    }
                }
                if steps >= min_straight {
                    if x > 0 {
                        if let Some(neighbour_heat_loss) = heat_loss_map.get(x - 1, y) {
                            heap.push(HeapItem {
                                heat_loss: heat_loss + neighbour_heat_loss,
                                node: (x - 1, y, 1, Dir::West),
                            });
                        }
                    }
                    if let Some(neighbour_heat_loss) = heat_loss_map.get(x + 1, y) {
                        heap.push(HeapItem {
                            heat_loss: heat_loss + neighbour_heat_loss,
                            node: (x + 1, y, 1, Dir::East),
                        });
                    }
                }
            }
            Dir::East => {
                if steps < max_straight {
                    if let Some(neighbour_heat_loss) = heat_loss_map.get(x + 1, y) {
                        heap.push(HeapItem {
                            heat_loss: heat_loss + neighbour_heat_loss,
                            node: (x + 1, y, steps + 1, Dir::East),
                        });
                    }
                }
                if steps >= min_straight {
                    if y > 0 {
                        if let Some(neighbour_heat_loss) = heat_loss_map.get(x, y - 1) {
                            heap.push(HeapItem {
                                heat_loss: heat_loss + neighbour_heat_loss,
                                node: (x, y - 1, 1, Dir::North),
                            });
                        }
                    }
                    if let Some(neighbour_heat_loss) = heat_loss_map.get(x, y + 1) {
                        heap.push(HeapItem {
                            heat_loss: heat_loss + neighbour_heat_loss,
                            node: (x, y + 1, 1, Dir::South),
                        });
                    }
                }
            }
            Dir::West => {
                if x > 0 && steps < max_straight {
                    if let Some(neighbour_heat_loss) = heat_loss_map.get(x - 1, y) {
                        heap.push(HeapItem {
                            heat_loss: heat_loss + neighbour_heat_loss,
                            node: (x - 1, y, steps + 1, Dir::West),
                        });
                    }
                }
                if steps >= min_straight {
                    if y > 0 {
                        if let Some(neighbour_heat_loss) = heat_loss_map.get(x, y - 1) {
                            heap.push(HeapItem {
                                heat_loss: heat_loss + neighbour_heat_loss,
                                node: (x, y - 1, 1, Dir::North),
                            });
                        }
                    }
                    if let Some(neighbour_heat_loss) = heat_loss_map.get(x, y + 1) {
                        heap.push(HeapItem {
                            heat_loss: heat_loss + neighbour_heat_loss,
                            node: (x, y + 1, 1, Dir::South),
                        });
                    }
                }
            }
        }
    }
    None
}

fn parse(input: &str) -> Result<HeatLossMap, String> {
    if input.is_empty() {
        return Err("input is empty".to_string());
    }
    let width = input
        .lines()
        .next()
        .ok_or_else(|| "no lines in input".to_string())?
        .len();
    let tiles = input
        .chars()
        .filter(|c| *c != '\n')
        .map(|c| {
            c.to_digit(10)
                .ok_or_else(|| format!("unable to parse heat loss '{c}': unknown digit"))
        })
        .collect::<Result<_, _>>()?;

    Ok(HeatLossMap { width, tiles })
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533
"#;

    const ULTRA_EXAMPLE: &str = r#"111111111111
999999999991
999999999991
999999999991
999999999991
"#;

    #[test]
    fn shortest_path_works_for_example() {
        // given
        let heat_loss_map = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let heat_loss = shortest_path(&heat_loss_map, 0, 3);

        // then
        assert_eq!(heat_loss, Some(102));
    }

    #[test]
    fn shortest_path_works_for_ultra_example() {
        // given
        let heat_loss_map = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let heat_loss = shortest_path(&heat_loss_map, 4, 10);

        // then
        assert_eq!(heat_loss, Some(94));
    }

    #[test]
    fn shortest_path_works_for_other_ultra_example() {
        // given
        let heat_loss_map = parse(ULTRA_EXAMPLE).expect("expected successful parsing");

        // when
        let heat_loss = shortest_path(&heat_loss_map, 4, 10);

        // then
        assert_eq!(heat_loss, Some(71));
    }
}
