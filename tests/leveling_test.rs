use bot_leveling_rust::game::proto::*;

#[test]
fn test_leveling_proto_tower_request() {
    let packet = build_tower_request(101, true, true);
    assert!(!packet.is_empty());
    // Byte 0 has field 1 wire type 0: (1 << 3) | 0 = 0x08
    assert_eq!(packet[0], 0x08);
    // Value of req_id = 101
    assert_eq!(packet[1], 101);
}

#[test]
fn test_leveling_proto_talent_allocation() {
    let packet = build_allocate_talent(202, "phys_atk", 5);
    assert!(!packet.is_empty());
    // Must contain "phys_atk" string bytes
    let str_bytes = "phys_atk".as_bytes();
    let found = packet.windows(str_bytes.len()).any(|w| w == str_bytes);
    assert!(found);
}

#[test]
fn test_leveling_proto_gem_combine() {
    let packet = build_combine_gems(303, 2);
    assert!(!packet.is_empty());
    assert_eq!(packet[0], 0x08);
}
