/*

https://adventofcode.com/2022/day/17

*/

use std::collections::HashMap;
use std::env;
use std::io;
use std::io::Write;
use std::usize;
mod ground_detection;
use crate::ground_detection::{detect_ground, ground_to_string};

#[derive(Eq, Hash, PartialEq)]
struct SavedStateKey {
    ground: u64,
    rock_type: usize,
    jet_index: usize,
}
struct SavedState {
    rock_index: usize,
    top_of_the_top: usize,
}

fn clamp<T>(input: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    match input {
        n if n < min => min,
        n if n > max => max,
        n => n,
    }
}

const ROCK_SHAPES: [[u8; 4]; 5] = [
    [
        //-
        0, 0, 0, 0b1111000,
    ],
    [
        //+
        0, 0b0100000, 0b1110000, 0b0100000,
    ],
    [
        //_|
        0, 0b0010000, 0b0010000, 0b1110000,
    ],
    [
        //|
        0b1000000, 0b1000000, 0b1000000, 0b1000000,
    ],
    [
        //[]
        0, 0, 0b1100000, 0b1100000,
    ],
];

fn pyrocastic_flow(jets_pattern: &str, rocks_count: usize, visualize: bool) {
    let mut rocks: [Vec<[u8; 4]>; 5] = Default::default();

    //this was my first optimization attempt: storing all rocks already shifted to all orizontal positions
    //(as opposed to use shift operator each time a puff)
    for (i, rock) in rocks.iter_mut().enumerate() {
        for j in 0..7 {
            let mut shifted_rock = ROCK_SHAPES[i];
            shifted_rock.iter_mut().for_each(|l| *l >>= j);
            rock.push(shifted_rock);
            if shifted_rock.iter().any(|l| (l & 1) == 1) {
                //there is at least a righmost bit 1, cannot shift further
                break;
            }
        }
    }

    let jet_pattern_len = jets_pattern.len();
    let mut jet_index = 0;
    let mut jet_it = jets_pattern
        .chars()
        .map(|c| if c == '<' { -1isize } else { 1isize })
        .cycle();
    const CHAMBER_HALF_CAPACITY: usize = 1024usize;
    const CHAMBER_CAPACITY: usize = CHAMBER_HALF_CAPACITY * 2;
    let mut chamber: [u8; CHAMBER_CAPACITY] = [0; CHAMBER_CAPACITY];
    let mut top_of_the_top = 0;
    const SPACE_FROM_BOTTOM: usize = 3;

    let mut chamber_size: usize = 0;

    //for cycle detection
    let mut detect_cycle = true;
    let mut state_cache: HashMap<SavedStateKey, SavedState> = HashMap::new();
    let mut fast_forward: usize = 0;
    let mut rock_index = 0;
    while rock_index < rocks_count {
        let jet_index_iteration = jet_index % jet_pattern_len;

        let mut rock_top: usize = top_of_the_top + 4 - 1;
        let next_chamber_size = rock_top + 1;
        //this ugly bit of code occasionaly resets half of chamber so that it can be used by new rocks, whithout this trick we would need a vector
        //that would grown massivily if we couldn't detect a cycle, but in reality, although I didn't benchmark this, I am sure that
        //this memory optimization will make it slower that using a big Vec (overdimensioned so that it doesn't need reallocation)
        if next_chamber_size % CHAMBER_CAPACITY < chamber_size % CHAMBER_CAPACITY {
            for i in &mut chamber[0..CHAMBER_HALF_CAPACITY] {
                *i = 0;
            }
        } else if next_chamber_size % CHAMBER_CAPACITY >= CHAMBER_HALF_CAPACITY
            && (chamber_size % CHAMBER_CAPACITY < CHAMBER_HALF_CAPACITY)
        {
            for i in &mut chamber[CHAMBER_HALF_CAPACITY..] {
                *i = 0;
            }
        }
        chamber_size = next_chamber_size;
        const INITIAL_ROCK_LEFT: isize = 2;
        let rock_type = rock_index % 5;
        let rock = &rocks[rock_type];

        //this was my second small optimization attempt:
        //we know that the rock won't hit the ground or other rocks while dropping of at least SPACE_FROM_BOTTOM units,
        //so we calculate the horizontal position after that not using collision detection yet
        let rock_left_max = rock.len() as isize - 1;
        let mut rock_left = jet_it
            .by_ref()
            .take(SPACE_FROM_BOTTOM)
            .fold(INITIAL_ROCK_LEFT, |acc, shift| {
                clamp(acc + shift, 0, rock_left_max)
            });
        jet_index += 3;

        let mut shifted_rock: &[u8; 4];

        loop {
            //jet effect
            let new_rock_left = rock_left + jet_it.next().unwrap();
            jet_index += 1;

            if new_rock_left >= 0
                && new_rock_left < rock.len() as isize
                && rock[new_rock_left as usize].iter().enumerate().all(
                    |(top, line): (usize, &u8)| {
                        *line & chamber[(rock_top - top) % CHAMBER_CAPACITY] == 0
                    },
                )
            {
                //there isn't horizontal collision, move it
                rock_left = new_rock_left;
            }

            shifted_rock = &rock[rock_left as usize];

            //gravity effect
            if rock_top == 3 {
                //floor hit
                break;
            } else if shifted_rock
                .iter()
                .enumerate()
                .all(|(top, line)| *line & chamber[(rock_top - top - 1) % CHAMBER_CAPACITY] == 0)
            {
                //there isn't vertical collision, move it
                rock_top -= 1;
            } else {
                //collision
                break;
            }
        }

        //the rock comes to a rest, we set bits in chamber accordingly and we update the tower size (top_of_the_top) if required
        for (top, line) in shifted_rock.iter().enumerate() {
            chamber[(rock_top - top) % CHAMBER_CAPACITY] |= line;

            if (*line != 0) && (rock_top - top + 1 > top_of_the_top) {
                top_of_the_top = rock_top - top + 1;
            }
        }

        if visualize {
            let mut index = chamber_size;
            while index + 24 >= chamber_size {
                let mut l = chamber[index % CHAMBER_CAPACITY];
                print!("{:06} ", index);
                for _ in 0..7 {
                    if (l & 0b1000000) == 0 {
                        print!(".")
                    } else {
                        print!("#")
                    }
                    l <<= 1;
                }
                println!();
                if index == 0 {
                    break;
                }
                index -= 1;
            }
        }

        if detect_cycle {
            let get_pixel_value = |x: i32, y: i32| {
                ((chamber[y as usize % CHAMBER_CAPACITY] << x) & 0b1000000) == 0b1000000
            };

            let ground = detect_ground(top_of_the_top as i32, &get_pixel_value).unwrap_or(0);

            if ground != 0 {
                if visualize {
                    print!("ground: {:61}", ground_to_string(ground));
                }
                let state_key = SavedStateKey {
                    ground,
                    rock_type,
                    jet_index: jet_index_iteration,
                };
                if let Some(state_value) = state_cache.get(&state_key) {
                    detect_cycle = false;

                    println!(
                        "Cycle from {} to {} detected",
                        state_value.top_of_the_top, top_of_the_top
                    );
                    let rocks_per_cycle = rock_index - state_value.rock_index;
                    while rock_index + rocks_per_cycle < rocks_count {
                        fast_forward += top_of_the_top - state_value.top_of_the_top;
                        rock_index += rocks_per_cycle;
                    }
                } else {
                    state_cache.insert(
                        state_key,
                        SavedState {
                            rock_index,
                            top_of_the_top,
                        },
                    );
                }
            }
        }

        if visualize {
            print!("Press Enter to continue...");
            _ = io::stdout().flush();
            let mut buffer = String::new();
            _ = io::stdin().read_line(&mut buffer);
        }

        rock_index += 1;
    }

    println!(
        "The tower of rocks is {} units tall",
        top_of_the_top + fast_forward
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let visualize: bool = args.len() > 1 && args[1] == "-v";
    let default_input_file = "input.txt".to_string();
    let input_file = args
        .get(if visualize { 2 } else { 1 })
        .unwrap_or(&default_input_file);
    let jets_pattern = std::fs::read_to_string(input_file)
        .unwrap_or_else(|_| panic!("Cannot open input file {}", input_file));
    pyrocastic_flow(&jets_pattern, 1000000000000, visualize);
}
