use bevy::prelude::*;

pub use input::RtsCameraControls;
use input::{grab_pan, pan, rotate, zoom};
use logic::{delta_grab, delta_pan, delta_zoom};
pub use logic::{DeltaGrab, DeltaPan, DeltaZoom};

use crate::RtsCameraSystemSet;

mod input;
mod logic;

/// An input-agnostic plugin that provides an easy to use interface for implementing your own
/// controller. For example, if you want to use an input manager you can use this plugin and
/// simply update resources with delta values directly from input events in order to control
/// the camera movement, rather than doing all the input -> 3D movement math yourself.
pub struct RtsCameraControlsPlugin;

impl Plugin for RtsCameraControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeltaZoom>()
            .init_resource::<DeltaPan>()
            .init_resource::<DeltaGrab>()
            .add_systems(
                Update,
                (delta_zoom, delta_pan, delta_grab)
                    .in_set(RtsCameraControlsSystemSet)
                    .before(RtsCameraSystemSet),
            );
    }
}

/// A system set containing the systems that convert simple deltas into camera movement.
/// If you modify any of the `Delta*` resources, you should do so before this system set
/// executes.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RtsCameraControlsSystemSet;

/// A plugin that includes built-in controls for RtsCamera. To get up and running quickly, before
/// switching to an input manager, add this plugin along with `RtsCameraControlsPlugin`, then
/// add `RtsCameraControls` to a `Camera3dBundle`. See documentation for `RtsCameraControls` for
/// the default controls.
pub struct RtsCameraControlsInputPlugin;

impl Plugin for RtsCameraControlsInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (zoom, pan, grab_pan, rotate).before(RtsCameraControlsSystemSet),
        );
    }
}
