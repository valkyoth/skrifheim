use super::*;
use core::fmt;

macro_rules! redacted_debug {
    ($($name:ident),+ $(,)?) => {
        $(impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "(<redacted>)"))
            }
        })+
    };
}

redacted_debug!(
    KeyScope,
    KeyLifecycleState,
    KeyErasureReason,
    KeyLifecycleEventSequence,
    KeyErasureMetadata,
    KeyRotationPreflight,
    KeyMetadata,
);

#[cfg(test)]
mod tests;
