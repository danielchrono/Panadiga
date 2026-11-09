//! Panasonic DMR-EH55 HDD Image Inspection, Partition Extraction & MEIHDFS Analysis
//!
//! Ultra-high performance zero-copy analysis utilizing `memmap2::Mmap`.

pub mod inspect;
pub mod meihdfs;
pub mod mkfs;
pub mod partitions;
pub mod types;
pub mod utils;
pub mod writer;

pub use inspect::inspect_hdd_image;
pub use meihdfs::{decode_descriptors, decode_meihdfs_descriptors};
pub use mkfs::*;
pub use partitions::extract_hdd_partitions;
pub use types::*;
pub use utils::{
    calculate_entropy, export_report_to_json, open_image_buffer, open_mmap, ImageBuffer,
};
pub use writer::*;
