use std::any::type_name_of_val;
use std::fmt::{Debug, Display, Formatter};

use serde::Deserialize;

/// Represents a secret part of the configuration.
#[derive(Deserialize, Eq, PartialEq)]
pub struct SecretConfig<T> {
    /// The wrapped value of this secret config.
    pub value: T,
}

impl<T> Debug for SecretConfig<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let type_name = type_name_of_val(&self.value);
        let result = format!("Secret({type_name})");
        f.write_str(&result)
    }
}

mod tests {
    use crate::config::secret_config::SecretConfig;

    #[test]
    fn debug_redacts_wrapped_value() {
        let secret_str = SecretConfig {
            value: "very secret value",
        };
        let result = format!("{secret_str:?}");
        assert_eq!(result, "Secret(&str)");

        let secret_i32 = SecretConfig { value: 1234 };
        let result = format!("{secret_i32:?}");
        assert_eq!(result, "Secret(i32)");
    }
}
