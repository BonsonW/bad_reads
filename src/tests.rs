use std::collections::HashMap;

use super::*;

#[test]
fn filter_reads_odd() {
    let read_ids_fpath = Path::new("test_data/rand_readids_5.txt");
    let slow5_fpath = Path::new("test_data/rand_reads_5.blow5");
    let read_ids = filter_reads(read_ids_fpath, slow5_fpath, FilterMode::Odd);
    
    assert!(read_ids.len() == 2);
    assert!(read_ids[0] == "8bfec45c-b89e-4510-9469-e94bb415b8e4");
    assert!(read_ids[1] == "503f0bd8-3a00-4c76-9f2e-c70ada3d418b");
}

#[test]
fn filter_reads_even() {
    let read_ids_fpath = Path::new("test_data/rand_readids_5.txt");
    let slow5_fpath = Path::new("test_data/rand_reads_5.blow5");
    let read_ids = filter_reads(read_ids_fpath, slow5_fpath, FilterMode::Even);
    
    assert!(read_ids.len() == 3);
    assert!(read_ids[0] == "d62da1d5-971e-4e5d-9465-5715300e8523");
    assert!(read_ids[1] == "d56f390f-2e33-436e-9220-a93aca7dd11b");
    assert!(read_ids[2] == "76b715cd-aaea-4ae1-8026-41c1772597ed");
}

#[test]
fn read_timestamps() {
    let read_timestamps = gen_read_timestamps(Path::new("test_data/rand_reads_5.blow5"));
    
    assert!(read_timestamps[0].read_id == "d62da1d5-971e-4e5d-9465-5715300e8523");
    assert!(read_timestamps[0].secs_start == (107652553 as f64 / 4000 as f64));
    assert!(read_timestamps[0].channel == 416);
    assert!(read_timestamps[0].pore == 4);
    
    assert!(read_timestamps[1].read_id == "8bfec45c-b89e-4510-9469-e94bb415b8e4");
    assert!(read_timestamps[1].secs_start == (114242867 as f64 / 4000 as f64));
    assert!(read_timestamps[1].channel == 333);
    assert!(read_timestamps[1].pore == 4);
    
    assert!(read_timestamps[2].read_id == "d56f390f-2e33-436e-9220-a93aca7dd11b");
    assert!(read_timestamps[2].secs_start == (119263451 as f64 / 4000 as f64));
    assert!(read_timestamps[2].channel == 348);
    assert!(read_timestamps[2].pore == 2);
    
    assert!(read_timestamps[3].read_id == "503f0bd8-3a00-4c76-9f2e-c70ada3d418b");
    assert!(read_timestamps[3].secs_start == (212162126 as f64 / 4000 as f64));
    assert!(read_timestamps[3].channel == 187);
    assert!(read_timestamps[3].pore == 2);
    
    assert!(read_timestamps[4].read_id == "76b715cd-aaea-4ae1-8026-41c1772597ed");
    assert!(read_timestamps[4].secs_start == (248115103 as f64 / 4000 as f64));
    assert!(read_timestamps[4].channel == 266);
    assert!(read_timestamps[4].pore == 1);
}

#[test]
fn pore_mux_map() {
    let pore_mux_map = gen_pore_mux_map(Path::new("test_data/pore_scan_test_data.csv"));
    
    let c1p1 = pore_mux_map.get(&(1, 1)).expect("could not get pore entry");
    assert!(c1p1.muxs[0].pore_state == PoreState::Alive);
    assert!(c1p1.muxs[0].secs_start == 1.into());
    assert!(c1p1.muxs[1].pore_state == PoreState::Dead);
    assert!(c1p1.muxs[1].secs_start == 2.into());
    
    let c1p2 = pore_mux_map.get(&(1, 2)).expect("could not get pore entry");
    assert!(c1p2.muxs[0].pore_state == PoreState::Dead);
    assert!(c1p2.muxs[0].secs_start == 1.into());
    assert!(c1p2.muxs[1].pore_state == PoreState::Dead);
    assert!(c1p2.muxs[1].secs_start == 2.into());
}

#[test]
fn one_read_one_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 1.0, pore_state: PoreState::Dead, ..Default::default() }
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(
        ReadTimestamp { read_id: "a".into(), secs_start: 0.0, channel: 0, pore: 0 }
    );
    
    let reads = get_last_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    assert!(!reads.is_empty());
    
    let reads = get_first_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    assert!(reads.is_empty());
}

#[test]
fn one_read_after_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 0.0, pore_state: PoreState::Dead, ..Default::default() }
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(
        ReadTimestamp { read_id: "a".into(), secs_start: 1.0, channel: 0, pore: 0 }
    );
    
    let reads = get_last_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    assert!(reads.is_empty());
    
    let reads = get_first_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    assert!(!reads.is_empty());
}

#[test]
fn two_read_one_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 2.0, pore_state: PoreState::Dead, ..Default::default() }
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 0.0, channel: 0, pore: 0 });
    read_timestamps.push(ReadTimestamp { read_id: "b".into(), secs_start: 1.0, channel: 0, pore: 0 });
    
    let reads = get_last_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    
    assert!(!reads.is_empty());
    assert!(reads.len() == 1);
    assert!(reads[0] == "b");
}

#[test]
fn one_bad_mux_two_read() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 0.0, pore_state: PoreState::Dead, ..Default::default() }
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 1.0, channel: 0, pore: 0 });
    read_timestamps.push(ReadTimestamp { read_id: "b".into(), secs_start: 2.0, channel: 0, pore: 0 });
    
    let reads = get_first_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    
    assert!(!reads.is_empty());
    assert!(reads.len() == 1);
    assert!(reads[0] == "a");
}

#[test]
fn two_read_two_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 1.0, pore_state: PoreState::Dead, ..Default::default() },
                MuxStat { secs_start: 3.0, pore_state: PoreState::Dead, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 0.0, channel: 0, pore: 0 });
    read_timestamps.push(ReadTimestamp { read_id: "b".into(), secs_start: 2.0, channel: 0, pore: 0 });
    
    let reads = get_last_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    
    assert!(!reads.is_empty());
    assert!(reads.len() == 2);
    assert!(reads[0] == "a");
    assert!(reads[1] == "b");
}

#[test]
fn two_bad_mux_two_read() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 0.0, pore_state: PoreState::Dead, ..Default::default() },
                MuxStat { secs_start: 2.0, pore_state: PoreState::Dead, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 1.0, channel: 0, pore: 0 });
    read_timestamps.push(ReadTimestamp { read_id: "b".into(), secs_start: 3.0, channel: 0, pore: 0 });
    
    let reads = get_first_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    
    assert!(!reads.is_empty());
    assert!(reads.len() == 2);
    assert!(reads[0] == "a");
    assert!(reads[1] == "b");
}

#[test]
fn one_bad_read_good_then_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 1.0, pore_state: PoreState::Alive, ..Default::default() },
                MuxStat { secs_start: 3.0, pore_state: PoreState::Dead, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 0.0, channel: 0, pore: 0 });
    
    let reads = get_last_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    
    assert!(reads.is_empty());
}

#[test]
fn bad_then_good_mux_one_bad_read() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 0.0, pore_state: PoreState::Dead, ..Default::default() },
                MuxStat { secs_start: 1.0, pore_state: PoreState::Alive, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 2.0, channel: 0, pore: 0 });
    
    let reads = get_first_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    
    assert!(reads.is_empty());
}

#[test]
fn good_mux_before_read_then_bad() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 1.0, pore_state: PoreState::Alive, ..Default::default() },
                MuxStat { secs_start: 3.0, pore_state: PoreState::Dead, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 2.0, channel: 0, pore: 0 });
    
    let reads = get_last_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    
    assert!(!reads.is_empty());
}

#[test]
fn bad_mux_before_read_then_good() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 1.0, pore_state: PoreState::Dead, ..Default::default() },
                MuxStat { secs_start: 3.0, pore_state: PoreState::Alive, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 2.0, channel: 0, pore: 0 });
    
    let reads = get_first_read(pore_mux_map.clone(), &read_timestamps, PoreState::Dead);
    
    assert!(!reads.is_empty());
}

#[test]
fn read_good_mux_read_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 1.0, pore_state: PoreState::Alive, ..Default::default() },
                MuxStat { secs_start: 3.0, pore_state: PoreState::Dead, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 0.0, channel: 0, pore: 0 });
    read_timestamps.push(ReadTimestamp { read_id: "b".into(), secs_start: 2.0, channel: 0, pore: 0 });
    
    let reads = get_last_read(pore_mux_map.clone().clone(), &read_timestamps, PoreState::Dead);
    
    assert!(!reads.is_empty());
    assert!(reads.len() == 1);
    assert!(reads[0] == "b");
}

#[test]
fn good_mux_read_bad_mux_read() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 0.0, pore_state: PoreState::Alive, ..Default::default() },
                MuxStat { secs_start: 2.0, pore_state: PoreState::Dead, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 1.0, channel: 0, pore: 0 });
    read_timestamps.push(ReadTimestamp { read_id: "b".into(), secs_start: 3.0, channel: 0, pore: 0 });
    
    let reads = get_first_read(pore_mux_map.clone().clone(), &read_timestamps, PoreState::Dead);
    
    assert!(!reads.is_empty());
    assert!(reads.len() == 1);
    assert!(reads[0] == "b");
}
