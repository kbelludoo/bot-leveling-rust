use std::collections::HashMap;

pub const WIRE_VARINT: u32 = 0;
pub const WIRE_FIXED64: u32 = 1;
pub const WIRE_BYTES: u32 = 2;
pub const WIRE_FIXED32: u32 = 5;

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

pub fn append_fixed32(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_le_bytes());
}

pub fn append_fixed64(buf: &mut Vec<u8>, val: u64) {
    buf.extend_from_slice(&val.to_le_bytes());
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

// 1. Auth (Tag 4)
pub fn build_auth_login(req_id: u64, user: &str, pass: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_BYTES);
    append_string(&mut payload, user);
    append_tag(&mut payload, 2, WIRE_BYTES);
    append_string(&mut payload, pass);
    wrap_client_msg(req_id, 4, &payload)
}

// 2. Ping (Tag 2)
pub fn build_ping(req_id: u64) -> Vec<u8> {
    wrap_client_msg(req_id, 2, &[])
}

// 3. StartHunt (Tag 30)
pub fn build_start_hunt(req_id: u64, gate_id: &str, tier_min: u32, difficulty: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    if !gate_id.is_empty() {
        append_tag(&mut payload, 1, WIRE_BYTES);
        append_string(&mut payload, gate_id);
    }
    if tier_min > 0 {
        append_tag(&mut payload, 2, WIRE_VARINT);
        append_varint(&mut payload, tier_min as u64);
    }
    if !difficulty.is_empty() {
        append_tag(&mut payload, 3, WIRE_BYTES);
        append_string(&mut payload, difficulty);
    }
    wrap_client_msg(req_id, 30, &payload)
}

// 4. LeaveHunt (Tag 31)
pub fn build_leave_hunt(req_id: u64) -> Vec<u8> {
    wrap_client_msg(req_id, 31, &[])
}

// 5. GiveUp (Tag 36)
pub fn build_give_up(req_id: u64) -> Vec<u8> {
    wrap_client_msg(req_id, 36, &[])
}

// 6. Revive (Tag 35)
pub fn build_revive(req_id: u64) -> Vec<u8> {
    wrap_client_msg(req_id, 35, &[])
}

// 7. EquipBest (Tag 54)
pub fn build_equip_best(req_id: u64) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_VARINT);
    append_varint(&mut payload, 2); // criterion = 2 (EQUIP_CRITERION_POWER)
    wrap_client_msg(req_id, 54, &payload)
}

// 8. RepairAll (Tag 65)
pub fn build_repair_all(req_id: u64) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 2, WIRE_VARINT);
    append_varint(&mut payload, 0); // partial = false
    wrap_client_msg(req_id, 65, &payload)
}

// 9. AllocateStats (Tag 90)
pub fn build_allocate_stats(req_id: u64, stat_amounts: &[(u32, u32)]) -> Vec<u8> {
    let mut payload = Vec::new();
    for &(stat_id, pts) in stat_amounts {
        let mut sa = Vec::new();
        append_tag(&mut sa, 1, WIRE_VARINT);
        append_varint(&mut sa, stat_id as u64);
        append_tag(&mut sa, 2, WIRE_VARINT);
        append_varint(&mut sa, pts as u64);

        append_tag(&mut payload, 1, WIRE_BYTES);
        append_bytes(&mut payload, &sa);
    }
    wrap_client_msg(req_id, 90, &payload)
}

// 10. AllocateTalent (Tag 92)
pub fn build_allocate_talent(req_id: u64, node_id: &str, points: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_BYTES);
    append_string(&mut payload, node_id);
    append_tag(&mut payload, 2, WIRE_VARINT);
    append_varint(&mut payload, points as u64);
    wrap_client_msg(req_id, 92, &payload)
}

// 11. ChooseClass (Tag 93)
pub fn build_choose_class(req_id: u64, class_id: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_BYTES);
    append_string(&mut payload, class_id);
    wrap_client_msg(req_id, 93, &payload)
}

// 12. SetAutomation (Tag 98)
pub fn build_set_automation(req_id: u64, level: u32) -> Vec<u8> {
    let mut hp_potion = Vec::new();
    append_tag(&mut hp_potion, 2, WIRE_FIXED32);
    append_fixed32(&mut hp_potion, 0x3f000000); // 0.5f
    append_tag(&mut hp_potion, 4, WIRE_VARINT);
    append_varint(&mut hp_potion, 1);

    let mut mana_potion = Vec::new();
    append_tag(&mut mana_potion, 2, WIRE_FIXED32);
    append_fixed32(&mut mana_potion, 0x3e99999a); // 0.3f
    append_tag(&mut mana_potion, 4, WIRE_VARINT);
    append_varint(&mut mana_potion, 1);

    let mut auto_msg = Vec::new();
    append_tag(&mut auto_msg, 2, WIRE_VARINT);
    append_varint(&mut auto_msg, 1); // auto_continue
    if level >= 15 {
        append_tag(&mut auto_msg, 5, WIRE_VARINT);
        append_varint(&mut auto_msg, 1); // auto_arise
    }
    append_tag(&mut auto_msg, 6, WIRE_VARINT);
    append_varint(&mut auto_msg, 1); // auto_chest
    append_tag(&mut auto_msg, 7, WIRE_BYTES);
    append_bytes(&mut auto_msg, &hp_potion);
    append_tag(&mut auto_msg, 8, WIRE_BYTES);
    append_bytes(&mut auto_msg, &mana_potion);
    if level >= 15 {
        append_tag(&mut auto_msg, 9, WIRE_VARINT);
        append_varint(&mut auto_msg, 1); // auto_shadow_potion
    }
    append_tag(&mut auto_msg, 10, WIRE_VARINT);
    append_varint(&mut auto_msg, 1); // auto_sell
    if level >= 15 {
        append_tag(&mut auto_msg, 16, WIRE_VARINT);
        append_varint(&mut auto_msg, 1); // auto_shadow_revive
    }

    let mut set_auto = Vec::new();
    append_tag(&mut set_auto, 1, WIRE_BYTES);
    append_bytes(&mut set_auto, &auto_msg);

    wrap_client_msg(req_id, 98, &set_auto)
}

// 13. Promotion (Tag 99)
pub fn build_start_promotion(req_id: u64) -> Vec<u8> {
    let mut promo = Vec::new();
    append_tag(&mut promo, 2, WIRE_VARINT);
    append_varint(&mut promo, 1); // start = true
    wrap_client_msg(req_id, 99, &promo)
}

// 14. ShadowSetTeam (Tag 121)
pub fn build_shadow_team_best(req_id: u64) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_VARINT);
    append_varint(&mut payload, 5); // SHADOW_TEAM_ACTION_BEST
    wrap_client_msg(req_id, 121, &payload)
}

// 15. ShadowEquipBest (Tag 127)
pub fn build_shadow_equip_best(req_id: u64) -> Vec<u8> {
    wrap_client_msg(req_id, 127, &[])
}

// 16. ShadowExpedition (Tag 133)
pub fn build_shadow_expedition(req_id: u64, claim: bool, start_best: bool) -> Vec<u8> {
    let mut payload = Vec::new();
    if claim {
        append_tag(&mut payload, 1, WIRE_VARINT);
        append_varint(&mut payload, 1);
    }
    if start_best {
        append_tag(&mut payload, 2, WIRE_VARINT);
        append_varint(&mut payload, 1);
    }
    wrap_client_msg(req_id, 133, &payload)
}

// 17. ShadowEvolve (Tag 138)
pub fn build_shadow_evolve(req_id: u64, shadow_id: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_BYTES);
    append_string(&mut payload, shadow_id);
    wrap_client_msg(req_id, 138, &payload)
}

// 18. Gems (Tag 85)
pub fn build_combine_gems(req_id: u64, tier: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_VARINT);
    append_varint(&mut payload, tier as u64);
    append_tag(&mut payload, 2, WIRE_VARINT);
    append_varint(&mut payload, 1); // combine_all
    wrap_client_msg(req_id, 85, &payload)
}

// 19. Enchant (Tag 80)
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

// 20. Party Requests (Tag 290)
pub fn build_party_invite(req_id: u64, target_name: &str) -> Vec<u8> {
    let mut invite_by_name = Vec::new();
    append_tag(&mut invite_by_name, 1, WIRE_BYTES);
    append_string(&mut invite_by_name, target_name);

    let mut party_req = Vec::new();
    append_tag(&mut party_req, 2, WIRE_BYTES);
    append_bytes(&mut party_req, &invite_by_name);

    wrap_client_msg(req_id, 290, &party_req)
}

pub fn build_party_answer(req_id: u64, from_account: i64, accept: bool) -> Vec<u8> {
    let mut answer = Vec::new();
    append_tag(&mut answer, 1, WIRE_VARINT);
    append_varint(&mut answer, from_account as u64);
    append_tag(&mut answer, 2, WIRE_VARINT);
    append_varint(&mut answer, if accept { 1 } else { 0 });

    let mut party_req = Vec::new();
    append_tag(&mut party_req, 3, WIRE_BYTES);
    append_bytes(&mut party_req, &answer);

    wrap_client_msg(req_id, 290, &party_req)
}

pub fn build_party_leave(req_id: u64) -> Vec<u8> {
    let mut party_req = Vec::new();
    append_tag(&mut party_req, 4, WIRE_BYTES);
    append_bytes(&mut party_req, &[]);

    wrap_client_msg(req_id, 290, &party_req)
}

// 21. Tower (Tag 370)
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

// 22. Daily Boss (Tag 39)
pub fn build_daily_boss(req_id: u64) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_VARINT);
    append_varint(&mut payload, 1);
    wrap_client_msg(req_id, 39, &payload)
}

// 23. Claim Missions (Tag 410)
pub fn build_claim_missions(req_id: u64) -> Vec<u8> {
    let mut payload = Vec::new();
    append_tag(&mut payload, 1, WIRE_VARINT);
    append_varint(&mut payload, 1); // state = true
    append_tag(&mut payload, 2, WIRE_VARINT);
    append_varint(&mut payload, 1); // claim_journey = true
    append_tag(&mut payload, 3, WIRE_BYTES);
    append_string(&mut payload, "daily"); // claim_daily = "daily"

    wrap_client_msg(req_id, 410, &payload)
}

// Protowire Decoding Helpers
pub fn consume_tag(data: &[u8]) -> Option<(u32, u32, usize)> {
    let (v, n) = consume_varint(data)?;
    let num = (v >> 3) as u32;
    let wtype = (v & 0x07) as u32;
    Some((num, wtype, n))
}

pub fn consume_varint(data: &[u8]) -> Option<(u64, usize)> {
    let mut val: u64 = 0;
    let mut shift = 0;
    for (i, &b) in data.iter().enumerate() {
        if i >= 10 {
            return None;
        }
        val |= ((b & 0x7F) as u64) << shift;
        if b < 0x80 {
            return Some((val, i + 1));
        }
        shift += 7;
    }
    None
}

pub fn consume_bytes(data: &[u8]) -> Option<(&[u8], usize)> {
    let (len, n) = consume_varint(data)?;
    let len = len as usize;
    if data.len() < n + len {
        return None;
    }
    Some((&data[n..n + len], n + len))
}

pub fn parse_string_field(mut data: &[u8], target_num: u32) -> String {
    while !data.is_empty() {
        let (num, wtype, n) = match consume_tag(data) {
            Some(t) => t,
            None => break,
        };
        data = &data[n..];
        if wtype == WIRE_BYTES {
            let (bytes, n2) = match consume_bytes(data) {
                Some(b) => b,
                None => break,
            };
            data = &data[n2..];
            if num == target_num {
                return String::from_utf8_lossy(bytes).to_string();
            }
        } else if wtype == WIRE_VARINT {
            let (_, n2) = match consume_varint(data) {
                Some(v) => v,
                None => break,
            };
            data = &data[n2..];
        } else if wtype == WIRE_FIXED32 {
            if data.len() < 4 { break; }
            data = &data[4..];
        } else if wtype == WIRE_FIXED64 {
            if data.len() < 8 { break; }
            data = &data[8..];
        } else {
            break;
        }
    }
    String::new()
}

pub fn parse_varint_field(mut data: &[u8], target_num: u32) -> u64 {
    while !data.is_empty() {
        let (num, wtype, n) = match consume_tag(data) {
            Some(t) => t,
            None => break,
        };
        data = &data[n..];
        if wtype == WIRE_VARINT {
            let (val, n2) = match consume_varint(data) {
                Some(v) => v,
                None => break,
            };
            data = &data[n2..];
            if num == target_num {
                return val;
            }
        } else if wtype == WIRE_BYTES {
            let (_, n2) = match consume_bytes(data) {
                Some(b) => b,
                None => break,
            };
            data = &data[n2..];
        } else if wtype == WIRE_FIXED32 {
            if data.len() < 4 { break; }
            data = &data[4..];
        } else if wtype == WIRE_FIXED64 {
            if data.len() < 8 { break; }
            data = &data[8..];
        } else {
            break;
        }
    }
    0
}

pub fn parse_codex_monsters(mut data: &[u8]) -> HashMap<String, u64> {
    let mut res = HashMap::new();
    while !data.is_empty() {
        let (num, wtype, n) = match consume_tag(data) {
            Some(t) => t,
            None => break,
        };
        data = &data[n..];

        if wtype == WIRE_BYTES {
            let (sub, n2) = match consume_bytes(data) {
                Some(b) => b,
                None => break,
            };
            data = &data[n2..];

            if num == 1 { // repeated MonstersEntry
                let mut m_entry = sub;
                let mut key = String::new();
                let mut val = 0u64;
                while !m_entry.is_empty() {
                    let (e_num, e_wtype, en) = match consume_tag(m_entry) {
                        Some(t) => t,
                        None => break,
                    };
                    m_entry = &m_entry[en..];

                    if e_wtype == WIRE_BYTES {
                        let (e_sub, en2) = match consume_bytes(m_entry) {
                            Some(b) => b,
                            None => break,
                        };
                        m_entry = &m_entry[en2..];
                        if e_num == 1 {
                            key = String::from_utf8_lossy(e_sub).to_string();
                        }
                    } else if e_wtype == WIRE_VARINT {
                        let (v, en2) = match consume_varint(m_entry) {
                            Some(b) => b,
                            None => break,
                        };
                        m_entry = &m_entry[en2..];
                        if e_num == 2 {
                            val = v;
                        }
                    } else {
                        break;
                    }
                }
                if !key.is_empty() {
                    res.insert(key, val);
                }
            }
        } else if wtype == WIRE_VARINT {
            let (_, n2) = match consume_varint(data) {
                Some(b) => b,
                None => break,
            };
            data = &data[n2..];
        } else {
            break;
        }
    }
    res
}
