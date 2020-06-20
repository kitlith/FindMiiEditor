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
                    6 if favorite_pending => println!("Warning: level index {} is of type 'pick your favorite' after another 'pick your favorite' level. Game will crash.", idx),
                    7 if !favorite_pending => println!("Warning: level index {} is of type 'find your favorite' without a preceeding 'pick your favorite' level. Game will crash.", idx),
                    7 if favorite_pending => favorite_pending = false,
                    9  | 10 | 11 if level.behavior != 1 && level.behavior != 3 =>
                        println!("Warning: level index {} has an objective that requires mii behaviors 1 or 3 to function properly, but is set to {}", idx, level.behavior),
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
				
// This is where the magic happens!
                
                level.num_miis = rng.sample(Uniform::new_inclusive(6, max_miis)); //The Minimum is actually 6 because of the "Find 5 look alikes" Objective
				
				level.unk12 = 0.0;
				level.unk13 = 0.0;
				level.unk14 = 0.0;
				level.unk15 = 0.0;
				level.unk16 = 0.0;
                level.behavior = match level.level_type {
                    9  | 10 | 5 | 11 | 20 => if rng.gen_ratio(1, 2) { 1 } else { 3 } //Find The Odd Mii(s) out
                    17 | 18 | 19 => 0, // Find The Insomniac 
					14 | 15 | 16 => 0, // Find The Sleepyhead
					12 | 13 => if rng.gen_ratio(1, 2) { 4 } else { 2 }
                    _ => rng.sample(Uniform::new_inclusive(0, 6)),
                };


                level.map = rng.sample(Uniform::new_inclusive(0, 4));
				if level.map == 4 {
				level.behavior = rng.sample(Uniform::new_inclusive(0, 1));
}
	
                 if level.map == 1 {
                 level.behavior = match level.map {
	             1 => if rng.gen_ratio(1, 4) { 0 } else if rng.gen_ratio(1, 2) { 1 } else { 4 }
				 4 => if rng.gen_ratio(1, 2) { 0 } else { 1 } //Elevator Behavior Softlock Fix
				 3 => if rng.gen_ratio(1, 2) { 0 } else { 1 } //Tennis Stands Behavior Softlock Fix
	             _ => rng.sample(Uniform::new_inclusive(4, 5)),
                };
	}
                level.zoom_out_max = rng.sample(Uniform::new_inclusive(-406.0, -135.0));
                level.zoom_in_max = rng.sample(Uniform::new_inclusive(-135.0, -22.0));

                level.darkness = if rng.gen_ratio(1, 2) {
                    0.0 // 50% chance for no darkness
                } else {
                    rng.sample(Uniform::new_inclusive(84.0, 110.0))
                };
                level.head_size = rng.sample(Uniform::new_inclusive(1.35, 3.5));
        if level.darkness > 1.0 {
       level.unk12 = 3.0;
      level.unk13 = 10.0;
       level.unk14 = 1.0;
      level.unk15 = 0.0;
      level.unk16 = 6.0;
}
				
				if level.map == 3 { 
				level.zoom_out_max = if rng.gen_ratio(1, 2) { -279.60767 } else { -187.60767 };
				level.zoom_in_max = -279.60767;
				}
				
				if level.map == 3 { 
				level.horiz_dist = 4.7;
				level.vert_dist = 22.4;
				}
				
				if level.map == 4 { 
				level.unk7 = 35.0;
				
				}
				
			    if level.map == 2 {
				level.unk7 = 0.0;
				
				
				}
				if level.map == 3 {
				level.unk7 = -60.0;
				
				}
				//Street Mii Bounding Fixes
			    if level.map == 0 { 
                 level.zoom_out_max = if rng.gen_ratio(3, 4) { -199.0 } else { -205.0 };
				 level.zoom_in_max = if rng.gen_ratio(3, 4) { -199.0 } else { -205.0 };
			     level.horiz_dist = 35.0;
				  level.vert_dist = 97.0;
				}
				//Ocean Mii Bounding Fixes
			    if level.map == 1 { 
                 level.zoom_out_max = if rng.gen_ratio(3, 4) { -180.0 } else { -240.0 };
				 level.zoom_in_max = if rng.gen_ratio(3, 4) { -180.0 } else { -240.0 };
			     level.horiz_dist = if rng.gen_ratio(3, 4) { 30.0 } else { 40.0 };
				  level.vert_dist = if rng.gen_ratio(3, 4) { 90.0 } else { 113.0 };
			
				}
				//Better safe than sorry
				// If you have too many miis on the Escalator the game will crash
		        if level.map == 4 { // If Escalator Stage
                level.num_miis = rng.sample(Uniform::new_inclusive( 8, 40)); // Nothing more then 40 miis
				
				}
				if level.level_type == 5 {
				
				level.behavior = 1;
				}
				
				if level.level_type == 9 {
				level.behavior = 1;
				
				}
				
				if level.level_type == 10 {
				level.behavior = 1;
				
				}
				
				if level.level_type == 11 {
				level.behavior = 1;
				
				}
				
				if level.level_type == 20 {
				level.behavior = 1;
				
				}
				
                 if level.map == 3 && (level.level_type == 17 || level.level_type == 18 || level.level_type == 19) {
				 level.darkness = 0.0; // Darkness doesn't work on Find The Insomniac Stages on the Tennis Stands

				}
				
				 if level.map == 3 && (level.level_type == 12 || level.level_type == 13) { //Fastest Mii Fix on Tennis Stands
				 level.behavior = 3;
                 level.level_type == 2;

				}
				
				if level.level_type == 14 {
				level.behavior = if rng.gen_ratio(1, 2) { 1 } else { 0 }
				
				}
								
				if level.level_type == 15 {
				level.behavior = if rng.gen_ratio(1, 2) { 1 } else { 0 }
				
				}
								
				if level.level_type == 16 {
				level.behavior = if rng.gen_ratio(1, 2) { 1 } else { 0 }
				
				}
				

                 if level.map == 0 && (level.level_type == 14 || level.level_type == 15 || level.level_type == 16) {
				 level.darkness = 0.0; // Darkness doesn't work on Find the Sleepyhead Stages on the Street. Even though its in the original game levels....
				 
				 }
				 
				 if level.map == 0 && (level.zoom_out_max > -199.0) {  // Street Non Zoom Fixes
				  level.zoom_out_max = -222.0;
				   level.zoom_in_max = -222.0;
				      level.horiz_dist = 4.5;
				        level.vert_dist = 35.4;
				 


				}
				
				if level.map == 0 && (level.behavior == 2 || level.behavior == 4 || level.behavior == 6 || level.level_type == 14 || level.level_type == 15 || level.level_type == 16) {
				 level.zoom_out_max = -222.0;
				 level.zoom_in_max = -222.0;
				 level.horiz_dist = if rng.gen_ratio(1, 2) { 33.4 } else { 57.0 };
				  level.vert_dist = 57.4;
				 


				}
				
				
				if level.map == 1 {
				level.darkness = 0.0; // Darkness doesn't work on Ocean Stages


				
				}
				
				
				 if level.map == 3 && (level.behavior == 3 || level.behavior == 4 || level.behavior == 6) {
				  level.behavior = 1;
				
					
				 }
				 
				 if level.map == 2 { // If It's the Space Stage.
				 level.num_miis = rng.sample(Uniform::new_inclusive( 20, 30)); // More than 30 Miis makes the stage unplayable. Due to the game's settings in this environment, 30 > x > 20 Miis works the best without any camera zoom edits.
				 level.zoom_out_max = -180.0;
				 level.zoom_in_max = if rng.gen_ratio(1, 2) { -180.0 } else { -297.4774 };
				 
				 
				 
				 }
				 
				 
				// Unfortunately, I could not come up with a better fix. This is how it has to be until I can figure out the exteremely complicated settings for Cameras and Mii Placement this stage uses
				  if level.map == 2 && (level.behavior == 5) {
				  level.num_miis = rng.sample(Uniform::new_inclusive( 8, 12));
				  level.zoom_out_max = -222.0;
				   level.zoom_in_max = -222.0;
				   level.horiz_dist = if rng.gen_ratio(70, 100) { 30.0 } else { 35.0 };
				   level.vert_dist = 21.0;
				   
				   }
				   // Yet..... another.... find the sleepyhead fix for the escalator stage.
				   if level.map == 4 && (level.level_type == 14 || level.level_type == 15 || level.level_type == 16) {
				   level.level_type = rng.sample(Uniform::new_inclusive( 1, 5));
				  
				  
				  }
				  
				  if level.map == 2 && (level.behavior == 0 || level.behavior == 1) {
				  level.zoom_out_max = -180.0;
				   level.zoom_in_max = -297.4774;
				   level.horiz_dist = 26.9;
				   level.vert_dist = 21.9;
				   level.num_miis = rng.sample(Uniform::new_inclusive( 8, 10));
				   
				   
				   }
				   
				   if level.map == 2 && (level.level_type == 9 || level.level_type == 10 || level.level_type == 10) { //S I G H. Yet another level fix for the space stage.
				 level.behavior = 1;
				   
				   }
				   
				   //Ocean Fix for Fixated Screen with "Find The Fastest Mii"
				   if level.map == 1 && (level.behavior == 6) { 
				   level.behavior = 4; //YET... .ABOTHER.... BOUNDING.... FIX... THAT... I... MIGHT... HAVE... ALREADY... MADE... BEFORE.. BUT... STILL.. DIDNT... WORK...
				  
				  
				  }
				  // I don't think that the escalator stages even have a "Find The Odd Mii Out" Objective in the original game.
				  if level.map == 4 && (level.level_type == 9 || level.level_type == 10 || level.level_type == 11) {
				  level.level_type = if rng.gen_ratio(2, 4) { 3 } else if rng.gen_ratio(2, 4) { 9 } else { 21 }
				  
				  
				  }
				  
				  if level.map == 2 && (level.level_type == 12 || level.level_type == 13) { // YET ANOTHER FIX FOR THIS LEVEL TYPE
				  level.level_type = 1;
				  
				  
				  }
				 
				 if level.map == 2 && (level.zoom_out_max == -180.0 || level.zoom_in_max == -180.0) { // space camera randomizer param 1
				 level.horiz_dist = 29.9;
				 level.vert_dist = 16.9;
				 
				 }
				 
				 if level.map == 2 && (level.zoom_out_max == -180.0 || level.zoom_in_max == -297.4774) { // space camera randomizer param 2
				 level.horiz_dist = 29.9; 
				 level.vert_dist = 16.9;

				 
				 }
				 
				 if level.map == 2 {
				level.behavior = if rng.gen_ratio(1, 2) { 5 } else { 1 };
				 
				 }
				 
				 if level.map == 2 && (level.behavior == 5 || level.zoom_out_max == -180.0 || level.zoom_in_max == -180.0) { // space camera randomizer param 2
				 level.horiz_dist = 26.9; 
				 level.vert_dist = 21.9;
				 
				 }
				 
				 if level.map == 2 && (level.behavior == 1) { // SPACE CAMERA ODD MII OUT PARAM
				 level.horiz_dist = 26.9; 
				 level.vert_dist = 21.9;
				 level.zoom_out_max = -180.0; // You absolutely need this
				 level.zoom_in_max = -297.4774; // You absolutely need this
				 
				 
				 }
				 
	              if level.map == 0 && (level.zoom_out_max == -205.0 || level.zoom_in_max == -199.0) {
				      level.horiz_dist = 22.4;
				        level.vert_dist = 64.0;
				 
				 }
			
			        if level.map == 1 && (level.zoom_out_max == -240.0) {
			        level.zoom_in_max = -180.0;
				        level.horiz_dist = 15.0;
				        level.vert_dist = 72.0;
				 
				 }
				 //Find the Sleepyhead Tennis Stands Fix
				 if level.map == 3 && (level.level_type == 14 || level.level_type == 15 || level.level_type == 16) {
				 level.level_type = 21;
			     level.behavior = 1;
				 
				 
				 }
				 //Fastest Mii Tennis Stands
				 if level.map == 3 && (level.level_type == 12 || level.level_type == 13) {
				 	 level.level_type = 21;
				      level.behavior = 0;
				 
				 
				 
				 }
				 //Odd Mii Out Fix Tennis Stands Fix
				 	if level.map == 3 && (level.level_type == 9 || level.level_type == 10 || level.level_type == 11 || level.level_type == 20) {
				     level.behavior = 1;
				 
				 
				 }
				 
				 if level.map == 2 { 
				level.darkness = if rng.gen_ratio(2, 4) {
                    0.0 // 50% chance for no darkness
                } else {
                    rng.sample(Uniform::new_inclusive(94.9, 135.0))
				 
				 
				 };
				 
				 
		 }
		 //Insomniac Fix Tennis Stands
				 if level.map == 3 && (level.level_type == 17 || level.level_type == 18 || level.level_type == 19) {
				 level.level_type = 10;
			     level.behavior = 1;
				 
				 
				 }
				 //Street Non Zoom Level Fix for Miis
				 if level.map == 0 && (level.zoom_out_max == -205.0 || level.zoom_in_max == -205.0) {
				  level.horiz_dist = 4.7;
				  level.vert_dist = 24.0;
				 
				 }
				 
				  if level.map == 1 && (level.zoom_out_max == -180.0 || level.zoom_in_max == -180.0) {
				  level.horiz_dist = 10.2;
				  level.vert_dist = 17.4;
				 
				}
			
			 if level.map == 0 && (level.behavior == 5) {
				  level.behavior = 1;
				
				}
				
			 if level.map == 0 && (level.zoom_out_max == -199.0 || level.zoom_in_max == -199.0) {
				  level.horiz_dist = 4.0;
				  level.vert_dist = 11.0;
				
				}
				
				 if level.map == 0 && (level.horiz_dist == 10.2 || level.vert_dist == 17.4) {
				  level.horiz_dist = 22.4;
				  level.vert_dist = 64.0;
				  level.zoom_in_max = -205.0;
				
				}
				
				if level.map == 2 && (level.level_type == 12 || level.level_type == 13) {
				  level.horiz_dist = 16.3;
				  level.vert_dist = 12.8;				  
				}
				//Darkness Find The Odd Mii Out fix on Space Stages. Which doesnt work for some reason
				if level.map == 2 && (level.level_type == 9 || level.level_type == 10 || level.level_type == 11) {
				 level.darkness = 0.0;
				
				
				}
				//Yet Another Find the Fastest Mii Escalator Fix
				if level.map == 4 && (level.level_type == 13) {
				 level.level_type = if rng.gen_ratio(2, 4) { 10 } else { 2 };
			     level.behavior = 1;
				
				}
				
				
		//Yet Another Find the Fastest Mii Escalator Fix For Type 12
				if level.map == 4 && (level.level_type == 12) {
				 level.level_type = if rng.gen_ratio(2, 4) { 10 } else { 2 };
			     level.behavior = 1;
				 
				 }
				 
				if level.level_type == 17 {
				
				level.behavior = 0;
				}
				
				if level.level_type == 18 {
				level.behavior = 0;
				
				}
				
				if level.level_type == 19 {
				level.behavior = 0;
				
				}
				// "Find 3 odd miis out" doesnt work with Darkness on this stage for some reason
				 if level.map == 0 && (level.level_type == 11 || level.darkness > 0.1) { // Set to greater than 0.1, Just in case
				   level.level_type = 10;
				
				}
				
		//Street Mii Bounding Fix for Odd Mii out at
				 if level.map == 0 && (level.level_type == 9 || level.level_type == 10) {
				   level.behavior = 1;
				
				}
				
				//Ocean Behavior Fix for Fastest Mii Levels
				 if level.map == 1 && (level.level_type == 12 || level.level_type == 13) { 
				   level.behavior = if rng.gen_ratio(3, 5) { 4 } else { 6 }
				
				}
				
				if level.map == 0 && (level.zoom_out_max == -222.0 || level.zoom_in_max == -222.0 || level.behavior == 0 || level.behavior == 1 || level.behavior == 3 || level.behavior == 5 ) {
				  level.horiz_dist = 11.4;
				  level.vert_dist = 12.9;
				  
				}
				
				if level.map == 3 && (level.behavior == 5) {
				level.behavior = 1;
				level.level_type = 10;
				}

	          if level.map == 1 && (level.zoom_out_max == -180.0 || level.zoom_out_max == -180.0) {  
				     level.horiz_dist = 8.2;
				      level.vert_dist = 12.4;
				
				
				}
				//Fix for "Find The Insomniac" levels on the Escalator stage. Which don't function. 
				if level.map == 1 && (level.level_type == 17 || level.level_type == 18 || level.level_type == 19) { 
			    level.behavior = 1;
				level.level_type = 21;
				
				}
				//Ocean Mii Behavior 4 Fix
				if level.map == 1 && (level.zoom_out_max == -240.0 || level.zoom_out_max == -240.0 || level.behavior == 4 || level.behavior == 2 || level.behavior == 6) {
				level.horiz_dist = 38.2;
		        level.vert_dist = 26.4;
				
				}
				//Yet another Mii Behavior Type 3 fix for the tennis stands stages
				if level.map == 3 && (level.behavior == 3) {
				 level.behavior = 1;
				 
				}
				//Find 3 look alikes doesn't work on Tennis Stands with darkness for some reason
				if level.map == 3 && (level.level_type == 2 || level.level_type == 4 || level.darkness > 0.1) { // Set to greater than 0.1, Just in case
				 level.level_type = 10;
				 level.behavior = 1;
				   
				   
				   }
				   if level.map == 1 && (level.zoom_out_max == -240.0 || level.zoom_out_max == -240.0 || level.behavior == 2 || level.behavior == 4 || level.behavior == 6 ) { // Yet Another Ocean Mii Position Fix
				   level.horiz_dist = 40.6;
		           level.vert_dist = 60.4;
				   
				   }
				   
				   //Find 3 - 5 look alikes doesn't work on Street Stagess with darkness for some reason
				if level.map == 0 && (level.level_type == 2 || level.level_type == 3 || level.level_type == 4 || level.darkness > 0.1) { // Set to greater than 0.1, Just in case
				 level.level_type = 8;
				 level.behavior = 4;
				   
				   
				   }
				   if level.map == 3 && (level.zoom_out_max == -279.60767 || level.zoom_out_max == -279.60767) { //Hopefully the last Tennis Mii Pos Fix
				   level.horiz_dist = 5.4;
		           level.vert_dist = 8.2;
				   
				   }
				   //I don't even know anymore....
				   if level.map == 0 && (level.behavior == 4 || level.zoom_out_max == -222.0 || level.zoom_out_max == -222.0) { //Screen Bounding Fix for Street Levels with walking Miis
				   level.unk7 = 17.0; // This is way to complicated to explain currently. TLDR: Screen Position on Map + Respawn Point Area
				   level.horiz_dist = rng.sample(Uniform::new_inclusive(30.0, 38.3));
				   level.vert_dist = 117.0;
				   level.zoom_in_max = -200.0;
				   level.zoom_out_max = -200.0;
				   level.num_miis = rng.sample(Uniform::new_inclusive( 8, 45)); //Anything higher than 45 causes half the original amount of miis to spawn.
				   
				   
				   
				   }
				   
				   if level.map == 1 && (level.zoom_out_max == -240.0 || level.zoom_out_max == -240.0 || level.behavior == 0 || level.behavior == 1 || level.behavior == 3) { // Yet Again, Another Ocean Mii Position Fix
				   level.zoom_out_max = -255.0;
				    level.zoom_in_max = -255.0;
				     level.horiz_dist = 6.4;
		              level.vert_dist = 17.1;
					  
				   }
				   
				   if level.map == 1 && (level.zoom_out_max == -255.0) {
				   //You can have up to 99 miis on the ocean stage but I have to add this because Mii Bounding wont work without it (Offscreen Miis Fix)
				   level.num_miis = rng.sample(Uniform::new_inclusive( 8, 64));
				   
				   
				   }
				   
				 if level.map == 4 && (level.level_type == 3 || level.level_type == 4 || level.darkness > 0.1) {//Find 5 Lookalikes darkness fix
				 level.level_type = 21;
				 
				 }
				 //I really really hope this is the last bounding fix
				 if level.map == 1 && (level.zoom_out_max == -255.0 || level.zoom_in_max == -255.0 || level.behavior == 2 || level.behavior == 4 || level.behavior == 6) {
				 level.horiz_dist = 40.6;
		         level.vert_dist = 60.4;
				 level.unk7 = 0.0;
				 
				 
				 }
				 
				 // Nope.
			     if level.map == 1 && (level.zoom_out_max == -255.0 || level.zoom_in_max == -255.0 || level.behavior == 0 || level.behavior == 1 || level.behavior == 3) {
				 level.horiz_dist = 6.4;
		         level.vert_dist = 17.1;
				 level.unk7 = 0.0;
				 
				 }
				 
				  if level.map == 4 && (level.level_type == 17 || level.level_type == 18 || level.level_type == 18) {
				   level.behavior = 1;
				    level.level_type = 2;
				 
				 }
				   
				   
				   
				   
			
 } 

            let output = File::create(output).unwrap();
            Level::to_file(output, levels);
        }
    }

}
