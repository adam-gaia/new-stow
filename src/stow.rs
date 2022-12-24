use crate::filter::StowFilters;
use crate::link::Link;
use crate::settings::LinkSettings;
use crate::settings::Settings;
use anyhow::{bail, Result};
use lazy_static::lazy_static;
use log::{debug, trace};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct StowfileContents {
    vars: Option<Vec<String>>,
    stow: serde_yaml::Value,
}

#[derive(Debug)]
struct Stowfile<'a> {
    stows: serde_yaml::Value,
    variables: HashMap<String, String>,
    filters: &'a StowFilters,
    link_settings: &'a LinkSettings,
}

impl<'a> Stowfile<'a> {
    pub fn new(
        stowfile_path: &'a Path,
        filters: &'a StowFilters,
        link_settings: &'a LinkSettings,
    ) -> Result<Self> {
        let f = std::fs::File::open(stowfile_path)?;
        let contents: StowfileContents = serde_yaml::from_reader(f)?;

        // Save env vars for evaluation later
        let mut variables = HashMap::new();
        for (key, value) in env::vars() {
            variables.insert(key, value);
        }

        // If the stowfile contains variable definitions, add them to our collection
        if let Some(user_defined_variables) = &contents.vars {
            for var in user_defined_variables {
                let Ok((key, value)) = parse_variable(&var) else {
                    bail!("Malformatted stowfile variable '{}'", var)
                };
                variables.insert(key, value);
            }
        }
        Ok(Stowfile {
            stows: contents.stow,
            variables,
            filters,
            link_settings,
        })
    }

    fn traverse_sequence(
        &mut self,
        stowables: &serde_yaml::Sequence,
        current_src_path: &mut PathBuf,
    ) -> Result<Vec<Link<'a>>> {
        let mut collected_links = Vec::new();
        for stowable in stowables {
            if stowable.is_mapping() {
                let stowable = stowable.as_mapping().unwrap();

                if stowable.contains_key("src") {
                    // Found a link to make
                    let src = &stowable["src"];
                    if !src.is_string() {
                        bail!("Malformatted stowfile");
                    }
                    let src = src.as_str().unwrap();
                    current_src_path.push(src);

                    let links = &stowable["links"];
                    if !links.is_sequence() {
                        bail!("Malformatted stowfile");
                    }
                    let targets = links.as_sequence().unwrap();

                    for target in targets {
                        if !target.is_string() {
                            bail!("Malformatted stowfile");
                        }

                        let processed_src = var_replacement(
                            current_src_path.to_str().unwrap(),
                            &mut self.variables,
                        )?;
                        if !self.filters.check_src(&processed_src) {
                            continue;
                        }

                        let processed_target =
                            var_replacement(target.as_str().unwrap(), &mut self.variables)?;
                        if !self.filters.check_target(&processed_target) {
                            continue;
                        }

                        // Continue and save the link only if it passes the filters
                        let link = Link::new(processed_src, processed_target, &self.link_settings)?;
                        collected_links.push(link);
                        current_src_path.pop();
                    }
                } else {
                    // Just another mapping
                    let mut new_links = self.traverse_mapping(stowable, current_src_path)?;
                    collected_links.append(&mut new_links);
                }
            } else {
                bail!("Malformatted stowfile");
            }
        }
        Ok(collected_links)
    }

    fn traverse_value(
        &mut self,
        current_value: &serde_yaml::Value,
        current_src_path: &mut PathBuf,
    ) -> Result<Vec<Link<'a>>> {
        trace!("Current serde_yaml::Value: {:?}", current_value);
        let collected_links = if current_value.is_mapping() {
            self.traverse_mapping(current_value.as_mapping().unwrap(), current_src_path)?
        } else if current_value.is_sequence() {
            self.traverse_sequence(current_value.as_sequence().unwrap(), current_src_path)?
        } else {
            bail!("Malformatted stowfile");
        };
        Ok(collected_links)
    }

    pub fn get_links(&mut self, mut current_src_path: PathBuf) -> Result<Vec<Link<'a>>> {
        let stows = self.stows.clone(); // TODO: instead of dealing with the borrow-checker I just
                                        // cloned. Not a huge performace hit, but still should be
                                        // fixed later
        let links = self.traverse_value(&stows, &mut current_src_path)?;
        Ok(links)
    }

    fn traverse_mapping(
        &mut self,
        current_node: &serde_yaml::Mapping,
        current_src_path: &mut PathBuf,
    ) -> Result<Vec<Link<'a>>> {
        trace!("Current Mapping: {:?}", current_node);
        let mut collected_links = Vec::new();
        for child in current_node {
            if child.0.is_string() {
                let name = child.0;
                if !name.is_string() {
                    bail!("Malformatted stowfile");
                }
                current_src_path.push(name.as_str().unwrap());

                let mut new_links = self.traverse_value(child.1, current_src_path)?;
                collected_links.append(&mut new_links);
                current_src_path.pop();
            } else {
                bail!("Malformatted stowfile");
            }
        }
        Ok(collected_links)
    }
}

fn parse_variable(var: &str) -> Result<(String, String)> {
    // TODO: could all this be simplified with Nom instead?
    trace!("Parsing stowfile variable '{}'", var);
    let parts: Vec<&str> = var.split("=").collect();
    let (key, value) = match parts.len() {
        0 | 1 => {
            bail!("Could not parse variable");
        }
        2 => {
            let key = parts[0];
            let value = parts[1];
            (key.to_string(), value.to_string())
        }
        _ => {
            let key = parts[0];
            // Combine the rest of parts into the value. We wont juge if a '=' should or shouldn't
            // be in the variable's value
            let value = parts[1..].join("=");
            (key.to_string(), value)
        }
    };

    Ok((key, value))
}

fn var_replacement(text: &str, known_variables: &mut HashMap<String, String>) -> Result<String> {
    // Compile regex once with lazy static
    lazy_static! {
        // Regex matches shell-style variables '${VAR_NAME}' that contans any alphanumeric char or '_'
        // Var name is first capture group
        // TODO: invalid variable syntax should be caught. Like having a '-' in the var name
        static ref RE: Regex = Regex::new(r"\$\{([[:alpha:]_[0-9]]+)\}").unwrap();
    }
    let mut processed_text = text.to_string();
    for cap in RE.captures_iter(&text) {
        let full_match = &cap[0];
        let key_to_replace = &cap[1];
        trace!(
            "Found key of another variable '{}' in stowfile defined variable '{}'",
            key_to_replace,
            text
        );

        let Some(replacement_value) = known_variables.get(key_to_replace) else {
            bail!("Undefined variable '{}' found while processing '{}'", key_to_replace, text);
        };

        // Perform recursive variable replacement in our value
        let replacement_value = var_replacement(&replacement_value.clone(), known_variables)?;

        // Add this key/value pair back to our variable map
        known_variables.insert(key_to_replace.to_string(), replacement_value.clone());

        processed_text = processed_text.replace(full_match, &replacement_value);
        debug!("Replaced {:?} with {:?}", full_match, replacement_value);
    }
    Ok(processed_text)
}

pub struct Stow<'a> {
    links: Vec<Link<'a>>,
    settings: &'a Settings,
}
impl<'a> Stow<'a> {
    pub fn with_settings(settings: &'a Settings) -> Result<Self> {
        let mut stowfile = Stowfile::new(
            settings.stowfile_path(),
            settings.filters(),
            settings.link_settings(),
        )?;
        let links = stowfile.get_links(settings.current_working_dir().to_path_buf())?;
        Ok(Stow { links, settings })
    }

    pub fn stow(&self) -> Result<()> {
        trace!("Iterating over links for stowing: {:#?}", &self.links);
        for link in &self.links {
            link.link()?;
        }
        Ok(())
    }

    pub fn unstow(&self) -> Result<()> {
        trace!("Iterating over links for unstowing: {:#?}", &self.links);
        for link in &self.links {
            link.unlink()?;
        }
        Ok(())
    }

    pub fn restow(&self) -> Result<()> {
        trace!("Iterating over links for restowing: {:#?}", &self.links);
        for link in &self.links {
            // Ignore any unlinking errors
            link.unlink()?;
            link.link()?;
            // TODO: when running with --dry-run do not let link() write a warining when a link exists. unlink() would have taken care of that
        }
        Ok(())
    }
}
