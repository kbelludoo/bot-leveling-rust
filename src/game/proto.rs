pub const WIRE_VARINT: u32 = 0;
pub const WIRE_BYTES: u32 = 2;

pub fn append_tag(buf: &mut Vec<u8>, field_number: u32, wire_type: u32) {
    let key = (field_number << 3) | wire_type;
    append_varint(buf, key as u64);
}

pub fn append_varint(buf: &mut Vec<u8>, mut val: u64) {
    while val >= 0x80 {
        buf.push(((val & 0x7F) | 0x80) as u8);
        val >>= 7;
    }
    buf.push((val & 0x7F) as u8);
}

pub fn append_bytes(buf: &mut Vec<u8>, bytes: &[u8]) {
    append_varint(buf, bytes.len() as u64);
    buf.extend_from_slice(bytes);
}

pub fn append_string(buf: &mut Vec<u8>, s: &str) {
    append_bytes(buf, s.as_bytes());
}

pub fn wrap_client_msg(req_id: u64, opcode_tag: u32, payload: &[u8]) -> Vec<u8> {
    let mut msg = Vec::new();
    append_tag(&mut msg, 1, WIRE_VARINT);
    append_varint(&mut msg, req_id);
    append_tag(&mut msg, opcode_tag, WIRE_BYTES);
    append_bytes(&mut msg, payload);
    msg
}

// 1. Talentos (Tag 92)
pub fn build_allocate_talent(req_id: u64, node_id: &str, points: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_BYTES);
    append_string(&mut payload, node_id);
    append_tag(&mut payload, 2, WIRE_VARINT);
    append_varint(&mut payload, points as u64);
    wrap_client_msg(req_id, 92, &payload)
}

// 2. Torre dos Desafios (Tag 370)
pub fn build_tower_request(req_id: u64, start: bool, auto_advance: bool) -> Vec<u8> {
    let mut settings = Vec::new();
    if auto_advance {
        append_tag(&mut settings, 1, WIRE_VARINT);
        append_varint(&mut settings, 1);
    }

    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_VARINT);
    append_varint(&mut payload, 1); // info = true
    if start {
        append_tag(&mut payload, 2, WIRE_VARINT);
        append_varint(&mut payload, 1);
    }
    if !settings.is_empty() {
        append_tag(&mut payload, 3, WIRE_BYTES);
        append_bytes(&mut payload, &settings);
    }

    wrap_client_msg(req_id, 370, &payload)
}

// 3. Boss Diário (Tag 39)
pub fn build_daily_boss(req_id: u64) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_VARINT);
    append_varint(&mut payload, 1);
    wrap_client_msg(req_id, 39, &payload)
}

// 4. Masmorras (Tag 40)
pub fn build_enter_dungeon(req_id: u64, dungeon_id: &str, floor: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_BYTES);
    append_string(&mut payload, dungeon_id);
    if floor > 0 {
        append_tag(&mut payload, 2, WIRE_VARINT);
        append_varint(&mut payload, floor as u64);
    }
    wrap_client_msg(req_id, 40, &payload)
}

// 5. Gems (Tag 85)
pub fn build_combine_gems(req_id: u64, tier: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_VARINT);
    append_varint(&mut payload, tier as u64);
    append_tag(&mut payload, 2, WIRE_VARINT);
    append_varint(&mut payload, 1); // combine_all
    wrap_client_msg(req_id, 85, &payload)
}

// 6. Encantamento (Tag 80)
pub fn build_enchant(req_id: u64, slot: u32, safe_only: bool) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_VARINT);
    append_varint(&mut payload, slot as u64);
    if safe_only {
        append_tag(&mut payload, 2, WIRE_VARINT);
        append_varint(&mut payload, 1);
    }
    wrap_client_msg(req_id, 80, &payload)
}
