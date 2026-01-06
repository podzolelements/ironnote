const TEXT_EXT: &[&str] = &["txt", "text", "md", "TXT", "TEXT", "MD"];
pub const TEXT_EXT_LIST: &[(&str, &[&str])] = &[("Plaintext", TEXT_EXT)];

const JSON_EXT: &[&str] = &["json", "JSON"];
pub const JSON_EXT_LIST: &[(&str, &[&str])] = &[("JSON", JSON_EXT)];

const DIC_EXT: &[&str] = &["dic", "DIC"];
pub const DIC_EXT_LIST: &[(&str, &[&str])] = &[("Hunspell Dictionary", DIC_EXT)];

const AFF_EXT: &[&str] = &["aff", "AFF"];
pub const AFF_EXT_LIST: &[(&str, &[&str])] = &[("Hunspell Affix Rules", AFF_EXT)];

/// constructs the constant extension data into allocated extension data for use with the FilePicker
pub fn build_extensions(extension_list: &[(&str, &[&str])]) -> Vec<(String, Vec<String>)> {
    extension_list
        .iter()
        .map(|(name, extensions)| {
            (
                name.to_string(),
                extensions
                    .iter()
                    .map(|extension| extension.to_string())
                    .collect(),
            )
        })
        .collect()
}
