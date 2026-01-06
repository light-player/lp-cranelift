//! Runtime types and traits

pub mod frame_time;
pub mod lifecycle;
pub mod contexts;

pub use frame_time::FrameTime;
pub use lifecycle::NodeLifecycle;
pub use contexts::{
    FixtureRenderContext, InitContext, OutputRenderContext, ShaderRenderContext,
    TextureRenderContext,
};

