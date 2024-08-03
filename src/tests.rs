use super::*;

#[test]
fn one_read_one_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 1.0, bad: true, ..Default::default() }
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(
        ReadTimestamp { read_id: "a".into(), secs_start: 0.0, channel: 0, pore: 0 }
    );
    
    let bad_reads = get_bad_reads(pore_mux_map, &read_timestamps);
    
    assert!(!bad_reads.is_empty());
}

#[test]
fn one_read_after_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 0.0, bad: true, ..Default::default() }
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(
        ReadTimestamp { read_id: "a".into(), secs_start: 1.0, channel: 0, pore: 0 }
    );
    
    let bad_reads = get_bad_reads(pore_mux_map, &read_timestamps);
    
    assert!(bad_reads.is_empty());
}

#[test]
fn two_read_one_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 2.0, bad: true, ..Default::default() }
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 0.0, channel: 0, pore: 0 });
    read_timestamps.push(ReadTimestamp { read_id: "b".into(), secs_start: 1.0, channel: 0, pore: 0 });
    
    let bad_reads = get_bad_reads(pore_mux_map, &read_timestamps);
    
    assert!(!bad_reads.is_empty());
    assert!(bad_reads.len() == 1);
    assert!(bad_reads[0] == "b");
}

#[test]
fn two_read_two_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 1.0, bad: true, ..Default::default() },
                MuxStat { secs_start: 3.0, bad: true, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 0.0, channel: 0, pore: 0 });
    read_timestamps.push(ReadTimestamp { read_id: "b".into(), secs_start: 2.0, channel: 0, pore: 0 });
    
    let bad_reads = get_bad_reads(pore_mux_map, &read_timestamps);
    
    assert!(!bad_reads.is_empty());
    assert!(bad_reads.len() == 2);
    assert!(bad_reads[0] == "a");
    assert!(bad_reads[1] == "b");
}

#[test]
fn one_bad_read_good_then_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 1.0, bad: false, ..Default::default() },
                MuxStat { secs_start: 3.0, bad: true, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 0.0, channel: 0, pore: 0 });
    
    let bad_reads = get_bad_reads(pore_mux_map, &read_timestamps);
    
    assert!(bad_reads.is_empty());
}

#[test]
fn good_mux_before_read_then_bad() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 1.0, bad: false, ..Default::default() },
                MuxStat { secs_start: 3.0, bad: true, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 2.0, channel: 0, pore: 0 });
    
    let bad_reads = get_bad_reads(pore_mux_map, &read_timestamps);
    
    assert!(!bad_reads.is_empty());
}

#[test]
fn read_good_mux_read_bad_mux() {
    let mut pore_mux_map = HashMap::new();
    let mut read_timestamps = Vec::new();
    
    pore_mux_map.insert((0, 0),
        PoreMuxStats {
            muxs: vec![
                MuxStat { secs_start: 1.0, bad: false, ..Default::default() },
                MuxStat { secs_start: 3.0, bad: true, ..Default::default() },
            ],
            ..Default::default()
        }
    );
    
    read_timestamps.push(ReadTimestamp { read_id: "a".into(), secs_start: 0.0, channel: 0, pore: 0 });
    read_timestamps.push(ReadTimestamp { read_id: "b".into(), secs_start: 2.0, channel: 0, pore: 0 });
    
    let bad_reads = get_bad_reads(pore_mux_map, &read_timestamps);
    
    assert!(!bad_reads.is_empty());
    assert!(bad_reads.len() == 1);
    assert!(bad_reads[0] == "b");
}