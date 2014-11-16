//! Defines the data structures in which the imported scene is returned.

use libc::{c_uint, c_void};
use std::mem;

use animation::Animation;
use camera::Camera;
use light::Light;
use material::Material;
use mesh::Mesh;
use texture::Texture;
use types::{Matrix4x4, AiString, MemoryInfo};
use util::{ptr_ptr_to_slice, ptr_to_slice};
use postprocess::PostProcessSteps;
use ffi;


/// A node in the imported hierarchy.
///
/// Each node has name, a parent node (except for the root node),
/// a transformation relative to its parent and possibly several child nodes.
/// Simple file formats don't support hierarchical structures - for these formats
/// the imported scene does consist of only a single root node without children.
#[repr(C)]
pub struct Node {
    /// The name of the node.
    ///
    /// The name might be empty (length of zero) but all nodes which
    /// need to be accessed afterwards by bones or anims are usually named.
    /// Multiple nodes may have the same name, but nodes which are accessed
    /// by bones (see #aiBone and #aiMesh::mBones) *must* be unique.
    ///
    /// Cameras and lights are assigned to a specific node name - if there
    /// are multiple nodes with this name, they're assigned to each of them.
    ///
    /// There are no limitations regarding the characters contained in
    /// this text. You should be able to handle stuff like whitespace, tabs,
    /// linefeeds, quotation marks, ampersands, ... .
    ///
    pub name: AiString,

    /// The transformation relative to the node's parent.
    pub transformation: Matrix4x4,

    /// Parent node. NULL if this node is the root node.
    parent: *mut Node,

    /// The number of child nodes of this node.
    pub num_children: c_uint,

    /// The child nodes of this node. NULL if mNumChildren is 0.
    children: *mut*mut Node,

    /// The number of meshes of this node.
    pub num_meshes: c_uint,

    /// The meshes of this node. Each entry is an index into the mesh.
    meshes: *mut c_uint,
}

impl Node {
    /// Get the parent node. Returns None if this node is the root node.
    pub fn get_parent(&self) -> Option<&Node> {
        if self.parent.is_null() {
            None
        } else {
            unsafe {
                Some(&*self.parent)
            }
        }
    }

    pub fn get_children(&self) -> &[&Node] {
        unsafe { ptr_ptr_to_slice(self.children, self.num_children as uint) }
    }

    pub fn get_meshes(&self) -> &[u32] {
        unsafe { ptr_to_slice(self.meshes, self.num_meshes as uint) }
    }
}

#[deriving(Show)]
#[repr(C, u32)]
pub enum SceneFlags {
    /// Specifies that the scene data structure that was imported is not complete.
    ///
    /// This flag bypasses some internal validations and allows the import
    /// of animation skeletons, material libraries or camera animation paths
    /// using Assimp. Most applications won't support such data.
    SceneFlags_INCOMPLETE = 0x1,

    ///  This flag is set by the validation postprocess-step if the validation is
    ///  successful.
    ///
    ///  In a validated scene you can be sure that any cross references in the
    ///  data structure (e.g.  vertex indices) are valid.
    SceneFlags_VALIDATED = 0x2,

    /// This flag is set by the validation postprocess-step if the validation is
    /// successful but some issues have been found.
    ///
    /// This can for example mean that a texture that does not exist is referenced
    /// by a material or that the bone weights for a vertex don't sum to 1.0 ... .
    /// In most cases you should still be able to use the import. This flag could
    /// be useful for applications which don't capture Assimp's log output.
    SceneFlags_VALIDATION_WARNING = 0x4,

    /// This flag is currently only set by the aiProcess_JoinIdenticalVertices step.
    ///
    /// It indicates that the vertices of the output meshes aren't in the internal
    /// verbose format anymore. In the verbose format all vertices are unique,
    /// no vertex is ever referenced by more than one face.
    SceneFlags_NON_VERBOSE_FORMAT = 0x8,

    /// Denotes pure height-map terrain data.
    ///
    /// Pure terrains usually consist of quads, sometimes triangles, in a regular
    /// grid. The x,y coordinates of all vertex positions refer to the x,y
    /// coordinates on the terrain height map, the z-axis stores the elevation at
    /// a specific point.
    ///
    /// TER (Terragen) and HMP (3D Game Studio) are height map formats.
    ///
    /// Note: Assimp is probably not the best choice for loading *huge* terrains
    /// - fully triangulated data takes extremely much free store and should be
    /// avoided as long as possible (typically you'll do the triangulation when
    /// you actually need to render it).
    SceneFlags_TERRAIN = 0x10,
}

/// Objects of this class are generally maintained and owned by Assimp, not
/// by the caller. You shouldn't want to instance it, nor should you ever try to
/// delete a given scene on your own.
#[repr(C)]
pub struct RawScene {
    /// Any combination of the AI_SCENE_FLAGS_XXX flags.
    ///
    /// By default this value is 0, no flags are set. Most applications will
    /// want to reject all scenes with the AI_SCENE_FLAGS_INCOMPLETE bit set.
    pub flags: c_uint,

    /// The root node of the hierarchy.
    ///
    /// There will always be at least the root node if the import
    /// was successful (and no special flags have been set).
    /// Presence of further nodes depends on the format and content
    /// of the imported file.
    pub root_node: *mut Node,

    /// The number of meshes in the scene.
    pub num_meshes: c_uint,

    /// The array of meshes.
    ///
    /// Use the indices given in the aiNode structure to access
    /// this array. The array is mNumMeshes in size. If the
    /// AI_SCENE_FLAGS_INCOMPLETE flag is not set there will always
    /// be at least ONE material.
    pub meshes: *mut*mut Mesh,

    /// The number of materials in the scene.
    pub num_materials: c_uint,

    /// The array of materials.
    ///
    /// Use the index given in each aiMesh structure to access this
    /// array. The array is mNumMaterials in size. If the
    /// AI_SCENE_FLAGS_INCOMPLETE flag is not set there will always
    /// be at least ONE material.
    pub materials: *mut*mut Material,

    /// The number of animations in the scene.
    pub num_animations: c_uint,

    /// The array of animations.
    ///
    /// All animations imported from the given file are listed here.
    /// The array is mNumAnimations in size.
    pub animations: *mut*mut Animation,

    /// The number of textures embedded into the file
    pub num_textures: c_uint,

    /// The array of embedded textures.
    ///
    /// Not many file formats embed their textures into the file.
    /// An example is Quake's MDL format (which is also used by
    /// some GameStudio versions)
    pub textures: *mut*mut Texture,

    /// The number of light sources in the scene. Light sources
    /// are fully optional, in most cases this attribute will be 0
    pub num_lights: c_uint,

    /// The array of light sources.
    ///
    /// All light sources imported from the given file are listed here.  Light
    /// sources are fully optional, in most cases this array will contain 0.
    pub lights: *mut*mut Light,

    /// The number of cameras in the scene. Cameras
    /// are fully optional, in most cases this attribute will be 0
    pub num_cameras: c_uint,

    /// The array of cameras.
    ///
    /// All cameras imported from the given file are listed here.
    /// The array is mNumCameras in size. The first camera in the
    /// array (if existing) is the default camera view into
    /// the scene.
    pub cameras: *mut*mut Camera,

    /// Internal data, do not touch
    pub private: *mut c_void,
}

/// The root structure of the imported data.
///
/// Everything that was imported from the given file can be accessed from here.
pub struct Scene<'a> {
    // Note we use this struct to wrap the RawScene so that we
    // can aiReleaseImport gets dropped.
    raw_scene: &'a RawScene,
    /// Any combination of the flags in scene::SceneFlags.
    ///
    /// By default this value is 0, no flags are set. Most applications will
    /// want to reject all scenes with the SceneFlags_INCOMPLETE bit set.
    pub flags: c_uint,

    /// The number of meshes in the scene.
    pub num_meshes: c_uint,

    /// The number of materials in the scene.
    pub num_materials: c_uint,

    /// The number of animations in the scene.
    pub num_animations: c_uint,

    /// The number of textures embedded into the file
    pub num_textures: c_uint,

    /// The number of light sources in the scene. Light sources
    /// are fully optional, in most cases this attribute will be 0
    pub num_lights: c_uint,

    /// The number of cameras in the scene. Cameras
    /// are fully optional, in most cases this attribute will be 0
    pub num_cameras: c_uint,
}

impl<'a> Scene<'a> {
    pub fn from_file(fname: &str, flags: c_uint) -> Scene {
        // TODO FIXME DONT FORGET, CHECK RESULTS!!!! FIXME TODO
        unsafe {
            let raw = fname.with_c_str(|s| ffi::aiImportFile(s, flags) );
            Scene::from_raw_scene(raw)
        }
    }

    pub unsafe fn from_raw_scene(raw: *const RawScene) -> Scene<'a> {
        let raw = &*raw;
        Scene {
            raw_scene: raw,
            flags: raw.flags,
            num_meshes: raw.num_meshes,
            num_materials: raw.num_materials,
            num_animations: raw.num_animations,
            num_textures: raw.num_textures,
            num_lights: raw.num_lights,
            num_cameras: raw.num_cameras,
        }
    }

    /// Get the root node of the hierarchy.
    ///
    /// There will always be a root node if the import
    /// was successful (and no special flags have been set).
    /// Presence of further nodes depends on the format and content
    /// of the imported file.
    pub fn get_root_node(&self) -> &Node {
        unsafe {
            &*(self.raw_scene.root_node)
        }
    }

    /// Get the array of animations.
    ///
    /// All animations imported from the given file are listed here.
    pub fn get_animations(&self) -> &[&Animation] {
        unsafe { ptr_ptr_to_slice(self.raw_scene.animations,
                                  self.raw_scene.num_animations as uint) }
    }

    /// Get the array of meshes.
    ///
    /// Use the indices given in the Node structure to access
    /// this array. If the AI_SCENE_FLAGS_INCOMPLETE flag is not set there
    /// will always be at least ONE mesh.
    pub fn get_meshes(&self) -> &[&Mesh] {
        unsafe { ptr_ptr_to_slice(self.raw_scene.meshes,
                                  self.raw_scene.num_meshes as uint) }
    }

    /// Get the array of light sources.
    ///
    /// All light sources imported from the given file are listed here.  Light
    /// sources are fully optional, in most cases this array will contain 0.
    pub fn get_lights(&self) -> &[&Light] {
        unsafe { ptr_ptr_to_slice(self.raw_scene.lights,
                                  self.raw_scene.num_lights as uint) }
    }

    /// Get the array of cameras.
    ///
    /// All cameras imported from the given file are listed here.
    /// The first camera in the array (if existing) is the default camera view
    /// into the scene.
    pub fn get_cameras(&self) -> &[&Camera] {
        unsafe { ptr_ptr_to_slice(self.raw_scene.cameras,
                                  self.raw_scene.num_cameras as uint) }
    }

    /// Get the array of materials.
    ///
    /// Use the index given in each Mesh structure to access this
    /// array. If the SceneFlags_INCOMPLETE flag is not set there will
    /// always be at least ONE material.
    pub fn get_materials(&self) -> &[&Material] {
        unsafe { ptr_ptr_to_slice(self.raw_scene.materials,
                                  self.raw_scene.num_materials as uint) }
    }

    /// Get the array of embedded textures.
    ///
    /// Not many file formats embed their textures into the file.
    /// An example is Quake's MDL format (which is also used by
    /// some GameStudio versions)
    pub fn get_textures(&self) -> &[&Texture] {
        unsafe { ptr_ptr_to_slice(self.raw_scene.textures,
                                  self.raw_scene.num_textures as uint) }
    }

    pub fn get_memory_info(&self) -> MemoryInfo {
        unsafe {
            let mut mem_info = mem::zeroed();
            ffi::aiGetMemoryRequirements(self.raw_scene, &mut mem_info);
            mem_info
        }
    }

    pub fn apply_postprocessing(&mut self,
                                flags: &[PostProcessSteps])
                                -> Result<(), &str> {
        unsafe {
            let flags = flags.iter().fold(0, |x, &y| x | y as u32);
            let scene = ffi::aiApplyPostProcessing(self.raw_scene,
                                                   flags);
            if scene.is_null() {
                Err("Post processing failed")
            } else {
                Ok(())
            }
        }
    }
}

#[unsafe_destructor]
impl<'a> Drop for Scene<'a> {
    fn drop(&mut self) {
        unsafe { ffi::aiReleaseImport(mem::transmute(self.raw_scene)) }
    }
}

#[cfg(test)]
mod test {
    use super::Scene;
    #[test]
    fn test_import() {
        let scene = Scene::from_file("cube.dae", 0);

        println!("mem_info {}", scene.get_memory_info());

        for node in scene.get_root_node().get_children().iter() {
            println!("node: {}", node.name);
        }
        for mesh in scene.get_meshes().iter() {
            println!("mesh.num_vertices: {}", mesh.num_vertices);
            for vert in mesh.get_vertices().iter() {
                println!("vert: {}", vert);
            }
        }
    }
}
