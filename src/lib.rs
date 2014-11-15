//! A binding for Open Asset Import Library (assimp)

#![feature(struct_variant)]
#![feature(unsafe_destructor)]
#![allow(dead_code)]

extern crate libc;

pub use types::{Vector2D, Vector3D, Color3D, Color4D, Matrix3x3, Matrix4x4,
                Quaternion, Plane, Ray, AiString};
pub use scene::{Scene};

pub mod animation;
pub mod camera;
pub mod info;
pub mod light;
pub mod material;
pub mod mesh;
pub mod scene;
pub mod texture;
pub mod types;
pub mod postprocess;

mod util;
mod import;
mod export;
mod fileio;
