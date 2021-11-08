//! show roll result in detail

use std::{
    borrow::Cow,
    fmt::{Display, Formatter, Write},
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use once_cell::sync::Lazy;

use crate::{
    checker::{Checker, Compare},
    expr::{Operator, PostProcessor},
    roll::{DiceRoll, GurgleRoll, ItemRoll, RollTree, RollTreeNode},
};

static WANTED_LANG: AtomicUsize = AtomicUsize::new(Language::EN.value());
static CUSTOM_LANG_PTR: AtomicPtr<OutputSpans> =
    AtomicPtr::new(std::ptr::null::<OutputSpans>() as *mut _);

static LANG: Lazy<Cow<'static, OutputSpans>> =
    Lazy::new(
        || match Language::from_value(WANTED_LANG.load(Ordering::SeqCst)) {
            Language::EN => Cow::Owned(OutputSpans::new_en()),
            Language::ZhCN => Cow::Owned(OutputSpans::new_zh_cn()),
            Language::Custom => Cow::Borrowed(Language::get_global_custom().unwrap()),
        },
    );

/// Rolling result detailed output language
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    /// English
    EN,
    /// Simplified Chinese
    ZhCN,
    /// Your custom language set, see `[Language::set_global_custom]`
    ///
    /// `[Language::set_global_custom]`: #method.set_global_custom
    Custom,
}

impl Language {
    const fn value(&self) -> usize {
        match *self {
            Self::EN => 0,
            Self::ZhCN => 1,
            Self::Custom => 999,
        }
    }

    fn from_value(value: usize) -> Self {
        match value {
            0 => Self::EN,
            1 => Self::ZhCN,
            999 => Self::Custom,
            _ => panic!("Can't convert {} into Language", value),
        }
    }

    /// Set a predefined language to be used globally
    ///
    /// You can call this method more then once, only the last value set before the first output will be used.
    ///
    /// ## Panics
    ///
    /// If `lang` is `Language::Custom`
    #[allow(clippy::needless_pass_by_value)] // because language is copy
    pub fn set_global(lang: Self) {
        if lang == Self::Custom {
            panic!("Call set global with custom is invalid, you should use `set_global_custom` instead");
        }

        WANTED_LANG.store(lang.value(), Ordering::SeqCst);
    }

    /// Set a custom language to be used globally
    ///
    /// You can call this method only once.
    ///
    /// ## Panics
    ///
    /// If you call this more than once
    pub fn set_global_custom(s: OutputSpans) {
        WANTED_LANG.store(Self::Custom.value(), Ordering::SeqCst);

        let p = Box::into_raw(Box::new(s));
        let last = CUSTOM_LANG_PTR.swap(p, Ordering::SeqCst);
        if !last.is_null() {
            panic!("`set_global_custom` can only be called once");
        }
    }

    fn get_global_custom() -> Option<&'static OutputSpans> {
        let p = CUSTOM_LANG_PTR.load(Ordering::SeqCst);
        if p.is_null() {
            None
        } else {
            Some(unsafe { &*p })
        }
    }
}

/// Some span of detailed output string of rolling result, differ between different language
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutputSpans {
    /// Sentence separator
    pub comma: Cow<'static, str>,
    /// Output before target
    pub target_is: Cow<'static, str>,
    /// Output word for success
    pub success: Cow<'static, str>,
    /// Output word for failed
    pub failed: Cow<'static, str>,
}

impl OutputSpans {
    /// Create a new output spans of predefined Zh-CN language
    #[must_use]
    pub fn new_en() -> Self {
        Self {
            comma: ", ".into(),
            target_is: "target is".into(),
            success: "success".into(),
            failed: "failed".into(),
        }
    }

    /// Create a new output spans of predefined English language
    #[must_use]
    pub fn new_zh_cn() -> Self {
        Self {
            comma: "，".into(),
            target_is: "目标为".into(),
            success: "通过".into(),
            failed: "失败".into(),
        }
    }
}

impl Display for Checker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.compare {
            Compare::Gte => ">=",
            Compare::Gt => ">",
            Compare::Lte => "<=",
            Compare::Lt => "<",
            Compare::Eq => "=",
        })?;
        f.write_fmt(format_args!("{}", self.target))
    }
}

impl Display for DiceRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (prefix, mid, postfix) = match self.post_processor() {
            PostProcessor::Sum => ("", "+", ""),
            PostProcessor::Avg => ("Avg[", ",", "]"),
            PostProcessor::Max => ("Max[", ",", "]"),
            PostProcessor::Min => ("Min[", ",", "]"),
        };

        f.write_char('(')?;
        f.write_str(prefix)?;
        let last = self.len() - 1;
        for (i, value) in self.points().iter().enumerate() {
            f.write_fmt(format_args!("{}", value))?;
            if i != last {
                f.write_str(mid)?;
            }
        }
        f.write_str(postfix)?;
        if self.post_processor() != PostProcessor::Sum {
            f.write_fmt(format_args!("={}", self.value()))?;
        }
        f.write_char(')')
    }
}

impl Display for ItemRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(x) => f.write_fmt(format_args!("{}", x)),
            Self::Dice(dice) => f.write_fmt(format_args!("{}", dice)),
            Self::Parentheses(e) => f.write_fmt(format_args!("({})", e.as_ref())),
        }
    }
}

impl Display for RollTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let op = match self.mid {
            Operator::Add => "+",
            Operator::Minus => "-",
            Operator::Multiply => "*",
        };
        f.write_fmt(format_args!("{} {} {}", self.left, op, self.right))
    }
}

impl Display for RollTreeNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Leaf(leaf) => f.write_fmt(format_args!("{}", leaf)),
            Self::Tree(tree) => f.write_fmt(format_args!("{}", tree)),
        }
    }
}

impl Display for GurgleRoll<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.expr()))?;

        if !std::matches!(self.expr(), RollTreeNode::Leaf(ItemRoll::Number(_))) {
            f.write_fmt(format_args!(" = {}", self.value()))?;
        }

        if let Some(c) = self.checker() {
            f.write_str(&LANG.comma)?;
            f.write_str(&LANG.target_is)?;
            f.write_fmt(format_args!("{}", c))?;
            f.write_str(&LANG.comma)?;
            if self.success().unwrap() {
                f.write_str(&LANG.success)?;
            } else {
                f.write_str(&LANG.failed)?;
            }
        }
        Ok(())
    }
}
