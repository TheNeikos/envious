/// An error happened during deserialization
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum EnvDeserializationError {
    /// An error occurred during deserialization with serde
    #[error("An error occured during deserialization: {}", .0)]
    GenericDeserialization(String),

    /// An unsupported variant was tried to be deserialized. Only structs and maps are currently
    /// supported.
    #[error("An unsupported variant was tried to be deserialized. Only structs and maps are currently supported.")]
    UnsupportedValue,

    /// An invalid nested value was given. (Usually only a simple value is allowed)
    #[error("Tried to nest values while a simple value was expected")]
    InvalidNestedValues,

    /// Invalid nesting detected for the given paths ending in the given array
    #[error("Invalid nesting detected for paths ending in: {:?}", .0)]
    InvalidEnvNesting(Vec<String>),
}

impl serde::de::Error for EnvDeserializationError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::GenericDeserialization(msg.to_string())
    }
}
