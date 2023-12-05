use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let almanac = parse(&content)?;

    let mapped_seeds = map_seeds(&almanac);
    if let Some(min) = mapped_seeds.iter().min() {
        println!("The lowest location number is {min}");
    } else {
        println!("There were no seeds");
    }

    let mapped_seed_ranges = map_seed_ranges(&almanac);
    if let Some(min) = mapped_seed_ranges.iter().map(|(start, _)| start).min() {
        println!("The lowest location number for seed ranges is {min}");
    }

    Ok(())
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Almanac {
    seeds: Vec<u64>,
    maps: Vec<Vec<Map>>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Map {
    source_start: u64,
    dest_start: u64,
    length: u64,
}

fn parse(input: &str) -> Result<Almanac, String> {
    let (seeds, maps) = input
        .split_once("\n\n")
        .ok_or_else(|| "unable to split seeds from maps".to_owned())?;
    let seeds = seeds
        .strip_prefix("seeds: ")
        .ok_or_else(|| format!("unable to parse seeds line '{seeds}': missing prefix"))?
        .split_whitespace()
        .map(|s| {
            s.parse::<u64>()
                .map_err(|e| format!("unable to parse seed '{s}': {e}"))
        })
        .collect::<Result<Vec<u64>, String>>()?;

    // assumption: the maps are in the correct order and we can ignore the source and destination
    // category
    let maps = maps
        .split("\n\n")
        .map(parse_map)
        .collect::<Result<Vec<Vec<Map>>, String>>()?;

    Ok(Almanac { seeds, maps })
}

fn parse_map(block: &str) -> Result<Vec<Map>, String> {
    // assumption: the map starts correctly with the source and destination category and we can
    // ignore it
    let mut map = block
        .lines()
        .skip(1)
        .map(|line| {
            let (dest, rest) = line
                .split_once(' ')
                .ok_or_else(|| format!("unable to split off destination in line '{line}'"))?;
            let dest_start = dest
                .parse::<u64>()
                .map_err(|e| format!("unable to parse destination range start '{dest}': {e}"))?;

            let (source, length) = rest.split_once(' ').ok_or_else(|| {
                format!("unable to split off source from length in line '{line}'")
            })?;
            let source_start = source
                .parse::<u64>()
                .map_err(|e| format!("unable to parse source range start '{source}': {e}"))?;
            let length = length
                .parse::<u64>()
                .map_err(|e| format!("unable to parse length '{length}': {e}"))?;

            Ok(Map {
                source_start,
                dest_start,
                length,
            })
        })
        .collect::<Result<Vec<Map>, String>>()?;

    map.sort_unstable_by_key(|m| m.source_start);
    Ok(map)
}

fn map_seeds(almanac: &Almanac) -> Vec<u64> {
    almanac
        .seeds
        .iter()
        .copied()
        .map(|seed| {
            almanac
                .maps
                .iter()
                .fold(seed, |source, map| map_category(source, map))
        })
        .collect()
}

fn map_category(cat: u64, map: &[Map]) -> u64 {
    // assumption: mapping ranges do not overlap
    map.iter()
        .filter(|m| m.source_start <= cat && cat < m.source_start + m.length)
        .map(|m| (cat - m.source_start) + m.dest_start)
        .next()
        .unwrap_or(cat)
}

fn map_seed_ranges(almanac: &Almanac) -> Vec<(u64, u64)> {
    // assumption: input seeds are valid ranges (i.e. seeds array has an even length)
    let mut cat_ranges: Vec<(u64, u64)> = almanac
        .seeds
        .chunks_exact(2)
        .map(|chunk| (chunk[0], chunk[0] + chunk[1]))
        .collect();

    for map in &almanac.maps {
        cat_ranges = cat_ranges
            .iter()
            .flat_map(|(cat_start, cat_end)| map_category_range(*cat_start, *cat_end, map))
            .collect();
    }

    cat_ranges
}

fn map_category_range(mut cat_start: u64, cat_end: u64, map: &[Map]) -> Vec<(u64, u64)> {
    let mut destinations: Vec<(u64, u64)> = Vec::with_capacity(100);

    for m in map {
        let start = m.source_start;
        let end = m.source_start + m.length;
        // the maps are sorted by source_start, so if the category starts before the current
        // mapping range, it is unmapped and thus taken over in the destination 1:1
        if cat_start < start {
            let new_cat_start = start.min(cat_end);
            destinations.push((cat_start, new_cat_start));
            cat_start = new_cat_start;
        }
        if cat_start >= cat_end {
            break;
        }
        // If the end of the source sequence is before the
        // start of the mapping range, we can end this loop here
        if cat_end <= start {
            break;
        }
        // if the start of the source sequence is after this mapping sequence, skip to the next map
        if cat_start >= end {
            continue;
        }
        let overlap_start = start.max(cat_start);
        let overlap_end = end.min(cat_end);
        destinations.push((
            overlap_start - m.source_start + m.dest_start,
            overlap_end - m.source_start + m.dest_start,
        ));

        cat_start = overlap_end;
    }
    // deal with possible leftover unmapped source range
    if cat_start < cat_end {
        destinations.push((cat_start, cat_end));
    }

    destinations
}

#[cfg(test)]
mod test {
    use super::*;

    const ALMANAC: &str = r#"seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4
"#;

    #[test]
    fn map_seeds_works_for_example() {
        // given
        let almanac = parse(ALMANAC).expect("expected successful parsing");

        // when
        let mapped = map_seeds(&almanac);

        // then
        assert_eq!(&mapped, &[82, 43, 86, 35]);
    }

    #[test]
    fn map_category_works_for_example() {
        // given
        let map = &[
            Map {
                dest_start: 50,
                source_start: 98,
                length: 2,
            },
            Map {
                dest_start: 52,
                source_start: 50,
                length: 48,
            },
        ];
        let seeds = &[79, 14, 55, 13];

        // when
        let soils: Vec<u64> = seeds
            .iter()
            .copied()
            .map(|seed| map_category(seed, map))
            .collect();

        // then
        assert_eq!(&soils, &[81, 14, 57, 13]);
    }

    #[test]
    fn map_seed_ranges_works_for_example() {
        // given
        let almanac = parse(ALMANAC).expect("expected successful parsing");

        // when
        let mapped = map_seed_ranges(&almanac);

        // then
        assert_eq!(mapped.iter().map(|(start, _)| start).min(), Some(&46));
    }
}
