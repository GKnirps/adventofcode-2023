use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let (modules, connections) = parse(&content)?;

    let (low, high) = push_button_n(modules.clone(), &connections, 1000);
    println!(
        "After pressing the button 1000 times: {low} low pulses, {high} high pulses, product: {}",
        low * high
    );

    match first_rx_signal(modules, &connections) {
        Ok(rx_count) => {
            println!("after {rx_count} button presses, the first low pulse has been sent to rx");
        }
        Err(e) => {
            println!("My shortcut to get the required number of button pushes failed: {e}");
        }
    }

    Ok(())
}

fn push_button_n(
    mut modules: HashMap<&str, Module>,
    connections: &HashMap<&str, Vec<&str>>,
    n: u64,
) -> (u64, u64) {
    let mut low = 0;
    let mut high = 0;
    for _ in 0..n {
        let (l, h, _) = push_button(&mut modules, connections);
        low += l;
        high += h;
    }
    (low, high)
}

fn first_rx_signal(
    mut modules: HashMap<&str, Module>,
    connections: &HashMap<&str, Vec<&str>>,
) -> Result<u64, String> {
    // we can't just push the button until rx receives a low pulse, that would take
    // very long (for my input).
    // However, in my input, the only module that feeds into rx ist a conjunction module that only
    // has conjunction modules as input,and each of those has exactly _one_ conjunction module as
    // input.
    // So the idea is: check for loops in the inputs of the conjunction that feeds into rx, and
    // calculate how long it would take for rx to receive a low pulse
    let mut rx_inputs = connections
        .iter()
        .filter(|(_, outputs)| outputs.contains(&"rx"))
        .map(|(name, _)| name);
    let rx_input = rx_inputs
        .next()
        .ok_or_else(|| "did not find any input to rx in the data".to_string())?;
    if rx_inputs.next().is_some() {
        return Err("more than one input to rx, violating my assumption".to_string());
    }

    let second_order_dep: Vec<(&str, Vec<&str>)> =
        if let Some(Module::Conjunction(inputs)) = modules.get(rx_input) {
            inputs
                .keys()
                .map(|key| {
                    if let Some(Module::Conjunction(t_inputs)) = modules.get(key) {
                        Ok((*key, t_inputs.keys().copied().collect::<Vec<&str>>()))
                    } else {
                        Err(
                    "input to rx is not a conjunction (or not present), violating my assumption"
                        .to_string(),
                )
                    }
                })
                .collect::<Result<_, _>>()?
        } else {
            return Err(
                "input to rx is not a conjunction (or not present), violating my assumption"
                    .to_string(),
            );
        };
    let inputs: Vec<(&str, Vec<&str>)> = second_order_dep
        .iter()
        .flat_map(|(_, names)| {
            names.iter().map(|key| {
                if let Some(Module::Conjunction(t_inputs)) = modules.get(key) {
                    Ok((*key, t_inputs.keys().copied().collect::<Vec<&str>>()))
                } else {
                    Err(
                    "input to rx is not a conjunction (or not present), violating my assumption"
                        .to_string(),
                )
                }
            })
        })
        .collect::<Result<_, _>>()?;

    let unique_transient_inputs = inputs
        .iter()
        .flat_map(|(_, ti)| ti)
        .copied()
        .collect::<HashSet<&str>>();
    let mut input_cycle_count: HashMap<&str, u64> =
        HashMap::with_capacity(unique_transient_inputs.len());
    let mut counter: u64 = 0;

    while input_cycle_count.len() < unique_transient_inputs.len() {
        counter += 1;
        push_button(&mut modules, connections);
        for (t_input, _) in &inputs {
            if let Some(Module::Conjunction(inputs)) = modules.get(t_input) {
                for (name, pulse) in inputs {
                    if *pulse {
                        input_cycle_count.entry(name).or_insert(counter);
                    }
                }
            } else {
                panic!("modules changed after button push, that should not happen!");
            }
        }
    }
    Ok(inputs
        .iter()
        .map(|(_, sub_inputs)| {
            // IMPORTANT: this assumes the inputs are bits of some kind of binary counter
            sub_inputs
                .iter()
                .map(|sub_name| input_cycle_count.get(sub_name).unwrap())
                .sum::<u64>()
        })
        // IMPORTANT: this assumes all the cycle lengths are primes
        // if this is not the case, the LCM needs to be used here
        .product::<u64>())
}

fn push_button(
    modules: &mut HashMap<&str, Module>,
    connections: &HashMap<&str, Vec<&str>>,
) -> (u64, u64, bool) {
    let mut low: u64 = 0;
    let mut high: u64 = 0;
    let mut rx_low: bool = false;
    // false means low pulse, true means high pulse
    let mut queue: VecDeque<(&str, bool, &str)> = VecDeque::with_capacity(modules.len());
    queue.push_back(("broadcaster", false, "button"));

    while let Some((name, pulse, sender)) = queue.pop_front() {
        if pulse {
            high += 1;
        } else {
            low += 1;
        }

        if name == "rx" {
            rx_low = rx_low || !pulse;
            continue;
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

    (low, high, rx_low)
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
