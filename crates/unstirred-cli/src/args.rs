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

pub fn optional<'a>(flags: &'a Flags, flag: &str) -> Option<&'a str> {
    flags.get(flag).map(String::as_str)
}
