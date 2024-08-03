use std::{collections::HashMap, env, fs::{read_to_string, OpenOptions}, io::{BufWriter, Write}, path::Path, process::exit};

use slow5::{FileReader, RecordExt};

#[cfg(test)]
mod tests;

#[derive(Default)]
struct PoreMuxs<'a> {
    muxs: Vec<BadMux<'a>>,
    last_entry: usize,
}

#[derive(Default)]
struct BadMux<'a> {
    secs_start: f64,
    last_read_secs_start: f64,
    last_read_id: Option<&'a String>,
}

struct ReadTimestamp {
    read_id: String,
    secs_start: f64,
    channel: u32,
    pore: u8,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 4 {
        println!("usage: bad_reads <slow5_file path> <scan_data_file path> <out_file path>");
        exit(0);
    }
    
    let slow5_fpath = Path::new(&args[1]);
    let scan_data_fpath = Path::new(&args[2]);
    let out_fpath = Path::new(&args[3]);
    
    if !scan_data_fpath.exists() {
        panic!("invalid scan_data path");
    }
    
    if !slow5_fpath.exists() {
        panic!("invalid slow5 path");
    }
    
    let out_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .append(true)
        .open(out_fpath)
        .expect("could not open out file");
        
    println!("reading mux scan data...");
    let pore_mux_map = get_scan_data(scan_data_fpath);

    println!("generating slow5 rec timestamps...");
    let read_timestamps = get_read_timestamps(slow5_fpath);
    
    println!("fetching bad reads...");
    let bad_reads = get_bad_reads(pore_mux_map, &read_timestamps);
    
    println!("writing read_ids into file...");
    let mut out_file = BufWriter::new(out_file);
    
    for bad_read in bad_reads.into_iter() {
        writeln!(out_file, "{}", bad_read).expect("error writing read_id to out file");
    }
    
    println!("all done!");
}

fn get_read_timestamps(slow5_fpath: &Path) -> Vec<ReadTimestamp> {
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

fn get_scan_data(scan_data_fpath: &Path) -> HashMap<(u32, u8), PoreMuxs> {
    let mut ret = HashMap::new();
    
    let mux_stat_idx = 26;
    let channel_idx = 0;
    let pore_idx = 1;
    let mux_secs_start_idx = 36;
    
    for line in read_to_string(scan_data_fpath).unwrap().lines().skip(1) {
        let csv_entry = line.split(',').collect::<Vec<&str>>();
        if csv_entry[mux_stat_idx] != "single_pore" {
            let channel = csv_entry[channel_idx].parse::<u32>().expect("could not parse channel col");
            let pore = csv_entry[pore_idx].parse::<u8>().expect("could not parse pore col");
            let key = (channel, pore);

            let secs_start = csv_entry[mux_secs_start_idx].parse::<f64>().expect("could not parse start time col");
            
            let pore_muxs = ret.entry(key).or_insert(PoreMuxs::default());
            
            pore_muxs.muxs.push(BadMux {
                secs_start,
                ..Default::default()
            });
        }
    }

    ret
}

fn get_bad_reads<'a>(mut pore_mux_map: HashMap<(u32, u8), PoreMuxs<'a>>, read_timestamps: &'a Vec<ReadTimestamp>) -> Vec<&'a String> {
    let mut ret = Vec::new();
    
    for ts in read_timestamps.iter() {
        let pore_muxs = pore_mux_map.get_mut(&(ts.channel, ts.pore));
        if pore_muxs.is_none() { continue; }
        let pore_muxs = pore_muxs.unwrap();
        
        for i in pore_muxs.last_entry..pore_muxs.muxs.len() {
            let bad_mux = pore_muxs.muxs.get_mut(i).expect("error indexing channel_muxs");
            if ts.secs_start >= bad_mux.secs_start {
                pore_muxs.last_entry = i+1;
                continue;
            }
            
            bad_mux.last_read_id = Some(&ts.read_id);
            bad_mux.last_read_secs_start = ts.secs_start;
            
            pore_muxs.last_entry = i;
            break;
        }
    }
    
    for pore_muxs in pore_mux_map.values() {
        for bad_mux in pore_muxs.muxs.iter() {
            if let Some(read_id) = bad_mux.last_read_id {
                ret.push(read_id);
            }
        }
    }
    
    ret
}