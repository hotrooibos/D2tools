use crate::process::Process;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum D2RVersion {
    D2R,
}

pub const STAT_STRENGTH: u16 = 0;
pub const STAT_ENERGY: u16 = 1;
pub const STAT_DEXTERITY: u16 = 2;
pub const STAT_VITALITY: u16 = 3;
pub const STAT_HITPOINTS: u16 = 6;
pub const STAT_MAXHP: u16 = 7;
pub const STAT_MANA: u16 = 8;
pub const STAT_MAXMANA: u16 = 9;
pub const STAT_STAMINA: u16 = 10;
pub const STAT_MAXSTAMINA: u16 = 11;
pub const STAT_LEVEL: u16 = 12;

pub fn detect_version(process: &Process) -> Option<D2RVersion> {
    if process.get_base_address() != 0 {
        Some(D2RVersion::D2R)
    } else {
        None
    }
}

pub fn read_cstring(process: &Process, address: usize, max_len: usize) -> Option<String> {
    let data = process.read_array(address, max_len)?;
    let null_pos = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8(data[..null_pos].to_vec()).ok()
}

fn find_pattern_in_memory(process: &Process, base: usize, size: usize, pattern: &[u8], mask: &[u8]) -> Option<usize> {
    let data = process.read_array(base, size)?;
    
    for i in 0..data.len() {
        if data.len() - i < pattern.len() {
            break;
        }
        
        let mut matches = true;
        for j in 0..pattern.len() {
            if mask[j] == 0xFF && data[i + j] != pattern[j] {
                matches = false;
                break;
            }
        }
        
        if matches {
            return Some(base + i);
        }
    }
    
    None
}

fn read_i32_at(process: &Process, addr: usize) -> Option<i32> {
    process.read_u32(addr).map(|v| v as i32)
}

pub struct D2RAddresses {
    pub hash_table_base: usize,
    pub roster_data: usize,
    pub player_unit_ptr: usize,
}

pub fn find_d2r_addresses(process: &Process) -> Option<D2RAddresses> {
    let base = process.get_base_address();
    
    eprintln!("DEBUG: Base address: 0x{:016X}", base);
    
    // Read PE header to find .text section
    if let Some(pe_offset) = process.read_u32(base + 0x3C) {
        eprintln!("DEBUG: PE header at offset 0x{:X}", pe_offset);
        
        let pe_base = base + pe_offset as usize;
        
        // Read signature
        if let Some(sig) = process.read_u32(pe_base) {
            eprintln!("DEBUG: Signature: 0x{:08X}", sig);
            if sig != 0x4550 { // "PE\0\0"
                eprintln!("DEBUG: Invalid PE signature!");
                return None;
            }
        }
        
        // COFF header
        let num_sections = process.read_u16(pe_base + 6).unwrap_or(0);
        let opt_header_size = process.read_u16(pe_base + 20).unwrap_or(0);
        eprintln!("DEBUG: Sections: {}, OptHeaderSize: {}", num_sections, opt_header_size);
        
        // Section headers start at PE + 24 + optional header size
        let section_headers_start = pe_base + 24 + opt_header_size as usize;
        eprintln!("DEBUG: Section headers at 0x{:016X}", section_headers_start);
        
        // Iterate through sections to find .text
        let section_header_size = 40;
        let mut code_start = 0usize;
        let mut code_size = 0usize;
        
        for i in 0..num_sections.min(16) as usize {
            let section_offset = section_headers_start + i * section_header_size;
            
            // Read section name
            let mut section_name = [0u8; 8];
            for j in 0..8 {
                section_name[j] = process.read_u8(section_offset + j).unwrap_or(0);
            }
            let name = String::from_utf8_lossy(&section_name);
            
            // VirtualSize at +8, VirtualAddress at +12, PointerToRawData at +20, SizeOfRawData at +16
            let virtual_size = process.read_u32(section_offset + 8).unwrap_or(0);
            let virtual_addr = process.read_u32(section_offset + 12).unwrap_or(0);
            let raw_data_ptr = process.read_u32(section_offset + 20).unwrap_or(0);
            let raw_data_size = process.read_u32(section_offset + 16).unwrap_or(0);
            
            eprintln!("DEBUG: Section {}: '{}' VSize=0x{:X}, VAddr=0x{:X}, RawPtr=0x{:X}, RawSize=0x{:X}", 
                      i, name, virtual_size, virtual_addr, raw_data_ptr, raw_data_size);
            
            // Check if this is .text section
            if name.starts_with(".text") || name.starts_with("CODE") {
                code_start = base + raw_data_ptr as usize;
                code_size = raw_data_size as usize;
                eprintln!("DEBUG: Found .text section! RawPtr=0x{:016X}, RawSize=0x{:X}", code_start, code_size);
                break;
            }
        }
        
        // If we didn't find .text, use base + 0x1000 as code
        if code_size == 0 {
            code_start = base + 0x1000;
            code_size = 0x10000000; // 256MB
            eprintln!("DEBUG: No .text found, using fallback code_start=0x{:016X}", code_start);
        }
        
        eprintln!("DEBUG: Scanning code at 0x{:016X}, size: {}MB", code_start, code_size / 1024 / 1024);
        
        // Also find .rdata section for data references
        let mut rdata_start = code_start;
        let mut rdata_size = 0usize;
        
        for i in 0..num_sections.min(16) as usize {
            let section_offset = section_headers_start + i * 40;
            let mut section_name = [0u8; 8];
            for j in 0..8 {
                section_name[j] = process.read_u8(section_offset + j).unwrap_or(0);
            }
            let name = String::from_utf8_lossy(&section_name);
            
            let raw_data_ptr = process.read_u32(section_offset + 20).unwrap_or(0);
            let raw_data_size = process.read_u32(section_offset + 16).unwrap_or(0);
            
            if name.contains("rdata") || name.contains("data") {
                rdata_start = base + raw_data_ptr as usize;
                rdata_size = raw_data_size as usize;
                eprintln!("DEBUG: Found '{}' section at 0x{:016X}, size 0x{:X}", name, rdata_start, rdata_size);
                break;
            }
        }
        
        // Scan .rdata section for roster pointer pattern
        if rdata_size > 0 {
            let pattern_roster = [0x02, 0x45, 0x33, 0xD2, 0x4D, 0x8B];
            let mask_roster = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
            eprintln!("DEBUG: Scanning .rdata for roster pattern...");
            if let Some(pos) = find_pattern_in_memory(process, rdata_start, rdata_size, &pattern_roster, &mask_roster) {
                eprintln!("DEBUG: Found roster pattern in .rdata at 0x{:016X}", pos);
                if let Some(rel) = read_i32_at(process, pos - 3) {
                    let ref_addr = pos + 1 + rel as usize;
                    eprintln!("DEBUG: Roster ref at 0x{:016X}", ref_addr);
                    if let Some(roster_ptr) = process.read_u64(ref_addr) {
                        eprintln!("DEBUG: Roster ptr = 0x{:016X}", roster_ptr);
                        if roster_ptr > 0x1000 {
                            if let Some((name, hp, max_hp)) = read_player_via_roster(process, roster_ptr as usize) {
                                eprintln!("DEBUG: SUCCESS! Player: {} HP: {}/{}", name, hp, max_hp);
                                return Some(D2RAddresses {
                                    hash_table_base: 0,
                                    roster_data: roster_ptr as usize,
                                    player_unit_ptr: 0,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Pattern 1: Roster pointer
        let pattern1 = [0x02, 0x45, 0x33, 0xD2, 0x4D, 0x8B];
        let mask1 = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        eprintln!("DEBUG: Scanning .text for roster pattern...");
        if let Some(pos) = find_pattern_in_memory(process, code_start, code_size, &pattern1, &mask1) {
            eprintln!("DEBUG: Found roster pattern at 0x{:016X}", pos);
            if let Some(rel) = read_i32_at(process, pos - 3) {
                let ref_addr = pos + 1 + rel as usize;
                if let Some(roster_ptr) = process.read_u64(ref_addr) {
                    eprintln!("DEBUG: Roster ptr = 0x{:016X}", roster_ptr);
                    if roster_ptr > 0x1000 {
                        if let Some((name, hp, max_hp)) = read_player_via_roster(process, roster_ptr as usize) {
                            eprintln!("DEBUG: SUCCESS! Player: {} HP: {}/{}", name, hp, max_hp);
                            return Some(D2RAddresses {
                                hash_table_base: 0,
                                roster_data: roster_ptr as usize,
                                player_unit_ptr: 0,
                            });
                        }
                    }
                }
            }
        }
        
        // Pattern 2: Hash table pointer
        let pattern2 = [0x48, 0x03, 0xC7, 0x49, 0x8B, 0x8C, 0xC6];
        let mask2 = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        eprintln!("DEBUG: Scanning .text for hash table pattern...");
        if let Some(pos) = find_pattern_in_memory(process, code_start, code_size, &pattern2, &mask2) {
            eprintln!("DEBUG: Found hash table at 0x{:016X}", pos);
            if let Some(rel) = read_i32_at(process, pos + 7) {
                let addr = pos + 11 + rel as usize;
                eprintln!("DEBUG: Hash table ref at 0x{:016X}", addr);
            }
        }
        
        // Pattern 3: Player unit pointer
        let pattern3 = [0x44, 0x88, 0x25, 0x00, 0x00, 0x00, 0x00, 0x66, 0x44, 0x89, 0x25];
        let mask3 = [0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF];
        if let Some(pos) = find_pattern_in_memory(process, code_start, code_size, &pattern3, &mask3) {
            eprintln!("DEBUG: Found player unit pattern at 0x{:016X}", pos);
            if let Some(rel) = read_i32_at(process, pos + 3) {
                let addr = pos + 7 + rel as usize - 0x121;
                eprintln!("DEBUG: Player unit ref at 0x{:016X}", addr);
            }
        }
    }
    
    // Search for strings in first 512MB of process memory
    eprintln!("DEBUG: Scanning for character class strings...");
    let search_strings = ["sorceress", "amazon", "necromancer", "paladin", "barbarian", "druid", "assassin"];
    for search_str in &search_strings {
        let bytes: Vec<u8> = search_str.as_bytes().to_vec();
        let mask: Vec<u8> = vec![0xFF; bytes.len()];
        
        if let Some(found_pos) = find_pattern_in_memory(process, base + 0x1000, 0x20000000, &bytes, &mask) {
            eprintln!("DEBUG: FOUND string '{}' at 0x{:016X}", search_str, found_pos);
            
            // Scan backwards for roster structure
            for back in (0..0x30000).step_by(16) {
                let check_addr = found_pos.wrapping_sub(back);
                if check_addr < base { break; }
                
                if let Some((name, hp, max_hp)) = read_player_via_roster(process, check_addr) {
                    eprintln!("DEBUG: SUCCESS via string! Player: {} HP: {}/{}", name, hp, max_hp);
                    return Some(D2RAddresses {
                        hash_table_base: 0,
                        roster_data: check_addr,
                        player_unit_ptr: 0,
                    });
                }
            }
        }
    }
    
    // Try scanning D2Client.dll for global data pointers
    // First, find D2Client.dll base address
    eprintln!("DEBUG: Enumerating D2R modules...");
    
    let modules = process.find_all_modules();
    eprintln!("DEBUG: Found {} modules", modules.len());
    
    let mut d2client_base = 0usize;
    for (name, addr) in &modules {
        if name.to_lowercase().contains("d2client") {
            d2client_base = *addr;
            eprintln!("DEBUG: Found D2Client at 0x{:016X}", d2client_base);
            break;
        }
    }
    
    if d2client_base == 0 {
        eprintln!("DEBUG: D2Client.dll not found in module list!");
        return None;
    }
    
    if d2client_base == 0 {
        eprintln!("DEBUG: D2Client.dll not found!");
        return None;
    }
    
    // Read D2Client.dll PE sections
    let pe_offset = process.read_u32(d2client_base + 0x3C)?;
    let opt_header_size = process.read_u16(d2client_base + pe_offset as usize + 20).unwrap_or(0);
    let num_sections = process.read_u16(d2client_base + pe_offset as usize + 6).unwrap_or(0);
    let section_headers_start = d2client_base + pe_offset as usize + 24 + opt_header_size as usize;
    
    eprintln!("DEBUG: D2Client PE: {} sections, opt header size {}", num_sections, opt_header_size);
    
    // Scan .data section of D2Client.dll for pointers
    for i in 0..num_sections.min(16) as usize {
        let section_offset = section_headers_start + i * 40;
        let mut section_name = [0u8; 8];
        for j in 0..8 {
            section_name[j] = process.read_u8(section_offset + j).unwrap_or(0);
        }
        let name = String::from_utf8_lossy(&section_name);
        
        let raw_data_ptr = process.read_u32(section_offset + 20).unwrap_or(0);
        let raw_data_size = process.read_u32(section_offset + 16).unwrap_or(0);
        
        if name.contains("data") {
            let data_start = d2client_base + raw_data_ptr as usize;
            let data_size = raw_data_size as usize;
            eprintln!("DEBUG: Scanning .data section at 0x{:016X}, size 0x{:X}", data_start, data_size);
            
            // Scan for pointers in .data section
            for addr in (0..data_size.min(0x100000)).step_by(8) {
                let check_addr = data_start + addr;
                if let Some(ptr) = process.read_u64(check_addr) {
                    if ptr > 0x1000000000 && ptr < 0x7FFFFFFFFFFF {
                        if let Some((name, hp, max_hp)) = read_player_via_roster(process, ptr as usize) {
                            eprintln!("DEBUG: SUCCESS! Player: '{}' HP: {}/{}", name, hp, max_hp);
                            return Some(D2RAddresses {
                                hash_table_base: 0,
                                roster_data: ptr as usize,
                                player_unit_ptr: 0,
                            });
                        }
                    }
                }
            }
        }
    }
    
    eprintln!("DEBUG: Pattern scanning complete");
    None
}

pub fn read_player_via_roster(process: &Process, roster_ptr: usize) -> Option<(String, i32, i32)> {
    if roster_ptr == 0 || roster_ptr < 0x10000 {
        return None;
    }
    
    // Read character name (offset 0x00, 16 bytes)
    let mut roster_name = [0u8; 16];
    for i in 0..16 {
        roster_name[i] = process.read_u8(roster_ptr + i).unwrap_or(0);
    }
    
    // Validate name: D2 character names are 3-15 chars, letters only (A-Z, a-z)
    let mut valid_name = true;
    let mut name_len = 0;
    for &b in &roster_name {
        if b == 0 {
            break;
        }
        // Only allow letters
        if !((b >= 65 && b <= 90) || (b >= 97 && b <= 122)) {
            valid_name = false;
            break;
        }
        name_len += 1;
    }
    
    // Character names in D2 are 3-15 characters
    if !valid_name || name_len < 3 || name_len > 15 {
        return None;
    }
    
    let player_name = String::from_utf8_lossy(&roster_name).trim_end_matches('\0').to_string();
    
    // Read life percent at offset 0x4C (stored as 0-10000)
    let life_percent_raw = process.read_u32(roster_ptr + 0x4C).unwrap_or(0);
    
    // Life percent should be 0-10000 (or sometimes 0-10000/256)
    if life_percent_raw > 10000 {
        return None;
    }
    
    // Read max HP at offset 0x68 (stored shifted by 8)
    let max_hp_raw = process.read_i32(roster_ptr + 0x68).unwrap_or(0);
    let max_hp = max_hp_raw >> 8;
    
    // Max HP should be reasonable (50 to 200000 for a D2 character)
    if max_hp < 50 || max_hp > 200000 {
        return None;
    }
    
    // Calculate current HP
    let life_pct = life_percent_raw.min(10000);
    let current_hp = ((life_pct as i64 * max_hp as i64) / 10000) as i32;
    
    Some((player_name, current_hp, max_hp))
}

pub fn read_player_stats(process: &Process, stat_list_ptr: usize) -> Option<PlayerStats> {
    let flag = process.read_u32(stat_list_ptr + 0x18)?;
    let full_stats = (flag & 0x80000000) != 0;
    
    let stat_ptr_offset = if full_stats { 0x80 } else { 0x10 };
    let stat_count_offset = if full_stats { 0x88 } else { 0x18 };

    let stat_ptr = process.read_u64(stat_list_ptr + stat_ptr_offset)? as usize;
    let stat_count = process.read_u32(stat_list_ptr + stat_count_offset)? as usize;

    if stat_ptr == 0 || stat_count == 0 || stat_count > 256 {
        return None;
    }

    let mut stats = PlayerStats::new();

    for i in 0..stat_count.min(256) {
        let entry_addr = stat_ptr + i * 8;
        
        let stat_id = process.read_u16(entry_addr)?;
        if stat_id == 0xFFFF {
            break;
        }
        
        let _param = process.read_u16(entry_addr + 2)?;
        let raw_value = process.read_i32(entry_addr + 4)?;
        
        let value = if stat_id >= STAT_HITPOINTS && stat_id <= STAT_MAXSTAMINA {
            raw_value >> 8
        } else {
            raw_value
        };
        
        stats.stats.push((stat_id, value));
    }

    Some(stats)
}

#[derive(Debug)]
pub struct PlayerStats {
    pub stats: Vec<(u16, i32)>,
}

impl PlayerStats {
    pub fn new() -> Self {
        Self { stats: Vec::new() }
    }

    pub fn get(&self, stat_id: u16) -> Option<i32> {
        self.stats.iter()
            .find(|(id, _)| *id == stat_id)
            .map(|(_, v)| *v)
    }
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self::new()
    }
}
