use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
};
use ops::atan2;

#[derive(Debug, Component)]
pub enum Shape {
    Circle(f32),
    Rect(f32, f32),
}

impl Shape {
    pub fn intersects(&self, position: Vec2, other: &Self, other_position: Vec2) -> bool {
        match self {
            Shape::Circle(radius) => {
                let a = BoundingCircle::new(position, *radius);
                match other {
                    Shape::Circle(radius) => {
                        a.intersects(&BoundingCircle::new(other_position, *radius))
                    }
                    Shape::Rect(width, height) => a.intersects(&Aabb2d::new(
                        other_position,
                        Vec2::new(width / 2.0, height / 2.0),
                    )),
                }
            }
            Shape::Rect(width, height) => {
                let a = Aabb2d::new(position, Vec2::new(width / 2.0, height / 2.0));
                match other {
                    Shape::Circle(radius) => {
                        a.intersects(&BoundingCircle::new(other_position, *radius))
                    }
                    Shape::Rect(width, height) => a.intersects(&Aabb2d::new(
                        other_position,
                        Vec2::new(width / 2.0, height / 2.0),
                    )),
                }
            }
        }
    }

    pub(crate) fn closest_point(
        &self,
        position: Vec2,
        other: &Shape,
        other_position: Vec2,
    ) -> Vec2 {
        match self {
            Shape::Circle(radius) => {
                let a = BoundingCircle::new(position, *radius);
                a.closest_point(other_position)
            }
            Shape::Rect(width, height) => {
                let a = Aabb2d::new(position, Vec2::new(width / 2.0, height / 2.0));
                a.closest_point(other_position)
            }
        }
    }
}
#[derive(Debug, Component)]
pub struct SpringConstraint {
    pub other: Entity,
    pub strength: f32,
    pub length: f32,
}

#[derive(Debug, Component)]
pub struct DynamicObject {
    pub velocity: Vec2,
    pub mass: f32,
    pub forces: Vec<Force>,
}
impl DynamicObject {
    pub fn new(mass: f32) -> Self {
        Self {
            velocity: Vec2::default(),
            forces: Vec::default(),
            mass,
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct Force {
    pub magnitude: f32,
    pub angle: f32,
    pub(crate) color: Option<Color>,
}
impl Force {
    pub(crate) fn from_x_and_y(x: f32, y: f32, color: Option<Color>) -> Self {
        Self {
            magnitude: (x.powi(2) + y.powi(2)).sqrt(),
            angle: atan2(y, x),
            color, // color:
        }
    }
    pub(crate) fn from_magnitude_and_angle(
        magnitude: f32,
        angle: f32,
        color: Option<Color>,
    ) -> Self {
        Self {
            magnitude,
            angle,
            color,
        }
    }
}

#[derive(Debug, Component)]
pub struct StaticObject {}
