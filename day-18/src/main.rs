use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let instructions = parse(&content)?;

    let lagoon_area = dig(&instructions);
    println!("The lagoon can hold {lagoon_area} m³ lava");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Dir {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Instruction {
    dir: Dir,
    length: u32,
    color: u32,
}

fn parse(input: &str) -> Result<Vec<Instruction>, String> {
    input.lines().map(parse_instruction).collect()
}

fn parse_instruction(line: &str) -> Result<Instruction, String> {
    let (dir, rest) = line
        .split_once(' ')
        .ok_or_else(|| format!("unable to split direction from rest in line '{line}'"))?;
    let dir = match dir {
        "U" => Ok(Dir::Up),
        "R" => Ok(Dir::Right),
        "D" => Ok(Dir::Down),
        "L" => Ok(Dir::Left),
        _ => Err(format!("unknown direction '{dir}' in line '{line}'")),
    }?;

    let (length, color) = rest
        .split_once(" (#")
        .ok_or_else(|| format!("unable to split length fromcolor in line '{line}'"))?;
    let length = length
        .parse::<u32>()
        .map_err(|e| format!("unable to parse length '{length}' in line '{line}': {e}"))?;
    let color = u32::from_str_radix(
        color
            .strip_suffix(')')
            .ok_or_else(|| format!("missing closing paranthesis after color in line '{line}'"))?,
        16,
    )
    .map_err(|e| format!("unable to parse color '{color}' in line '{line}': {e}"))?;

    Ok(Instruction { dir, length, color })
}

fn dig(instructions: &[Instruction]) -> usize {
    if instructions.is_empty() {
        return 0;
    }
    // let's do a simple implementation with a flood fill for part 1
    let mut pos: (i32, i32) = (0, 0);
    let mut dug_out: HashSet<(i32, i32)> = HashSet::with_capacity(instructions.len() * 10);

    for Instruction {
        dir,
        length,
        color: _,
    } in instructions
    {
        for _ in 0..*length {
            dug_out.insert(pos);
            match dir {
                Dir::Up => {
                    pos.1 -= 1;
                }
                Dir::Right => {
                    pos.0 += 1;
                }
                Dir::Down => {
                    pos.1 += 1;
                }
                Dir::Left => {
                    pos.0 -= 1;
                }
            }
        }
    }

    let min_x = dug_out.iter().map(|(x, _)| *x).min().unwrap_or(0) - 1;
    let min_y = dug_out.iter().map(|(_, y)| *y).min().unwrap_or(0) - 1;
    let max_x = dug_out.iter().map(|(x, _)| *x).max().unwrap_or(0) + 1;
    let max_y = dug_out.iter().map(|(_, y)| *y).max().unwrap_or(0) + 1;

    let total_area = ((max_x - min_x + 1) * (max_y - min_y + 1)) as usize;

    let mut queue: Vec<(i32, i32)> = Vec::with_capacity(total_area);
    let mut outside: HashSet<(i32, i32)> = HashSet::with_capacity(total_area);

    queue.push((min_x, min_y));
    while let Some((x, y)) = queue.pop() {
        if !outside.contains(&(x, y))
            && !dug_out.contains(&(x, y))
            && x >= min_x
            && y >= min_y
            && x <= max_x
            && y <= max_y
        {
            outside.insert((x, y));
            queue.push((x - 1, y));
            queue.push((x, y - 1));
            queue.push((x + 1, y));
            queue.push((x, y + 1));
        }
    }

    total_area - outside.len()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)
"#;

    #[test]
    fn dig_works_for_example() {
        // given
        let instructions = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let dug_out = dig(&instructions);

        // then
        assert_eq!(dug_out, 62);
    }
}
