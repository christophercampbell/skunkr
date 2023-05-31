include!(concat!(env!("OUT_DIR"), "/skunkr.rs"));

use tonic_reflection::server::{Error, ServerReflection, ServerReflectionServer};

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("skunkr");

pub fn create_reflection_server() -> Result<ServerReflectionServer<impl ServerReflection>, Error> {
    return tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build();
}