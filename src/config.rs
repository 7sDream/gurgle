use crate::error::CompileError;

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

pub struct Limit<'c> {
    config: &'c Config,
    pub item_count: u64,
    pub roll_times: u64,
}

impl<'c> Limit<'c> {
    pub const fn new(config: &'c Config) -> Self {
        Self {
            config,
            item_count: 0,
            roll_times: 0,
        }
    }

    pub fn inc_item_count(&mut self) -> Result<(), CompileError> {
        self.item_count += 1;
        self.check_item_count()
    }

    pub fn inc_roll_times(&mut self, times: u64) -> Result<(), CompileError> {
        self.roll_times += times;
        self.check_roll_times()
    }

    pub const fn check_number_item(&self, num: i64) -> Result<(), CompileError> {
        if num.abs() as u64 > self.config.max_number_item_value {
            return Err(CompileError::NumberItemOutOfRange);
        }
        Ok(())
    }

    pub const fn check_dice(&self, times: i64, sided: i64) -> Result<(), CompileError> {
        if times <= 0 || sided <= 0 {
            return Err(CompileError::DiceRollOrSidedNegative);
        }
        #[allow(clippy::cast_sign_loss)] // because times > 0
        if times as u64 > self.config.max_roll_times {
            return Err(CompileError::DiceRollTimesLimitExceeded);
        }
        #[allow(clippy::cast_sign_loss)] // because sided > 0
        if sided as u64 > self.config.max_dice_sides {
            return Err(CompileError::DiceSidedCountLimitExceeded);
        }

        Ok(())
    }

    const fn check_item_count(&self) -> Result<(), CompileError> {
        if self.item_count > self.config.max_item_count {
            Err(CompileError::ItemCountLimitExceeded)
        } else {
            Ok(())
        }
    }

    const fn check_roll_times(&self) -> Result<(), CompileError> {
        if self.roll_times > self.config.max_roll_times {
            Err(CompileError::DiceRollTimesLimitExceeded)
        } else {
            Ok(())
        }
    }

    #[allow(dead_code)]
    pub fn check(&self) -> Result<(), CompileError> {
        self.check_item_count().and(self.check_roll_times())
    }
}
