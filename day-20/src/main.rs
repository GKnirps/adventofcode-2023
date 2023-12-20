use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let (modules, connections) = parse(&content)?;

    let (low, high) = push_button_n(modules, &connections, 1000);
    println!(
        "After pressing the button 1000 times: {low} low pulses, {high} high pulses, product: {}",
        low * high
    );

    Ok(())
}

fn push_button_n(
    mut modules: HashMap<&str, Module>,
    connections: &HashMap<&str, Vec<&str>>,
    n: usize,
) -> (u64, u64) {
    let mut low = 0;
    let mut high = 0;
    for _ in 0..n {
        let (l, h) = push_button(&mut modules, connections);
        low += l;
        high += h;
    }
    (low, high)
}

fn push_button(
    modules: &mut HashMap<&str, Module>,
    connections: &HashMap<&str, Vec<&str>>,
) -> (u64, u64) {
    let mut low: u64 = 0;
    let mut high: u64 = 0;
    // false means low pulse, true means high pulse
    let mut queue: VecDeque<(&str, bool, &str)> = VecDeque::with_capacity(modules.len());
    queue.push_back(("broadcaster", false, "button"));

    while let Some((name, pulse, sender)) = queue.pop_front() {
        if pulse {
            high += 1;
        } else {
            low += 1;
        }

        match modules.get_mut(name) {
            Some(Module::FlipFlop(state)) => {
                if !pulse {
                    *state = !*state;
                    if let Some(outputs) = connections.get(name) {
                        for out in outputs {
                            queue.push_back((out, *state, name))
                        }
                    }
                }
            }
            Some(Module::Conjunction(inputs)) => {
                // TODO: this handling of the input pulse may be wrong, check if it is
                if let Some(state) = inputs.get_mut(sender) {
                    *state = pulse;
                }
                let out_pulse = !inputs.values().all(|state| *state);
                if let Some(outputs) = connections.get(name) {
                    for out in outputs {
                        queue.push_back((out, out_pulse, name))
                    }
                }
            }
            Some(Module::Broadcast) => {
                if let Some(outputs) = connections.get(name) {
                    for out in outputs {
                        queue.push_back((out, pulse, name))
                    }
                }
            }
            None => {
                eprintln!("tried to send pulse to unknown module '{name}', ignoring pulse");
            }
        }
    }

    (low, high)
}

#[derive(Clone, Debug)]
enum Module<'a> {
    FlipFlop(bool),
    Conjunction(HashMap<&'a str, bool>),
    Broadcast,
}

fn parse(input: &str) -> Result<(HashMap<&str, Module>, HashMap<&str, Vec<&str>>), String> {
    let mut modules: HashMap<&str, Module> = HashMap::with_capacity(input.len());
    let mut connections: HashMap<&str, Vec<&str>> = HashMap::with_capacity(input.len());

    for parsed in input.lines().map(parse_module) {
        let (name, module, out) = parsed?;
        modules.insert(name, module);
        connections.insert(name, out);
    }

    for (from, tos) in connections.iter() {
        for to in tos {
            if let Some(Module::Conjunction(inputs)) = modules.get_mut(to) {
                inputs.insert(from, false);
            }
        }
    }
    Ok((modules, connections))
}

fn parse_module(line: &str) -> Result<(&str, Module<'_>, Vec<&str>), String> {
    let (module, out) = line
        .split_once(" -> ")
        .ok_or_else(|| format!("unable to split module from its outputs in line '{line}'"))?;

    let (module, name) = if module == "broadcaster" {
        (Module::Broadcast, module)
    } else if let Some(name) = module.strip_prefix('%') {
        (Module::FlipFlop(false), name)
    } else if let Some(name) = module.strip_prefix('&') {
        (Module::Conjunction(HashMap::with_capacity(8)), name)
    } else {
        return Err(format!("unknown module type in line '{line}"));
    };

    let out: Vec<&str> = out.split(", ").collect();

    Ok((name, module, out))
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE1: &str = r#"broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a
"#;
    const EXAMPLE2: &str = r#"broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output
"#;

    #[test]
    fn push_button_n_works_for_example_1() {
        // given
        let (modules, connections) = parse(EXAMPLE1).expect("expected successful parsing");

        // when
        let (low, high) = push_button_n(modules, &connections, 1000);

        // then
        assert_eq!(low, 8000);
        assert_eq!(high, 4000);
    }

    #[test]
    fn push_button_n_works_for_example_2() {
        // given
        let (modules, connections) = parse(EXAMPLE2).expect("expected successful parsing");

        // when
        let (low, high) = push_button_n(modules, &connections, 1000);

        // then
        assert_eq!(low, 4250);
        assert_eq!(high, 2750);
    }
}
