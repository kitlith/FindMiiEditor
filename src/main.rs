use serde::{Serialize, Deserialize};
use std::fs::File;
use std::path::PathBuf;
use std::io::{Read, Write};
use byteorder::{BigEndian, ByteOrder};
use structopt::StructOpt;
use rand::prelude::*;
use rand::distributions::uniform::{Uniform, SampleUniform};
use rand::SeedableRng;
use std::fmt::Display;

use binread::{BinRead, ReadOptions};
use binwrite::BinWrite;

#[derive(Serialize, Deserialize, BinRead, BinWrite, Default, Debug)]
#[br(big)]
#[binwrite(big)]
struct Level {
    num_miis: u32,
    behavior: u32,
    level_type: u32,
    map: u32,
    zoom_out_max: f32,
    zoom_in_max: f32,
    unk7: f32,
    horiz_dist: f32,
    vert_dist: f32,
    darkness: f32,
    head_size: f32,
    unk12: f32,
    unk13: f32,
    unk14: f32,
    unk15: f32,
    unk16: f32
}

// error handling omitted for the moment.
impl Level {
    fn from_file(mut input: File) -> Vec<Level> {
        let mut options = ReadOptions::default();
        options.count = Some(99); // binread doesn't like a missing count for a vec... but there's supposed to be exactly 99 anyway.
        BinRead::read_options(&mut input, &options, ()).unwrap()
    }

    fn to_file(mut output: File, levels: Vec<Level>) {
        levels.write(&mut output).unwrap();
    }
}

#[derive(Debug, StructOpt)]
enum Args {
    Assemble {
        input: PathBuf,
        output: Option<PathBuf>
    },
    Disassemble {
        #[structopt(long, short)]
        compact: bool,
        input: PathBuf,
        output: Option<PathBuf>
    },
    Randomize {
        input: PathBuf,
        output: PathBuf,
        #[structopt(long, short)]
        seed: Option<u64>
        // TODO: let the parameters be tweaked
    }
}

#[derive(Debug)]
enum Range<T> where T: SampleUniform + PartialOrd + Display + Copy {
    Exact(T),
    Constraint { min: T, max: T }, // inclusive for the moment
}

impl<T> Range<T> where T: SampleUniform + PartialOrd + Display + Copy {
    fn min(&mut self, new_min: T) -> Result<(), String> {
        match self {
            Range::Exact(val) if new_min > *val
                => Err(format!("No possible value: new minimum value ({}) greater than exact value ({})", new_min, val))?,
            Range::Exact(_)
                => {},
            Range::Constraint { max, .. } if new_min > *max
                => Err(format!("No possible value: new minimum value ({}) greater than maximum value ({})", new_min, max))?,
            Range::Constraint { min, .. } if new_min > *min
                => *min = new_min,
            Range::Constraint { .. }
                => {},
        }

        Ok(())
    }

    fn max(&mut self, new_max: T) -> Result<(), String> {
        match self {
            Range::Exact(val) if new_max < *val
                => Err(format!("No possible value: new maximum value ({}) smaller than exact value ({})", new_max, val))?,
            Range::Exact(_)
                => {},
            Range::Constraint { min, .. } if new_max < *min
                => Err(format!("No possible value: new maximum value ({}) smaller than minimum value ({})", new_max, min))?,
            Range::Constraint { max, .. } if new_max < *max
                => *max = new_max,
            Range::Constraint { .. }
                => {},
        }

        Ok(())
    }

    // convienence wrapper around min+max at same time
    fn constrain(&mut self, new_min: T, new_max: T) -> Result<(), String> {
        self.min(new_min)?;
        self.max(new_max)
    }

    fn value(&mut self, new_value: T) -> Result<(), String> {
        match self {
            Range::Exact(val) if *val != new_value
                => Err(format!("No possible value: constrained to two different exact values. old: {}, new: {}", val, new_value))?,
            Range::Exact(_)
                => {},
            Range::Constraint { min, .. } if new_value < *min
                => Err(format!("No possible value: new exact value ({}) smaller than minimum value ({})", new_value, min))?,
            Range::Constraint { max, .. } if new_value > *max
                => Err(format!("No possible value: new exact value ({}) greater than maximum value ({})", new_value, max))?,
            Range::Constraint { .. }
                => *self = Range::Exact(new_value)
        }

        Ok(())
    }
}

impl<T> Distribution<T> for Range<T> where T: SampleUniform + PartialOrd + Display + Copy {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        match self {
            Range::Exact(val) => *val,
            Range::Constraint { min, max } => rng.sample(Uniform::new_inclusive(min, max))
        }
    }
}

struct Set<T>(std::collections::BTreeSet<T>);

impl<T> Set<T> where T: Ord + Display + Clone {
    fn new(elems: &[T]) -> Self {
        Set(elems.iter().cloned().collect())
    }

    fn remove(&mut self, elem: &T) -> Result<(), String> {
        self.0.remove(elem);
        if self.0.len() == 0 {
            Err(format!("Removed last element from set: {}", elem))
        } else {
            Ok(())
        }
    }

    fn subtract(&mut self, elems: &[T]) -> Result<(), String> {
        for elem in elems {
            self.remove(elem)?;
        }

        Ok(())
    }

    fn intersect(&mut self, elems: &[T]) -> Result<(), String> {
        self.0 = self.0.intersection(&elems.iter().cloned().collect()).cloned().collect();
        if self.0.len() == 0 {
            Err("No items left in set!".to_string())
        } else {
            Ok(())
        }
    }
}

impl<T> Distribution<T> for Set<T> where T: Ord + Clone {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        if self.0.len() == 0 {
            panic!("Attempted to sample an empty set!");
        }

        let idx = rng.gen_range(0, self.0.len());

        // take the item at the randomly generated index in the BTreeSet
        self.0.iter().skip(idx).next().unwrap().clone()
    }
}

fn main() {
    let args = Args::from_args();

    match args {
        Args::Assemble {input, output} => {
            let mut input_file = File::open(&input).unwrap();
            let levels: Vec<Level> = serde_json::from_reader(input_file).unwrap();

            let mut favorite_pending = false;
            for (idx, level) in levels.iter().enumerate() {
                match level.level_type {
                    6 if !favorite_pending => favorite_pending = true,
                    6 if favorite_pending => println!("Warning: level index {} is of type 'pick your favorite' after another 'pick your favorite' level. Game will crash.", idx),
                    7 if !favorite_pending => println!("Warning: level index {} is of type 'find your favorite' without a preceeding 'pick your favorite' level. Game will crash.", idx),
                    7 if favorite_pending => favorite_pending = false,
                    9  | 10 | 11 if level.behavior != 1 && level.behavior != 4 =>
                        println!("Warning: level index {} has an objective that requires mii behaviors 1 or 4 to function properly, but is set to {}", idx, level.behavior),
                    17 | 18 | 19 if level.behavior != 0 =>
                        println!("Warning: level index {} has an objective that requires mii behavior 0 to function properly, but is set to {}", idx, level.behavior),
                    _ => {}
                }

                let max_miis = match level.map {
                    4 => 40,
                    _ => 99
                };

                if level.num_miis > max_miis {
                    println!("Warning: level index {} has more than the maximum of {} miis for this level type.", idx, max_miis);
                }
            }

            if favorite_pending {
                println!("Warning: there is no matching 'find your favorite' level to a 'pick your favorite' level, and we've reached the end of the file. Game will crash.");
            }

            let output = output.unwrap_or_else(|| input.with_extension("bin"));
            let mut output = File::create(output).unwrap();

            Level::to_file(output, levels);
        },
        Args::Disassemble {compact, input, output} => {
            let levels = Level::from_file(File::open(&input).unwrap());

            let output = output.unwrap_or_else(|| input.with_extension("json"));
            let output = File::create(output).unwrap();
            if compact {
                serde_json::to_writer(output, &levels).unwrap();
                //println!("{}", toml::to_string(&levels).unwrap());
            } else {
                serde_json::to_writer_pretty(output, &levels).unwrap();
                //println!("{}", toml::to_string_pretty(&levels).unwrap());
            }
        },
        Args::Randomize {input, output, seed} => {
            let mut levels = Level::from_file(File::open(&input).unwrap());

            let seed = seed.unwrap_or_else(|| random());
            println!("Using seed: {}", seed);
            let mut rng = SmallRng::seed_from_u64(seed);

            let mut favorite_pending = false;

            let last_idx = levels.len() - 1;
            for (i, mut level) in levels.iter_mut().enumerate() {
                level.level_type = if (i == last_idx) {
                    if favorite_pending {
                        favorite_pending = false;
                        7
                    } else {
                        // avoid generating 6 or 7
                        let mut level_type = rng.sample(Uniform::new_inclusive(1, 19));
                        if level_type > 5 {
                            level_type += 2;
                        }
                        level_type
                    }
                } else {
                    let level_type = rng.sample(Uniform::new_inclusive(1, 21));
                    if level_type == 6 || level_type == 7 {
                        // special handling for levels dealing with favorites:
                        if favorite_pending {
                            favorite_pending = false;
                            7
                        } else {
                            favorite_pending = true;
                            6
                        }
                    } else {
                        level_type
                    }
                };

                let max_miis = match level.map {
                    4 => 40,
                    _ => 90
                };

                level.num_miis = rng.sample(Uniform::new_inclusive(4, max_miis));
                level.behavior = match level.level_type {
                    9  | 10 | 11 => if rng.gen_ratio(1, 2) { 1 } else { 4 }
                    17 | 18 | 19 => 0,
                    _ => rng.sample(Uniform::new_inclusive(0, 6)),
                };

                level.map = rng.sample(Uniform::new_inclusive(0, 4));
                level.zoom_out_max = rng.sample(Uniform::new_inclusive(-406.0, -135.0));
                level.zoom_in_max = rng.sample(Uniform::new_inclusive(-135.0, -22.0));

                level.darkness = if rng.gen_ratio(1, 2) {
                    0.0 // 50% chance for no darkness
                } else {
                    rng.sample(Uniform::new_inclusive(38.0, 90.0))
                };
                level.head_size = rng.sample(Uniform::new_inclusive(1.35, 3.5));
            }

            let output = File::create(output).unwrap();
            Level::to_file(output, levels);
        }
    }

}
