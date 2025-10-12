#[derive(Debug, Default)]
pub struct UserSettings {
    /// if true, the text typed in the search bar will ignore the capitalization the search
    pub(crate) ignore_search_case: bool,
}
