use smash::app::{self, lua_bind::*};
use smash::lib::{lua_const::*, L2CValue};
use smash::lua2cpp::L2CFighterCommon;
use smash::{phx::*, hash40};
use std::collections::HashMap; //this is weird i know... would be better to index into an array with fighter_kind but too lazy to print the fighter_kind's for everybody
use crate::utils::*;

//Jump (runs once at the beginning of the status)
pub static mut curr_momentum: [f32;8] = [0.0;8];
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_Jump_sub)]
pub unsafe fn status_jump_sub_hook(fighter: &mut L2CFighterCommon, param_2: L2CValue, param_3: L2CValue) -> L2CValue {
    let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);

    let air_friction_x = WorkModule::get_param_float(boma, hash40("air_brake_x"), 0);
    let mut inc_speed_vec = Vector3f { x: 0.01, y:0.0, z:0.0};

    if StatusModule::prev_status_kind(boma, 1) /*2 statuses ago to account for jumpsquat*/ != *FIGHTER_STATUS_KIND_TURN {
        if PostureModule::lr(boma) * KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN) < 0.0 {
            inc_speed_vec.x *= -1.0;
        }
        while KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN).abs() < (curr_momentum[get_player_number(boma)].abs() - air_friction_x) * get_fighter_momentum_mul(boma) {
            KineticModule::add_speed(boma, &inc_speed_vec); // note to self: add_speed interally accounts for current PostureModule::lr
        }
    }

    original!()(fighter, param_2, param_3)
}


//Aerials (runs once at the beginning of the status)
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_sub_attack_air_common)]
pub unsafe fn status_attack_air_hook(fighter: &mut L2CFighterCommon, param_1: L2CValue){
    let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);

    let prev_status_check = [*FIGHTER_STATUS_KIND_FALL, *FIGHTER_STATUS_KIND_JUMP].contains(&StatusModule::prev_status_kind(boma, 0));

    if KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN) * PostureModule::lr(boma) >= 0.0 && prev_status_check {
        let inc_speed_vec = Vector3f { x: 0.01, y:0.0, z:0.0};
        while KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN).abs() < curr_momentum[get_player_number(boma)].abs() - WorkModule::get_param_float(boma, hash40("air_brake_x"), 0) {  // <- apply air friction outright since without this, aerials ignore air friction for 1 frame
            KineticModule::add_speed(boma, &inc_speed_vec);
        }
    }

    original!()(fighter, param_1)
}

//called in moveset_edits in sys_line_system_control_fighter.rs
pub unsafe fn momentum_transfer_helper(boma: &mut app::BattleObjectModuleAccessor, status_kind: i32) {
    if get_player_number(boma) == 0 {
        println!("x_vel: {}", KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
    }
    if [*FIGHTER_STATUS_KIND_JUMP_SQUAT, *FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_FALL].contains(&status_kind) {
        curr_momentum[get_player_number(boma)] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN); 
    }
}

//addition special moves/anything else that should conserve momentum
pub unsafe fn momentum_transfer_additional(boma: &mut app::BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, curr_frame: f32, fighter_kind: i32) {
    let mut should_conserve_momentum = false;
    
    if situation_kind == *SITUATION_KIND_AIR && curr_frame <= 1.0 {

        if [*FIGHTER_KIND_CAPTAIN, *FIGHTER_KIND_MARIO, *FIGHTER_KIND_LUIGI]
            .contains(&fighter_kind) && status_kind == *FIGHTER_STATUS_KIND_SPECIAL_N { //put any fighter here whose neutral special should conserve momentum
                should_conserve_momentum = true; //spacie lasers, falcon punch, 
        }


        if should_conserve_momentum {
            let inc_speed_vec = Vector3f { x: 0.01, y:0.0, z:0.0};
            while KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN).abs() < curr_momentum[get_player_number(boma)].abs() {
                KineticModule::add_speed(boma, &inc_speed_vec);
            }
        }

    }
}

//this code is weird and kinda sucks but rust doesn't like static hashmaps i guess. I'm still learning :)
unsafe fn get_fighter_momentum_mul(boma: &mut app::BattleObjectModuleAccessor) -> f32{
    let char_momentum_multipliers: HashMap<i32, f32> = [
(*FIGHTER_KIND_MARIO, 1.0),    //MARIO
(*FIGHTER_KIND_DONKEY, 0.95),    //Donkey Kong
(*FIGHTER_KIND_LINK, 0.95),    //LINK
(*FIGHTER_KIND_SAMUS, 0.8),    //SAMUS
(*FIGHTER_KIND_SAMUSD, 0.8),    //Dark Samus
(*FIGHTER_KIND_YOSHI, 0.95),    //YOSHI
(*FIGHTER_KIND_KIRBY, 1.0),    //KIRBY
(*FIGHTER_KIND_FOX, 0.9),    //FOX
(*FIGHTER_KIND_PIKACHU, 0.8),    //PIKACHU
(*FIGHTER_KIND_LUIGI, 0.6),    //LUIGI
(*FIGHTER_KIND_NESS, 0.9),    //NESS
(*FIGHTER_KIND_CAPTAIN, 1.0),    //CAPTAIN
(*FIGHTER_KIND_PURIN, 1.0),    //Jigglypuff
(*FIGHTER_KIND_PEACH, 0.65),    //PEACH
(*FIGHTER_KIND_DAISY, 0.75),    //DAISY
(*FIGHTER_KIND_KOOPA, 0.55),    //Bowser
(*FIGHTER_KIND_SHEIK, 0.6),    //SHEIK
(*FIGHTER_KIND_ZELDA, 0.9),    //ZELDA
(*FIGHTER_KIND_MARIOD, 1.0),    //Doctor Mario
(*FIGHTER_KIND_PICHU, 0.8),    //PICHU
(*FIGHTER_KIND_FALCO, 0.95),    //FALCO
(*FIGHTER_KIND_MARTH, 0.8),    //MARTH
(*FIGHTER_KIND_LUCINA, 0.8),    //LUCINA
(*FIGHTER_KIND_YOUNGLINK, 0.8),    //YOUNGLINK
(*FIGHTER_KIND_GANON, 0.85),    //GANON
(*FIGHTER_KIND_MEWTWO, 0.75),    //MEWTWO
(*FIGHTER_KIND_ROY, 0.7),    //ROY
(*FIGHTER_KIND_CHROM, 0.7),    //CHROM
(*FIGHTER_KIND_GAMEWATCH, 0.9),    //GAMEWATCH
(*FIGHTER_KIND_METAKNIGHT, 0.75),    //METAKNIGHT
(*FIGHTER_KIND_PIT, 0.95),    //PIT
(*FIGHTER_KIND_PITB, 0.9),    //Dark Pit
(*FIGHTER_KIND_SZEROSUIT, 0.7),    //Zero Suit Samus
(*FIGHTER_KIND_WARIO, 0.8),    //WARIO
(*FIGHTER_KIND_SNAKE, 0.65),    //SNAKE
(*FIGHTER_KIND_IKE, 1.0),    //IKE
(*FIGHTER_KIND_PZENIGAME, 1.0),    //Squirtle
(*FIGHTER_KIND_PFUSHIGISOU, 0.95),    //Ivysaur
(*FIGHTER_KIND_PLIZARDON, 0.85),    //Charizard
(*FIGHTER_KIND_DIDDY, 0.85),    //DIDDY
(*FIGHTER_KIND_LUCAS, 0.95),    //LUCAS
(*FIGHTER_KIND_SONIC, 0.75),    //SONIC
(*FIGHTER_KIND_DEDEDE, 1.0),    //DEDEDE
(*FIGHTER_KIND_PIKMIN, 0.8),    //PIKMIN
(*FIGHTER_KIND_LUCARIO, 0.8),    //LUCARIO
(*FIGHTER_KIND_ROBOT, 0.7),    //ROBOT
(*FIGHTER_KIND_TOONLINK, 0.8),    //TOONLINK
(*FIGHTER_KIND_WOLF, 0.95),    //WOLF
(*FIGHTER_KIND_MURABITO, 0.8),    //Villager --starting non-brawl fighters here
(*FIGHTER_KIND_ROCKMAN, 0.9),    //Megaman
(*FIGHTER_KIND_WIIFIT, 0.8),    //WIIFIT
(*FIGHTER_KIND_ROSETTA, 0.7),    //Rosalina & luma
(*FIGHTER_KIND_LITTLEMAC, 0.9),    //LITTLEMAC
(*FIGHTER_KIND_GEKKOUGA, 0.8),    //Greninja
(*FIGHTER_KIND_PALUTENA, 0.7),    //PALUTENA
(*FIGHTER_KIND_PACMAN, 0.8),    //PACMAN
(*FIGHTER_KIND_REFLET, 0.8),    //Robin
(*FIGHTER_KIND_SHULK, 0.9),    //SHULK
(*FIGHTER_KIND_KOOPAJR, 0.8),    //Bowser Jr.
(*FIGHTER_KIND_DUCKHUNT, 0.8),    //DUCKHUNT
(*FIGHTER_KIND_RYU, 0.8),    //RYU
(*FIGHTER_KIND_KEN, 0.8),    //KEN
(*FIGHTER_KIND_CLOUD, 0.9),    //CLOUD
(*FIGHTER_KIND_KAMUI, 0.9),    //Corrin
(*FIGHTER_KIND_BAYONETTA, 0.95),    //BAYONETTA
(*FIGHTER_KIND_INKLING, 0.9),    //INKLING
(*FIGHTER_KIND_RIDLEY, 0.8),    //RIDLEY
(*FIGHTER_KIND_SIMON, 0.8),    //SIMON
(*FIGHTER_KIND_RICHTER, 0.8),    //RICHTER
(*FIGHTER_KIND_KROOL, 0.9),    //KROOL
(*FIGHTER_KIND_SHIZUE, 0.8),    //Isabelle
(*FIGHTER_KIND_GAOGAEN, 0.9),    //Incineroar
(*FIGHTER_KIND_PACKUN, 0.8),    //Plant gang
(*FIGHTER_KIND_JACK, 0.85),   //Joker
(*FIGHTER_KIND_BRAVE, 0.85),    //Hero
(*FIGHTER_KIND_BUDDY, 0.8),    //Banjo
(*FIGHTER_KIND_DOLLY, 0.75),   //Terry
(*FIGHTER_KIND_MASTER, 0.9),    //Byleth
(*FIGHTER_KIND_MIIFIGHTER, 0.8),    //MIIFIGHTER
(*FIGHTER_KIND_MIISWORDSMAN, 0.8),    //MIISWORDSMAN
(*FIGHTER_KIND_MIIGUNNER, 0.8),    //MIIGUNNER
(*FIGHTER_KIND_POPO, 0.8),    //POPO
(*FIGHTER_KIND_NANA, 0.8)    //NANA
].iter().cloned().collect();

    *char_momentum_multipliers.get(&get_kind(boma)).unwrap()
}