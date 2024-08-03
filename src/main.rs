use std::{collections::HashMap, env, fs::{read_to_string, OpenOptions}, io::{BufWriter, Write}, path::Path, process::exit};

use slow5::{FileReader, RecordExt};

#[cfg(test)]
mod tests;

#[derive(Default)]
struct ChannelMuxs<'a> {
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
    well: u8,
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
    
    println!("generating slow5 rec timestamps...");
    let mut read_timestamps = get_read_timestamps(slow5_fpath);
    
    // sort by start time
    read_timestamps.sort_by_key(|ts| {
        ts.secs_start;
    });
    
    println!("fetching bad reads...");
    let bad_reads = get_bad_reads(scan_data_fpath, &read_timestamps);
    
    // write read-ids to file
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
        let well = rec.get_aux_field::<u8>("start_mux").expect("could not load aux_field `start_mux`");
        
        let samples_start = rec.get_aux_field::<u64>("start_time").expect("could not load aux_field `start_time`");
        let secs_start = samples_start as f64 / rec.sampling_rate();

        ret.push(ReadTimestamp {
            read_id: String::from_utf8(rec.read_id().to_vec()).expect("could not get read_id from rec"),
            secs_start,
            channel,
            well
        })
    }

    ret
}

fn get_bad_reads<'a>(scan_data_fpath: &Path, read_timestamps: &'a Vec<ReadTimestamp>) -> Vec<&'a String> {
    let mut ret = Vec::new();
    
    let mut bad_channels = HashMap::new();
    
    let mux_stat_idx = 26;
    let channel_idx = 0;
    let well_idx = 1;
    let mux_secs_start_idx = 36;
    
    // get every bad mux scan on a channel
    println!("reading mux scan data...");
    for line in read_to_string(scan_data_fpath).unwrap().lines().skip(1) {
        let csv_entry = line.split(',').collect::<Vec<&str>>();
        if csv_entry[mux_stat_idx] != "single_pore" {
            let channel = csv_entry[channel_idx].parse::<u32>().expect("could not parse channel col");
            let well = csv_entry[well_idx].parse::<u8>().expect("could not parse channel col");
            let key = (channel, well);

            let secs_start = csv_entry[mux_secs_start_idx].parse::<f64>().expect("could not parse start time col");
            
            let cmuxs = bad_channels.entry(key).or_insert(ChannelMuxs::default());
            
            cmuxs.muxs.push(BadMux {
                secs_start,
                ..Default::default()
            });
        }
    }
    
    // update bad channel entries
    println!("inserting timestamps into channel entries...");
    for ts in read_timestamps.iter() {
        let cmuxs = bad_channels.get_mut(&(ts.channel, ts.well));
        if cmuxs.is_none() { continue; }
        let cmuxs = cmuxs.unwrap();
        
        for i in cmuxs.last_entry..cmuxs.muxs.len() {
            let bad_mux = cmuxs.muxs.get_mut(i).expect("error indexing channel_muxs");
            if ts.secs_start >= bad_mux.secs_start {
                cmuxs.last_entry = i+1;
                continue;
            }
            
            bad_mux.last_read_id = Some(&ts.read_id);
            bad_mux.last_read_secs_start = ts.secs_start;
            
            cmuxs.last_entry = i;
            break;
        }
    }
    
    for cmuxs in bad_channels.values() {
        for bad_mux in cmuxs.muxs.iter() {
            if let Some(read_id) = bad_mux.last_read_id {
                ret.push(read_id);
            }
        }
    }
    
    ret
}