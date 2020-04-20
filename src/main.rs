use serde::{Serialize, Deserialize};
use std::fs::File;
use std::path::PathBuf;
use std::io::{Read, Write};
use byteorder::{BigEndian, ByteOrder};
use structopt::StructOpt;

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
    unk10: f32,
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
            unk10: BigEndian::read_f32(&input[36..]),
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
        BigEndian::write_f32(&mut output[36..], self.unk10);
        BigEndian::write_f32(&mut output[40..], self.head_size);
        BigEndian::write_f32(&mut output[44..], self.unk12);
        BigEndian::write_f32(&mut output[48..], self.unk13);
        BigEndian::write_f32(&mut output[52..], self.unk14);
        BigEndian::write_f32(&mut output[56..], self.unk15);
        BigEndian::write_f32(&mut output[60..], self.unk16);
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
    }
}

fn main() {
    let args = Args::from_args();

    match args {
        Args::Assemble {input, output} => {
            let mut input_file = File::open(&input).unwrap();
            let levels: Vec<Level> = serde_json::from_reader(input_file).unwrap();

            let output = output.unwrap_or_else(|| input.with_extension("bin"));
            let mut output = File::create(output).unwrap();

            let mut lvl_bytes = [0u8;64];
            for level in levels {
                level.to_bytes(&mut lvl_bytes);
                output.write_all(&lvl_bytes);
            }
        },
        Args::Disassemble {compact, input, output} => {
            let mut input_file = File::open(&input).unwrap();

            let mut lvl_bytes = [0u8;64];
            let mut levels: Vec<Level> = Vec::new();
            while input_file.read_exact(&mut lvl_bytes).is_ok() {
                levels.push(Level::from_bytes(&lvl_bytes));
            }

            let output = output.unwrap_or_else(|| input.with_extension("json"));
            let output = File::create(output).unwrap();
            if compact {
                serde_json::to_writer(output, &levels).unwrap();
            } else {
                serde_json::to_writer_pretty(output, &levels).unwrap();
            }
        }
    }

}
