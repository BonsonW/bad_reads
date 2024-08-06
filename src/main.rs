use std::{env, fs::OpenOptions, io::{BufWriter, Write}, path::Path, process::exit};
use bad_reads::*;

#[cfg(test)]
mod tests;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    
    let subtool = &args[1];
    
    let mut subtool_args = Vec::new();
    for i in 2..args.len() {
        subtool_args.push(args[i].clone());
    }
    
    match subtool.as_str() {
        "get" => {
            get_main(subtool_args);
        }
        "filter" => {
            filter_main(subtool_args);
        }
        _ => {
            println!("available subtools: get | filter");
            exit(1);
        }
    }
}

fn get_main(args: Vec<String>) {
    if args.len() != 5 {
        println!("usage: bad_reads get <slow5_file path> <scan_data_file path> <out_file path> <pore_state> <read_mode>");
        exit(1);
    }
    
    let slow5_fpath = Path::new(&args[0]);
    let scan_data_fpath = Path::new(&args[1]);
    let out_fpath = Path::new(&args[2]);
    
    let pore_state_arg = &args[3];
    let read_mode_arg = &args[4];
    
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

fn filter_main(args: Vec<String>) {
    if args.len() != 4 {
        println!("usage: bad_reads filter <read_ids path> <slow5_file path> <out_file path> <filter_mode>");
        exit(1);
    }
    
    let read_ids_fpath = Path::new(&args[0]);
    let slow5_fpath = Path::new(&args[1]);
    let out_fpath = Path::new(&args[2]);
    let read_mode_arg = &args[3];
    
    let filter_mode = match read_mode_arg.as_str() {
        "odd" => FilterMode::Odd,
        "even" => FilterMode::Even,
        _ => {
            println!("valid modes: <odd> | <even>");
            exit(1);
        }
    };
    
    if !read_ids_fpath.exists() {
        println!("invalid read_list path");
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
    
    println!("filtering reads...");
    let filtered_reads = filter_reads(read_ids_fpath, slow5_fpath, filter_mode);
    
    println!("writing read_ids into file...");
    let mut out_file = BufWriter::new(out_file);
    
    for bad_read in filtered_reads.into_iter() {
        writeln!(out_file, "{}", bad_read).expect("error writing read_id to out file");
    }
    
    println!("all done!");
}
