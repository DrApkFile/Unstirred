use std::collections::BTreeMap;

/// The flags one command was given, by flag name including the leading dashes.
pub type Flags = BTreeMap<String, String>;

/// Reads `--flag value` pairs, rejecting anything the command does not declare, a flag without a
/// value, and the same flag twice.
pub fn parse_flags(args: &[String], allowed: &[&str]) -> Result<Flags, String> {
    let mut flags = Flags::new();
    let mut remaining = args.iter();
    while let Some(flag) = remaining.next() {
        if !allowed.contains(&flag.as_str()) {
            return Err(format!("unknown argument {flag:?}"));
        }
        let value = remaining.next().ok_or_else(|| format!("{flag} needs a value"))?;
        if flags.insert(flag.clone(), value.clone()).is_some() {
            return Err(format!("{flag} was given twice"));
        }
    }
    Ok(flags)
}

pub fn required<'a>(flags: &'a Flags, flag: &str) -> Result<&'a str, String> {
    flags
        .get(flag)
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} <value> is required"))
}

/// Removes value-less flags, returning which were present; a flag given twice is an error.
pub fn strip_bool_flags(args: &[String], flags: &[&str]) -> Result<(Vec<String>, Vec<String>), String> {
    let mut stripped = Vec::with_capacity(args.len());
    let mut present = Vec::new();
    for arg in args {
        if flags.contains(&arg.as_str()) {
            if present.contains(arg) {
                return Err(format!("{arg} was given twice"));
            }
            present.push(arg.clone());
        } else {
            stripped.push(arg.clone());
        }
    }
    Ok((stripped, present))
}

/// Removes every `--flag value` pair of one repeatable flag, keeping the values in order.
pub fn strip_repeatable_flags(args: &[String], flag: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let mut stripped = Vec::with_capacity(args.len());
    let mut values = Vec::new();
    let mut remaining = args.iter();
    while let Some(arg) = remaining.next() {
        if arg == flag {
            let value = remaining.next().ok_or_else(|| format!("{flag} needs a value"))?;
            values.push(value.clone());
        } else {
            stripped.push(arg.clone());
        }
    }
    Ok((stripped, values))
}

pub fn optional<'a>(flags: &'a Flags, flag: &str) -> Option<&'a str> {
    flags.get(flag).map(String::as_str)
}
