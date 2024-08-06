use std::{collections::HashMap, env, fs::{read_to_string, OpenOptions}, io::{BufWriter, Write}, path::Path, process::exit};

use slow5::{FileReader, RecordExt};

#[cfg(test)]
mod tests;

#[derive(Default, Clone)]
struct PoreMuxStats<'a> {
    muxs: Vec<MuxStat<'a>>,
    last_entry: usize,
}

#[derive(Default, Clone, Copy)]
struct MuxStat<'a> {
    secs_start: f64,
    read_secs_start: f64,
    read_id: Option<&'a String>,
    pore_state: PoreState
}

struct ReadTimestamp {
    read_id: String,
    secs_start: f64,
    channel: u32,
    pore: u8,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum PoreState {
    #[default]
    Dead,
    Alive,
}

enum ReadMode {
    First,
    Last,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 6 {
        println!("usage: bad_reads <slow5_file path> <scan_data_file path> <out_file path> <pore_state> <read_mode>");
        exit(1);
    }
    
    let slow5_fpath = Path::new(&args[1]);
    let scan_data_fpath = Path::new(&args[2]);
    let out_fpath = Path::new(&args[3]);
    
    let pore_state_arg = &args[4];
    let read_mode_arg = &args[5];
    
    let pore_state = match pore_state_arg.as_str() {
        "dead" => PoreState::Dead,
        "alive" => PoreState::Alive,
        _ => {
            println!("valid pore_states: <dead> | <alive>");
            exit(1);
        }
    };
    
    let read_mode = match read_mode_arg.as_str() {
        "first" => ReadMode::First,
        "last" => ReadMode::Last,
        _ => {
            println!("valid modes: <first> | <last>");
            exit(1);
        }
    };
    
    if !scan_data_fpath.exists() {
        println!("invalid scan_data path");
        exit(1);
    }
    
    if !slow5_fpath.exists() {
        println!("invalid slow5 path");
        exit(1);
    }
    
    let out_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .append(true)
        .open(out_fpath)
        .expect("could not open out file");
        
    println!("reading mux scan data...");
    let pore_mux_map = gen_pore_mux_map(scan_data_fpath);

    println!("generating slow5 read timestamps...");
    let read_timestamps = gen_read_timestamps(slow5_fpath);
    
    println!("fetching reads...");
    let bad_reads = match read_mode {
        ReadMode::First => get_first_read(pore_mux_map, &read_timestamps, pore_state),
        ReadMode::Last => get_last_read(pore_mux_map, &read_timestamps, pore_state),
    };
    
    println!("writing read_ids into file...");
    let mut out_file = BufWriter::new(out_file);
    
    for bad_read in bad_reads.into_iter() {
        writeln!(out_file, "{}", bad_read).expect("error writing read_id to out file");
    }
    
    println!("all done!");
}

fn gen_read_timestamps(slow5_fpath: &Path) -> Vec<ReadTimestamp> {
    let mut ret = Vec::new();
    
    let mut slow5 = FileReader::open(slow5_fpath).expect("could not open slow5");
    for rec in slow5.records() {
        if rec.is_err() {
            println!("error reading record {:?}, skipping...", rec.err());
            continue;
        }
        let rec = rec.unwrap();
        
        let channel = rec.get_aux_field::<&str>("channel_number").expect("could not load aux_field `channel_number`");
        let channel = channel.parse::<u32>().expect("could not parse channel_number as u32");
        let pore = rec.get_aux_field::<u8>("start_mux").expect("could not load aux_field `start_mux`");
        
        let samples_start = rec.get_aux_field::<u64>("start_time").expect("could not load aux_field `start_time`");
        let secs_start = samples_start as f64 / rec.sampling_rate();

        ret.push(ReadTimestamp {
            read_id: String::from_utf8(rec.read_id().to_vec()).expect("could not get read_id from rec"),
            secs_start,
            channel,
            pore
        })
    }
    
    ret.sort_by(|a, b| a.secs_start.partial_cmp(&b.secs_start).unwrap());

    ret
}

fn gen_pore_mux_map(scan_data_fpath: &Path) -> HashMap<(u32, u8), PoreMuxStats> {
    let mut ret = HashMap::new();
    
    let mux_stat_col = 26;
    let channel_col = 0;
    let pore_col = 1;
    let mux_secs_start_col = 36;
    
    for line in read_to_string(scan_data_fpath).unwrap().lines().skip(1) {
        let csv_entry = line.split(',').collect::<Vec<&str>>();
        let secs_start = csv_entry[mux_secs_start_col].parse::<f64>().expect("could not parse start time col");
        
        let channel = csv_entry[channel_col].parse::<u32>().expect("could not parse channel col");
        let pore = csv_entry[pore_col].parse::<u8>().expect("could not parse pore col");
        let key = (channel, pore);
        
        let pore_muxs = ret.entry(key).or_insert(PoreMuxStats::default());

        if csv_entry[mux_stat_col] == "single_pore" {
            pore_muxs.muxs.push(MuxStat {
                secs_start,
                pore_state: PoreState::Alive,
                ..Default::default()
            });
        } else {
            pore_muxs.muxs.push(MuxStat {
                secs_start,
                pore_state: PoreState::Dead,
                ..Default::default()
            });
        }
    }

    ret
}

fn get_last_read<'a>(mut pore_mux_map: HashMap<(u32, u8), PoreMuxStats<'a>>, read_timestamps: &'a Vec<ReadTimestamp>, pore_state: PoreState) -> Vec<&'a String> {
    let mut ret = Vec::new();
    
    for ts in read_timestamps.iter() {
        let pore_muxs = pore_mux_map.get_mut(&(ts.channel, ts.pore));
        if pore_muxs.is_none() { continue; }
        let pore_muxs = pore_muxs.unwrap();
        
        for i in pore_muxs.last_entry..pore_muxs.muxs.len() {
            let muxstat = pore_muxs.muxs.get_mut(i).expect("error indexing channel_muxs");
            
            if ts.secs_start < muxstat.secs_start {
                if pore_state != muxstat.pore_state {
                    break;
                }
            } else {
                pore_muxs.last_entry = i+1;
                continue;
            }
            
            muxstat.read_id = Some(&ts.read_id);
            muxstat.read_secs_start = ts.secs_start;
            
            pore_muxs.last_entry = i;
            break;
        }
    }
    
    for pore_muxs in pore_mux_map.values() {
        for muxstat in pore_muxs.muxs.iter() {
            if let Some(read_id) = muxstat.read_id {
                ret.push(read_id);
            }
        }
    }
    
    ret
}

fn get_first_read<'a>(mut pore_mux_map: HashMap<(u32, u8), PoreMuxStats<'a>>, read_timestamps: &'a Vec<ReadTimestamp>, pore_state: PoreState) -> Vec<&'a String> {
    let mut ret = Vec::new();
    
    for ts in read_timestamps.iter().rev() {
        let pore_muxs = pore_mux_map.get_mut(&(ts.channel, ts.pore));
        if pore_muxs.is_none() { continue; }
        let pore_muxs = pore_muxs.unwrap();
        
        for i in (0..(pore_muxs.muxs.len() - pore_muxs.last_entry)).rev() {
            let muxstat = pore_muxs.muxs.get_mut(i).expect("error indexing channel_muxs");
            
            if ts.secs_start > muxstat.secs_start {
                if pore_state != muxstat.pore_state {
                    break;
                }
            } else {
                pore_muxs.last_entry = i+1;
                continue;
            }
            
            muxstat.read_id = Some(&ts.read_id);
            muxstat.read_secs_start = ts.secs_start;
            
            pore_muxs.last_entry = i;
            break;
        }
    }
    
    for pore_muxs in pore_mux_map.values() {
        for muxstat in pore_muxs.muxs.iter() {
            if let Some(read_id) = muxstat.read_id {
                ret.push(read_id);
            }
        }
    }
    
    ret
}