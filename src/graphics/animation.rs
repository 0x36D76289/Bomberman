use std::{collections::HashMap, time::Duration};
use glam::{Mat4, Quat, Vec3};
use ufbx::Scene;
use vulkano::{buffer::{Subbuffer}};
use crate::graphics::{object::Object, renderer::animation_vs::JointsUbo};

/// A component of the skeleton, can also be refered as a bone
#[derive(Debug, Clone, PartialEq)]
pub struct Joint {
    pub id: u32,
    pub name: String,
    // transform in model space
    pub animated_transform: Mat4,
    pub children: Vec<Joint>,
    pub local_bind_transform: Mat4,
    pub inverse_bind_transform: Mat4,
}

impl Joint {
    pub fn new(id: u32, name: String, local_bind_transform: Mat4) -> Self {
        Self {
            id,
            name,
            animated_transform: Mat4::default(),
            children: Vec::new(),
            local_bind_transform,
            inverse_bind_transform: Mat4::default()
        }
    }

    pub fn add_child(&mut self, child: Joint) {
        self.children.push(child);
    }

    pub fn calc_inverse_bind_transform(&mut self, parent_bind_transform: Mat4) {
        let bind_transform = Mat4::mul_mat4(&parent_bind_transform, &self.local_bind_transform);
        self.inverse_bind_transform = bind_transform.inverse();
        for child in self.children.iter_mut() {
            child.calc_inverse_bind_transform(bind_transform);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Animation {
    pub length: Duration,
    pub frames: Vec<KeyFrame>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyFrame {
    pub time_stamp: Duration,
    // the transform for each joint in relation with its name
    // these transforms are in relation to to their parent joint, they are not in model space
    pub pose: HashMap<String, JointTransform>
}

#[derive(Debug, Clone, PartialEq)]
pub struct JointTransform {
    pub position: Vec3,
    pub rotation: Quat,
}

impl JointTransform {
    pub fn get_transform(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.position)
    }

    pub fn interpolate(frame_a: &JointTransform, frame_b: &JointTransform, progression: f32) -> JointTransform {
        let pos = {
            let position_a = frame_a.position;
            let position_b = frame_b.position;
            Vec3 {
                x: position_a.x + (position_b.x - position_a.x) * progression,
                y: position_a.y + (position_b.y - position_a.y) * progression,
                z: position_a.z + (position_b.z - position_a.z) * progression,
            }
        };

        let rot = frame_a.rotation.slerp(frame_b.rotation, progression);

        JointTransform {
            position: pos,
            rotation: rot
        }
    }
}

/// Applies an Animation to an AnimatedObject
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Animator {
    current_anim: Option<Animation>,
    animation_time: Duration,
    current_frame: usize
}

impl Animator {
    pub fn new(animation: Option<Animation>) -> Self {
        Self {
            current_anim: animation,
            animation_time: Duration::new(0, 0),
            current_frame: 0
        }
    }

    pub fn do_animation(&mut self, animation: Animation) {
        self.animation_time = Duration::new(0, 0);
        self.current_anim = Some(animation);
        self.current_frame = 0;
    }

    pub fn update(&mut self, delta_time: f32, root_joint: &mut Joint) {
        if self.current_anim.is_none() {
            return;
        }

        self.increase_animation_time(
            Duration::try_from_secs_f32(delta_time)
                .unwrap_or(Duration::new(0, 0))
        );

        let current_pose = self.calc_current_anim_pos();
        Self::apply_pose_to_joints(&current_pose, root_joint, Mat4::IDENTITY);
    }

    pub fn increase_animation_time(&mut self, time: Duration) {
        let current_anim = match &self.current_anim {
            Some(anim) => anim,
            None => {return;}
        };

        self.animation_time += time;
        if self.animation_time > current_anim.length {
            self.animation_time = self.animation_time - current_anim.length;
            self.current_frame = 0;
        }
        while self.current_frame + 1 < current_anim.frames.len() - 1
            && current_anim.frames[self.current_frame + 1].time_stamp < self.animation_time
        {
            self.current_frame += 1;
        }
    } 

    fn calc_current_anim_pos(&self) -> HashMap<String, Mat4> {
        let current_anim = self.current_anim.as_ref().unwrap();

        let (frame1, frame2) = {
            let next_frame_id = if self.current_frame + 1 >= current_anim.frames.len() {self.current_frame + 1} else {0};
            (&current_anim.frames[self.current_frame], &current_anim.frames[next_frame_id])
        };

        let progression = self.calc_progression(&frame1, &frame2);
        self.interpolate_poses(&frame1, &frame2, progression)
    }

    fn calc_progression(&self, frame1: &KeyFrame, frame2: &KeyFrame) -> f32 {
        let total_time = frame2.time_stamp - frame1.time_stamp;
        let current_time = self.animation_time - frame1.time_stamp;
        current_time.as_secs_f32() / total_time.as_secs_f32()
    }

    fn interpolate_poses(&self, frame1: &KeyFrame, frame2: &KeyFrame, progression: f32) -> HashMap<String, Mat4> {
        let mut current_pos = HashMap::new();
        for (joint_name, frame1_transform) in frame1.pose.iter() {
            let frame2_transform = match frame2.pose.get(joint_name) {
                Some(transform) => transform,
                None => {continue;}
            };
            let current_transform = JointTransform::interpolate(frame1_transform, frame2_transform, progression);
            current_pos.insert(joint_name.clone(), current_transform.get_transform());
        }
        current_pos
    }

    fn apply_pose_to_joints(current_pose: &HashMap<String, Mat4>, joint: &mut Joint, parent_transform: Mat4) {
        let current_local_transform = match current_pose.get(&joint.name) {
            Some(transform) => transform,
            None => {return;}
        };

        let mut current_transform = parent_transform.mul_mat4(current_local_transform);
        for child in joint.children.iter_mut() {
            Self::apply_pose_to_joints(current_pose, child, current_transform);
        }
        current_transform = current_transform.mul_mat4(&joint.inverse_bind_transform);
        joint.animated_transform = current_transform;
    }
}

