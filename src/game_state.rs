extern crate nalgebra as na;
use crate::frame;
use crate::frame::Frame;
use crate::heightmap_editor;
use crate::mobile;
use crate::utils;
use na::{Point3, Vector2, Vector3};
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use utils::*;

use crate::frame::Player;
use mobile::*;

pub struct State {
    pub position: Point3<f32>,
    pub dir: Vector3<f32>,

    pub position_smooth: Point3<f32>,
    pub dir_smooth: Vector3<f32>,

    pub mouse_world_pos: Option<Vector3<f32>>,
    pub screen_center_world_pos: Option<Vector3<f32>>,

    pub heightmap_editor: heightmap_editor::State,

    //Data to interpolate
    pub frame_minus_one: Frame,
    pub frame_zero: Frame,
    pub frame_zero_time_received: Instant,

    //Interpolated
    pub kbots: HashMap<Id<KBot>, KBot>,
    pub kinematic_projectiles: HashMap<Id<KinematicProjectile>, KinematicProjectile>,
    pub server_sec: f32,

    pub in_screen: Vec<(Id<KBot>, Vector2<f32>)>,
    pub selected: HashSet<IdValue>,

    pub start_time: Instant,
    pub last_frame: Instant,

    pub my_player_id: Option<Id<Player>>,

    pub players: HashMap<Id<Player>, Player>,

    pub fps: u64,
}

impl State {
    pub fn new() -> Self {
        State {
            position: Point3::new(1024.0, 100.0, 50.0),
            dir: Vector3::new(0.0, 0.3, -1.0),
            position_smooth: Point3::new(0.0, 0.0, 30000.0),
            dir_smooth: Vector3::new(0.0, 0.01, -1.0),

            mouse_world_pos: None,
            screen_center_world_pos: None,

            heightmap_editor: heightmap_editor::State::new(),

            frame_minus_one: Frame::new(),
            frame_zero: Frame::new(),
            frame_zero_time_received: Instant::now(),

            kbots: HashMap::new(),
            kinematic_projectiles: HashMap::new(),
            server_sec: 0.0,

            in_screen: Vec::new(),
            selected: HashSet::new(),

            players: HashMap::new(),
            my_player_id: None,

            start_time: Instant::now(),
            last_frame: Instant::now(),
            fps: 144,
        }
    }

    pub fn handle_new_frame(&mut self, frame: Frame) {
        self.frame_zero_time_received = Instant::now();
        self.frame_minus_one = std::mem::replace(&mut self.frame_zero, frame);

        self.selected = self
            .selected
            .difference(&self.frame_zero.kbots_dead.iter().map(|e| e.value).collect())
            .copied()
            .collect();
    }

    pub fn interpolate(&mut self) {
        let elapsed = self.frame_zero_time_received.elapsed().as_secs_f64();
        //elapsed normalize between 0 and 1 if frame arrives every 100ms (0.1s)
        let lambda = (elapsed / 0.1) as f32;
        let i0 = lambda;
        let im = 1.0 - lambda;

        self.server_sec =
            (self.frame_zero.number as f32 * i0 + self.frame_minus_one.number as f32 * im) / 10.0;

        log::trace!("server_sec {}", self.server_sec);

        self.kbots.clear();
        for kbot_0 in self.frame_zero.kbots.values() {
            let to_insert = {
                if let Some(kbot_m) = self.frame_minus_one.kbots.get(&kbot_0.id) {
                    let position = kbot_0.position * i0 + (im * kbot_m.position).coords;
                    let dir = kbot_0.dir * i0 + kbot_m.dir * im;
                    let life = (kbot_0.life as f64 * i0 as f64 + kbot_m.life as f64 * im as f64)
                        .round() as i32;
                    let kbot = KBot {
                        position,
                        dir,
                        life,
                        ..*kbot_0
                    };

                    kbot
                } else {
                    //No interpolation possible, taking last data point
                    kbot_0.clone()
                }
            };

            self.kbots.insert(to_insert.id, to_insert);
        }

        self.kinematic_projectiles.clear();
        for kine_0 in self.frame_zero.kinematic_projectiles.values() {
            let to_insert = {
                if let Some(kine_m) = self.frame_minus_one.kinematic_projectiles.get(&kine_0.id) {
                    let position = kine_0.position() * i0 + (im * kine_m.position()).coords;

                    let mut positions = vec![position];
                    positions.extend(&kine_0.positions.clone());
                    let kine = KinematicProjectile {
                        positions,
                        ..*kine_0
                    };

                    kine
                } else {
                    //No interpolation possible, taking last data point
                    kine_0.clone()
                }
            };

            self.kinematic_projectiles.insert(to_insert.id, to_insert);
        }

        // self.kbots = self.frame_zero.kbots.clone();
        // self.kinematic_projectiles = self.frame_zero.kinematic_projectiles.clone();
        self.players = self.frame_zero.players.clone();
    }

    pub fn my_player(&self) -> Option<&Player> {
        self.my_player_id
            .map(|id| self.players.get(&id))
            .unwrap_or(None)
    }
    pub fn my_player_mut(&mut self) -> Option<&mut Player> {
        match self.my_player_id {
            Some(id) => self.players.get_mut(&id),
            None => None,
        }
    }
}
