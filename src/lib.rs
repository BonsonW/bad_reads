use std::{collections::HashMap, fs::read_to_string, path::Path};

use slow5::{FileReader, RecordExt};

#[derive(Default, Clone)]
pub struct PoreMuxStats<'a> {
    pub muxs: Vec<MuxStat<'a>>,
    pub last_entry: usize,
}

#[derive(Default, Clone, Copy)]
pub struct MuxStat<'a> {
    pub secs_start: f64,
    pub read_secs_start: f64,
    pub read_id: Option<&'a String>,
    pub pore_state: PoreState
}

pub struct ReadTimestamp {
    pub read_id: String,
    pub secs_start: f64,
    pub channel: u32,
    pub pore: u8,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum PoreState {
    #[default]
    Dead,
    Alive,
}

pub enum ReadMode {
    First,
    Last,
}

pub enum FilterMode {
    Odd,
    Even,
}

pub fn gen_read_timestamps(slow5_fpath: &Path) -> Vec<ReadTimestamp> {
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

pub fn gen_pore_mux_map(scan_data_fpath: &Path) -> HashMap<(u32, u8), PoreMuxStats> {
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

pub fn get_last_read<'a>(mut pore_mux_map: HashMap<(u32, u8), PoreMuxStats<'a>>, read_timestamps: &'a Vec<ReadTimestamp>, pore_state: PoreState) -> Vec<&'a String> {
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

pub fn get_first_read<'a>(mut pore_mux_map: HashMap<(u32, u8), PoreMuxStats<'a>>, read_timestamps: &'a Vec<ReadTimestamp>, pore_state: PoreState) -> Vec<&'a String> {
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

pub fn filter_reads(read_ids_fpath: &Path, slow5_fpath: &Path, filter_mode: FilterMode) -> Vec<String> {
    let mut ret = Vec::new();
    
    let slow5 = FileReader::open(slow5_fpath).expect("could not open slow5");
    
    for read_id in read_to_string(read_ids_fpath).unwrap().lines() {
        let rec = slow5.get_record(read_id).expect("invalid read_id provided");
        let channel = rec.get_aux_field::<&str>("channel_number").expect("could not load aux_field `channel_number`");
        let channel = channel.parse::<u32>().expect("could not parse channel_number as u32");
        
        match filter_mode {
            FilterMode::Odd => {
                if channel % 2 != 0 { ret.push(read_id.into()); }
            },
            FilterMode::Even => {
                if channel % 2 == 0 { ret.push(read_id.into()); }
            },
        }
    }

    ret
}
