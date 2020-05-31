use smash::app::{self, lua_bind::*};
use smash::lib::{lua_const::*, L2CValue};
use smash::lua2cpp::L2CFighterCommon;
use smash::{phx::*, hash40};
use crate::utils::*;

//Jump (runs once at the beginning of the status)
pub static mut curr_momentum: [f32;8] = [0.0;8];
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_Jump_sub)]
pub unsafe fn status_jump_sub_hook(fighter: &mut L2CFighterCommon, param_2: L2CValue, param_3: L2CValue) -> L2CValue {
    let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);

    let jump_speed_x = WorkModule::get_param_float(boma, hash40("jump_speed_x"), 0);
    let jump_speed_x_mul = WorkModule::get_param_float(boma, hash40("jump_speed_x_mul"), 0);
    let stick_x = ControlModule::get_stick_x(boma);
    let x_vel = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
    let jump_speed_x_max = WorkModule::get_param_float(boma, hash40("jump_speed_x_max"), 0);

    let calcJumpSpeed = (jump_speed_x * stick_x) + (jump_speed_x_mul * x_vel);
    let jumpSpeedClamped = clamp(calcJumpSpeed, -jump_speed_x_max, jump_speed_x_max);  //melee jump speed calculation... courtesey of Brawltendo

    if StatusModule::prev_status_kind(boma, 1) /*2 statuses ago to account for jumpsquat*/ != *FIGHTER_STATUS_KIND_TURN {
        let mut inc_speed_vec = Vector3f { x: 0.01, y:0.0, z:0.0};
        if PostureModule::lr(boma) * KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN) < 0.0 {
            inc_speed_vec.x *= -1.0;
        }
        while KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN).abs() < jumpSpeedClamped.abs() {
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

        //for some reason spacie lasers aren't working here... seems they get their airspeed capped throughout the laser status

        if should_conserve_momentum {
            let inc_speed_vec = Vector3f { x: 0.01, y:0.0, z:0.0};
            while KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN).abs() < curr_momentum[get_player_number(boma)].abs() {
                KineticModule::add_speed(boma, &inc_speed_vec);
            }
        }

    }
}