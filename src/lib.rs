//! # Rust OBS Wrapper
//!
//! A safe wrapper around the OBS API, useful for creating OBS sources, filters and effects.
//!
//! ## Usage
//!
//! In your `Cargo.toml` file add the following section, substituting `<module-name>` for the name of
//! the module:
//!
//! ```toml
//! [dependencies]
//! obs-wrapper = "0.1"
//!
//! [lib]
//! name = "<module-name>"
//! crate-type = ["cdylib"]
//! ```
//!
//! The process for creating a plugin is:
//! 1. Create a struct that implements Module
//! 1. Create a struct that will store the plugin state
//! 1. Implement the required traits for the module
//! 1. Enable the traits which have been enabled in the module `load` method
//!
//! ~~~
//! use obs_wrapper::{
//!     // Everything required for modules
//!     prelude::*,
//!     // Everything required for creating a source
//!     source::*,
//!     // Macro for registering modules
//!     obs_register_module,
//!     // Macro for creating strings
//!     obs_string,
//! };
//!
//! // The module that will handle creating the source.
//! struct TestModule {
//!     context: ModuleContext
//! };
//!
//! // The source that will be shown inside OBS.
//! struct TestSource;
//!
//! // Implement the Sourceable trait for TestSource, this is required for each source.
//! // It allows you to specify the source ID and type.
//! impl Sourceable for TestSource {
//!     fn get_id() -> ObsString {
//!         obs_string!("test_source")
//!     }
//!
//!     fn get_type() -> SourceType {
//!         SourceType::FILTER
//!     }
//!
//!     fn create(create: &mut CreatableSourceContext<Self>, _source: SourceContext) -> Self {
//!         Self
//!     }
//! }
//!
//! // Allow OBS to show a name for the source
//! impl GetNameSource for TestSource {
//!     fn get_name() -> ObsString {
//!         obs_string!("Test Source")
//!     }
//! }
//!
//! // Implement the Module trait for TestModule. This will handle the creation of the source and
//! // has some methods for telling OBS a bit about itself.
//! impl Module for TestModule {
//!     fn new(context: ModuleContext) -> Self {
//!         Self { context }
//!     }
//!
//!     fn get_ctx(&self) -> &ModuleContext {
//!         &self.context
//!     }
//!    
//!     // Load the module - create all sources, returning true if all went well.
//!     fn load(&mut self, load_context: &mut LoadContext) -> bool {
//!         // Create the source
//!         let source = load_context
//!             .create_source_builder::<TestSource>()
//!             // Since GetNameSource is implemented, this method needs to be called to
//!             // enable it.
//!             .enable_get_name()
//!             .build();
//!    
//!         // Tell OBS about the source so that it will show it.
//!         load_context.register_source(source);
//!    
//!         // Nothing could have gone wrong, so return true.
//!         true
//!     }
//!    
//!     fn description() -> ObsString {
//!         obs_string!("A great test module.")
//!     }
//!
//!     fn name() -> ObsString {
//!         obs_string!("Test Module")
//!     }
//!
//!     fn author() -> ObsString {
//!         obs_string!("Bennett")
//!     }
//! }
//! ~~~
//!
//! ### Installing
//!
//! 1. Run `cargo build --release`
//! 2. Copy `/target/release/<module-name>.so` to your OBS plugins folder (`/usr/lib/obs-plugins/`)
//! 3. The plugin should be available for use from inside OBS

/// Raw bindings of OBS C API
pub use obs_sys;

/// `obs_data_t` handling
pub mod data;
/// Tools required for manipulating graphics in OBS
pub mod graphics;
mod hotkey;
/// Logger for logging to OBS console
pub mod log;
/// Tools for creating modules
pub mod module;
/// Tools for creating properties
pub mod properties;
/// Tools for creating sources
pub mod source;
/// Tools for creating outputs
pub mod output;
/// String macros
pub mod string;
/// FFI pointer wrapper
mod wrapper;

mod native_enum;

/// Re-exports of a bunch of popular tools
pub mod prelude {
    pub use crate::data::{DataArray, DataObj, FromDataItem};
    pub use crate::module::*;
    pub use crate::source::context::*;
    pub use crate::string::*;
}

#[derive(Debug)]
pub struct Error;

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OBS Error")
    }
}

pub type Result<T> = std::result::Result<T, Error>;
