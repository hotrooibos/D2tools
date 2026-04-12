use crate::d2r::{
    find_d2r_addresses, read_player_stats, read_player_via_roster,
    STAT_DEXTERITY, STAT_ENERGY, STAT_HITPOINTS, STAT_LEVEL, STAT_MAXHP, 
    STAT_MANA, STAT_MAXMANA, STAT_MAXSTAMINA, STAT_STAMINA, STAT_STRENGTH, 
    STAT_VITALITY, D2RVersion,
};
use crate::process::Process;

#[derive(Debug, Clone)]
pub struct PlayerData {
    pub name: String,
    pub level: i32,
    pub current_hp: i32,
    pub max_hp: i32,
    pub current_mana: i32,
    pub max_mana: i32,
    pub current_stamina: i32,
    pub max_stamina: i32,
    pub strength: i32,
    pub energy: i32,
    pub dexterity: i32,
    pub vitality: i32,
    pub in_game: bool,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            name: String::new(),
            level: 0,
            current_hp: 0,
            max_hp: 0,
            current_mana: 0,
            max_mana: 0,
            current_stamina: 0,
            max_stamina: 0,
            strength: 0,
            energy: 0,
            dexterity: 0,
            vitality: 0,
            in_game: false,
        }
    }
}

pub fn read_player(process: &Process, _version: D2RVersion) -> Option<PlayerData> {
    eprintln!("DEBUG: Finding D2R addresses via pattern scanning...");
    
    let addresses = find_d2r_addresses(process)?;
    eprintln!("DEBUG: Found addresses - roster: 0x{:016X}, player: 0x{:016X}", 
              addresses.roster_data, addresses.player_unit_ptr);
    
    // Try roster approach first
    if addresses.roster_data != 0 {
        eprintln!("DEBUG: Trying roster-based approach...");
        if let Some((name, hp, max_hp)) = read_player_via_roster(process, addresses.roster_data) {
            eprintln!("DEBUG: Roster approach succeeded!");
            return Some(PlayerData {
                name,
                level: 1,
                current_hp: hp,
                max_hp,
                current_mana: 0,
                max_mana: 0,
                current_stamina: 0,
                max_stamina: 0,
                strength: 0,
                energy: 0,
                dexterity: 0,
                vitality: 0,
                in_game: true,
            });
        }
    }
    
    // Fall back to unit-based approach
    if addresses.player_unit_ptr == 0 {
        eprintln!("DEBUG: No player unit pointer found");
        return None;
    }
    
    eprintln!("DEBUG: Trying unit-based approach...");
    
    let mut player = PlayerData::default();

    // Read character name at offset 0x14
    let char_name_addr = addresses.player_unit_ptr + 0x14;
    if let Some(name) = crate::d2r::read_cstring(process, char_name_addr, 16) {
        player.name = name;
    }

    if player.name.is_empty() {
        eprintln!("DEBUG: name is empty");
        return None;
    }

    player.in_game = true;

    // Read stat list pointer at offset 0x88
    let stat_list_ptr = process.read_u64(addresses.player_unit_ptr + 0x88)? as usize;
    let stats = read_player_stats(process, stat_list_ptr)?;

    player.strength = stats.get(STAT_STRENGTH).unwrap_or(0);
    player.energy = stats.get(STAT_ENERGY).unwrap_or(0);
    player.dexterity = stats.get(STAT_DEXTERITY).unwrap_or(0);
    player.vitality = stats.get(STAT_VITALITY).unwrap_or(0);
    player.current_hp = stats.get(STAT_HITPOINTS).unwrap_or(0);
    player.max_hp = stats.get(STAT_MAXHP).unwrap_or(0);
    player.current_mana = stats.get(STAT_MANA).unwrap_or(0);
    player.max_mana = stats.get(STAT_MAXMANA).unwrap_or(0);
    player.current_stamina = stats.get(STAT_STAMINA).unwrap_or(0);
    player.max_stamina = stats.get(STAT_MAXSTAMINA).unwrap_or(0);
    player.level = stats.get(STAT_LEVEL).unwrap_or(0);

    if player.max_hp == 0 {
        player.max_hp = player.current_hp;
    }
    if player.max_mana == 0 {
        player.max_mana = player.current_mana;
    }
    if player.max_stamina == 0 {
        player.max_stamina = player.current_stamina;
    }

    eprintln!("DEBUG: HP={}/{}, MP={}/{}", player.current_hp, player.max_hp, player.current_mana, player.max_mana);

    Some(player)
}
