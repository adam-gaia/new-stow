use anyhow::{bail, Result};
use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use regex::Regex;

#[derive(Debug)]
pub struct Filter {
    regexes: Vec<Regex>,
}
impl Filter {
    fn new(filter_strings: Vec<String>) -> Self {
        let capacity = filter_strings.len();
        let regexes: Vec<Regex> = Vec::with_capacity(capacity);
        Filter { regexes }
    }
    fn matches(&self, input: &str) -> bool {
        for re in &self.regexes {
            if re.is_match(input) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug)]
pub struct FilterCombo {
    only: Option<Filter>,
    ignore: Option<Filter>,
}
impl FilterCombo {
    pub fn new(only_strings: Option<Vec<String>>, ignore_strings: Option<Vec<String>>) -> Self {
        let only = if let Some(only_strings) = only_strings {
            Some(Filter::new(only_strings))
        } else {
            None
        };

        let ignore = if let Some(ignore_strings) = ignore_strings {
            Some(Filter::new(ignore_strings))
        } else {
            None
        };
        FilterCombo { only, ignore }
    }

    fn check_against_filters(&self, input: &str) -> bool {
        if let Some(only) = &self.only {
            if !only.matches(&input) {
                info!("Input {} did not match any 'only' regexes. Ignoring", input);
                return false;
            }
        }
        if let Some(ignore) = &self.ignore {
            if ignore.matches(&input) {
                info!("Input {} matched an 'ignore' regex. Ignoring", input);
                return false;
            }
        }
        true
    }
}

#[derive(Debug)]
pub struct StowFilters {
    src_filter: Option<FilterCombo>,
    target_filter: Option<FilterCombo>,
    override_filter: Option<Filter>,
}
impl StowFilters {
    pub fn new(
        only: Option<Vec<String>>,
        ignore: Option<Vec<String>>,
        only_target: Option<Vec<String>>,
        ignore_target: Option<Vec<String>>,
        overrides: Option<Vec<String>>,
    ) -> Self {
        let src_filter = if only.is_some() || ignore.is_some() {
            Some(FilterCombo::new(only, ignore))
        } else {
            None
        };
        let target_filter = if only_target.is_some() || ignore_target.is_some() {
            Some(FilterCombo::new(only_target, ignore_target))
        } else {
            None
        };

        let override_filter = if let Some(overrides) = overrides {
            Some(Filter::new(overrides))
        } else {
            None
        };

        StowFilters {
            src_filter,
            target_filter,
            override_filter,
        }
    }

    pub fn check_src(&self, input: &str) -> bool {
        if let Some(src_filter) = &self.src_filter {
            src_filter.check_against_filters(input)
        } else {
            // Always pass when no filter exists
            true
        }
    }
    pub fn check_target(&self, input: &str) -> bool {
        if let Some(target_filter) = &self.target_filter {
            target_filter.check_against_filters(input)
        } else {
            // Always pass when no filter exists
            true
        }
    }

    pub fn check_target_override(&self, input: &str) -> bool {
        if let Some(override_filter) = &self.override_filter {
            override_filter.matches(input)
        } else {
            // Always fail when no target exists
            false
        }
    }
}
