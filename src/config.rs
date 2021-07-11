pub static DEFAULT_CONFIG: Config = Config::default();

/// Gurgle command limitation configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    /// How many items can a gurgle expression contains
    pub max_item_count: u64,
    /// How many sided a dice can have
    pub max_dice_sides: u64,
    /// How many roll times(sum of all dice roll time) can a expression contains
    pub max_roll_times: u64,
    /// Max value of a number item
    pub max_number_item_value: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
}

impl Config {
    /// Default configure.
    ///
    /// - max item count: 20
    /// - max dice sides: 1000
    /// - max roll times: 100
    /// - max number item: 65536
    #[must_use]
    pub const fn default() -> Self {
        Self {
            max_item_count: 20,
            max_dice_sides: 1000,
            max_roll_times: 100,
            max_number_item_value: 65536,
        }
    }

    /// Give a new config, which only changes max item count with provided value.
    #[must_use]
    pub const fn max_item_count(self, c: u64) -> Self {
        Self {
            max_item_count: c,
            ..self
        }
    }

    /// Give a new config, which only changes max dice sides with provided value.
    #[must_use]
    pub const fn max_dice_sides(self, c: u64) -> Self {
        Self {
            max_dice_sides: c,
            ..self
        }
    }

    /// Give a new config, which only changes max roll times with provided value.
    #[must_use]
    pub const fn max_roll_times(self, c: u64) -> Self {
        Self {
            max_roll_times: c,
            ..self
        }
    }

    /// Give a new config, which only changes max number item value with provided value.
    #[must_use]
    pub const fn max_number_item_value(self, c: u64) -> Self {
        Self {
            max_number_item_value: c,
            ..self
        }
    }
}
