use serde::{Serialize, Deserialize};
use std::fs::File;
use std::path::PathBuf;
use std::io::{Read, Write};
use byteorder::{BigEndian, ByteOrder};
use structopt::StructOpt;
use rand::prelude::*;
use rand::distributions::uniform::Uniform;
use rand::SeedableRng;

#[derive(Serialize, Deserialize, Default, Debug)]
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

impl Level {
    fn from_bytes(input: &[u8; 64]) -> Level {
        Level {
            num_miis: BigEndian::read_u32(&input[0..]),
            behavior: BigEndian::read_u32(&input[4..]),
            level_type: BigEndian::read_u32(&input[8..]),
            map: BigEndian::read_u32(&input[12..]),
            zoom_out_max: BigEndian::read_f32(&input[16..]),
            zoom_in_max: BigEndian::read_f32(&input[20..]),
            unk7: BigEndian::read_f32(&input[24..]),
            horiz_dist: BigEndian::read_f32(&input[28..]),
            vert_dist: BigEndian::read_f32(&input[32..]),
            darkness: BigEndian::read_f32(&input[36..]),
            head_size: BigEndian::read_f32(&input[40..]),
            unk12: BigEndian::read_f32(&input[44..]),
            unk13: BigEndian::read_f32(&input[48..]),
            unk14: BigEndian::read_f32(&input[52..]),
            unk15: BigEndian::read_f32(&input[56..]),
            unk16: BigEndian::read_f32(&input[60..]),
        }
    }

    fn to_bytes(&self, output: &mut [u8; 64]) {
        BigEndian::write_u32(&mut output[0..], self.num_miis);
        BigEndian::write_u32(&mut output[4..], self.behavior);
        BigEndian::write_u32(&mut output[8..], self.level_type);
        BigEndian::write_u32(&mut output[12..], self.map);
        BigEndian::write_f32(&mut output[16..], self.zoom_out_max);
        BigEndian::write_f32(&mut output[20..], self.zoom_in_max);
        BigEndian::write_f32(&mut output[24..], self.unk7);
        BigEndian::write_f32(&mut output[28..], self.horiz_dist);
        BigEndian::write_f32(&mut output[32..], self.vert_dist);
        BigEndian::write_f32(&mut output[36..], self.darkness);
        BigEndian::write_f32(&mut output[40..], self.head_size);
        BigEndian::write_f32(&mut output[44..], self.unk12);
        BigEndian::write_f32(&mut output[48..], self.unk13);
        BigEndian::write_f32(&mut output[52..], self.unk14);
        BigEndian::write_f32(&mut output[56..], self.unk15);
        BigEndian::write_f32(&mut output[60..], self.unk16);
    }

    fn from_file(mut input: File) -> Vec<Level> {
        let mut levels: Vec<Level> = Vec::new();
        let mut lvl_bytes = [0u8;64];
        while input.read_exact(&mut lvl_bytes).is_ok() {
            levels.push(Level::from_bytes(&lvl_bytes));
        }
        levels
    }

    fn to_file(mut output: File, levels: Vec<Level>) {
        let mut lvl_bytes = [0u8;64];
        for level in levels {
            level.to_bytes(&mut lvl_bytes);
            output.write_all(&lvl_bytes).unwrap();
        }
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
                    6 if favorite_pending => println!("Warning: level index {} is of type 'pick your favorite' after another 'pick your favorite'. Game will crash.", idx),
                    7 if !favorite_pending => println!("Warning: level index {} is of type 'find your favorite' without a preceeding 'pick your favorite'. Game will crash.", idx),
                    7 if favorite_pending => favorite_pending = false,
                    9  | 10 | 11 if level.behavior != 1 && level.behavior != 4 =>
                        println!("Warning: level index {} has an objective that requires behavior 1 or 4 to function properly, but is set to {}", idx, level.behavior),
                    17 | 18 | 19 if level.behavior != 0 =>
                        println!("Warning: level index {} has an objective that requires behavior 0 to function properly, but is set to {}", idx, level.behavior),
                    _ => {}
                }

                let max_miis = match level.map {
                    4 => 40,
                    _ => 90
                };

                if level.num_miis > max_miis {
                    println!("Warning: level index {} has more than the maximum of {} miis.", idx, max_miis);
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
