//! Contains facilities that allows you diff public APIs between releases and
//! commits. [`cargo
//! public-api`](https://github.com/Enselic/cargo-public-api) contains
//! additional helpers for that.

use crate::PublicItem;

/// An item has changed in the public API. Two [`PublicItem`]s are considered
/// the same if their `path` is the same.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChangedPublicItem {
    /// How the item used to look.
    pub old: PublicItem,

    /// How the item looks now.
    pub new: PublicItem,
}

/// The return value of [`Self::between`]. To quickly get a sense of what it
/// contains, you can pretty-print it:
/// ```txt
/// println!("{:#?}", public_api_diff);
/// ```
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicItemsDiff {
    /// Items that have been removed from the public API. A MAJOR change, in
    /// semver terminology. Sorted.
    pub removed: Vec<PublicItem>,

    /// Items in the public API that has been changed. Generally a MAJOR change,
    /// but exceptions exist. For example, if the return value of a method is
    /// changed from `ExplicitType` to `Self` and `Self` is the same as
    /// `ExplicitType`.
    pub changed: Vec<ChangedPublicItem>,

    /// Items that have been added to public API. A MINOR change, in semver
    /// terminology. Sorted.
    pub added: Vec<PublicItem>,
}

impl PublicItemsDiff {
    /// Allows you to diff the public API between two arbitrary versions of a
    /// library, e.g. different releases. The input parameters `old` and `new`
    /// is the output of two different invocations of
    /// [`crate::public_api_from_rustdoc_json_str`].
    #[must_use]
    pub fn between(old_items: Vec<PublicItem>, new_items: Vec<PublicItem>) -> Self {
        let mut old_sorted = old_items;
        old_sorted.sort();

        let mut new_sorted = new_items;
        new_sorted.sort();

        eprintln!("old={:#?}", old_sorted);
        eprintln!("new={:#?}", new_sorted);

        // We can't implement this with sets, because different items might have
        // the same representations (e.g. because of limitations or bugs), so if
        // we used a Set, we would lose one or more of them.
        //
        // Our strategy is to only move items around, to reduce the risk of
        // duplicates and lost items.
        let mut removed: Vec<PublicItem> = vec![];
        let mut changed: Vec<ChangedPublicItem> = vec![];
        let mut added: Vec<PublicItem> = vec![];
        loop {
            match (old_sorted.pop(), new_sorted.pop()) {
                (None, None) => break,
                (Some(old), None) => {
                    removed.push(old);
                }
                (None, Some(new)) => {
                    added.push(new);
                }
                (Some(old), Some(new)) => {
                    if old != new && old.path == new.path {
                        // The same item, but there has been a change in type
                        changed.push(ChangedPublicItem { old, new });
                    } else {
                        match old.cmp(&new) {
                            std::cmp::Ordering::Less => {
                                added.push(new);

                                // Add it back and compare it again next
                                // iteration
                                old_sorted.push(old);
                            }
                            std::cmp::Ordering::Equal => {
                                // This is the same item, so just continue to
                                // the next pair
                                continue;
                            }
                            std::cmp::Ordering::Greater => {
                                removed.push(old);

                                // Add it back and compare it again next
                                // iteration
                                new_sorted.push(new);
                            }
                        }
                    }
                }
            }
        }

        // Make output predictable and stable
        removed.sort();
        changed.sort();
        added.sort();

        Self {
            removed,
            changed,
            added,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_and_only_item_removed() {
        let old = vec![item_with_path("foo")];
        let new = vec![];

        let actual = PublicItemsDiff::between(old, new);
        let expected = PublicItemsDiff {
            removed: vec![item_with_path("foo")],
            changed: vec![],
            added: vec![],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn single_and_only_item_added() {
        let old = vec![];
        let new = vec![item_with_path("foo")];

        let actual = PublicItemsDiff::between(old, new);
        let expected = PublicItemsDiff {
            removed: vec![],
            changed: vec![],
            added: vec![item_with_path("foo")],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn middle_item_added() {
        let old = vec![item_with_path("1"), item_with_path("3")];
        let new = vec![
            item_with_path("1"),
            item_with_path("2"),
            item_with_path("3"),
        ];

        let actual = PublicItemsDiff::between(old, new);
        let expected = PublicItemsDiff {
            removed: vec![],
            changed: vec![],
            added: vec![item_with_path("2")],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn middle_item_removed() {
        let old = vec![
            item_with_path("1"),
            item_with_path("2"),
            item_with_path("3"),
        ];
        let new = vec![item_with_path("1"), item_with_path("3")];

        let actual = PublicItemsDiff::between(old, new);
        let expected = PublicItemsDiff {
            removed: vec![item_with_path("2")],
            changed: vec![],
            added: vec![],
        };
        assert_eq!(actual, expected);
    }

    /// Regression test for <https://github.com/Enselic/cargo-public-api/issues/50>
    #[test]
    fn no_off_by_one_diff_skewing() {
        // old=[
        //     pub enum bat::MappingTarget<'a>,
        //     pub enum bat::PagingMode,
        //     pub enum bat::WrappingMode,
        //     pub enum bat::config::VisibleLines,
        //     pub enum bat::error::Error,
        //     pub enum bat::line_range::RangeCheckResult,
        //     pub enum bat::style::StyleComponent,
        //     pub enum variant bat::MappingTarget::MapExtensionToUnknown,
        //     pub enum variant bat::MappingTarget::MapTo(&'a str),
        //     pub enum variant bat::MappingTarget::MapToUnknown,
        //     pub enum variant bat::PagingMode::Always,
        //     pub enum variant bat::PagingMode::Never,
        //     pub enum variant bat::PagingMode::QuitIfOneScreen,
        //     pub enum variant bat::WrappingMode::Character,
        //     pub enum variant bat::WrappingMode::NoWrapping(bool),
        //     pub enum variant bat::config::VisibleLines::DiffContext(usize),
        //     pub enum variant bat::config::VisibleLines::Ranges(LineRanges),
        //     pub enum variant bat::error::Error::GlobParsingError(::globset::Error),
        //     pub enum variant bat::error::Error::InvalidPagerValueBat,
        //     pub enum variant bat::error::Error::Io(::std::io::Error),
        //     pub enum variant bat::error::Error::Msg(String),
        //     pub enum variant bat::error::Error::ParseIntError(::std::num::ParseIntError),
        //     pub enum variant bat::error::Error::SerdeYamlError(::serde_yaml::Error),
        //     pub enum variant bat::error::Error::SyntectError(::syntect::LoadingError),
        //     pub enum variant bat::error::Error::UndetectedSyntax(String),
        //     pub enum variant bat::error::Error::UnknownStyle(String),
        //     pub enum variant bat::error::Error::UnknownSyntax(String),
        //     pub enum variant bat::line_range::RangeCheckResult::AfterLastRange,
        //     pub enum variant bat::line_range::RangeCheckResult::BeforeOrBetweenRanges,
        //     pub enum variant bat::line_range::RangeCheckResult::InRange,
        //     pub enum variant bat::style::StyleComponent::Auto,
        //     pub enum variant bat::style::StyleComponent::Changes,
        //     pub enum variant bat::style::StyleComponent::Full,
        //     pub enum variant bat::style::StyleComponent::Grid,
        //     pub enum variant bat::style::StyleComponent::Header,
        //     pub enum variant bat::style::StyleComponent::HeaderFilename,
        //     pub enum variant bat::style::StyleComponent::HeaderFilesize,
        //     pub enum variant bat::style::StyleComponent::LineNumbers,
        //     pub enum variant bat::style::StyleComponent::Plain,
        //     pub enum variant bat::style::StyleComponent::Rule,
        //     pub enum variant bat::style::StyleComponent::Snip,
        //     pub fn bat::Input::from(input: input::Input<'a>) -> Self,
        //     pub fn bat::Input::from_bytes(bytes: &'a [u8]) -> Self,
        //     pub fn bat::Input::from_file(path: impl AsRef<Path>) -> Self,
        //     pub fn bat::Input::from_reader<R: Read + 'a>(reader: R) -> Self,
        //     pub fn bat::Input::from_stdin() -> Self,
        //     pub fn bat::Input::kind(self, kind: impl Into<String>) -> Self,
        //     pub fn bat::Input::name(self, name: impl AsRef<Path>) -> Self,
        //     pub fn bat::Input::title(self, title: impl Into<String>) -> Self,
        //     pub fn bat::MappingTarget::clone(&self) -> MappingTarget<'a>,
        //     pub fn bat::MappingTarget::eq(&self, other: &MappingTarget<'a>) -> bool,
        //     pub fn bat::MappingTarget::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::MappingTarget::ne(&self, other: &MappingTarget<'a>) -> bool,
        //     pub fn bat::PagingMode::clone(&self) -> PagingMode,
        //     pub fn bat::PagingMode::default() -> Self,
        //     pub fn bat::PagingMode::eq(&self, other: &PagingMode) -> bool,
        //     pub fn bat::PagingMode::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::PrettyPrinter::colored_output(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::default() -> Self,
        //     pub fn bat::PrettyPrinter::grid(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::header(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::highlight(&mut self, line: usize) -> &mut Self,
        //     pub fn bat::PrettyPrinter::highlight_range(&mut self, from: usize, to: usize) -> &mut Self,
        //     pub fn bat::PrettyPrinter::input(&mut self, input: Input<'a>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::input_file(&mut self, path: impl AsRef<Path>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>,
        //     pub fn bat::PrettyPrinter::input_from_bytes(&mut self, content: &'a [u8]) -> &mut Self,
        //     pub fn bat::PrettyPrinter::input_from_reader<R: Read + 'a>(&mut self, reader: R) -> &mut Self,
        //     pub fn bat::PrettyPrinter::input_stdin(&mut self) -> &mut Self,
        //     pub fn bat::PrettyPrinter::inputs(&mut self, inputs: impl IntoIterator<Item = Input<'a>>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::language(&mut self, language: &'a str) -> &mut Self,
        //     pub fn bat::PrettyPrinter::line_numbers(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::line_ranges(&mut self, ranges: LineRanges) -> &mut Self,
        //     pub fn bat::PrettyPrinter::new() -> Self,
        //     pub fn bat::PrettyPrinter::pager(&mut self, cmd: &'a str) -> &mut Self,
        //     pub fn bat::PrettyPrinter::paging_mode(&mut self, mode: PagingMode) -> &mut Self,
        //     pub fn bat::PrettyPrinter::print(&mut self) -> Result<bool>,
        //     pub fn bat::PrettyPrinter::rule(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::snip(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::syntax_mapping(&mut self, mapping: SyntaxMapping<'a>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::syntaxes(&self) -> impl Iterator<Item = &SyntaxReference>,
        //     pub fn bat::PrettyPrinter::tab_width(&mut self, tab_width: Option<usize>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::term_width(&mut self, width: usize) -> &mut Self,
        //     pub fn bat::PrettyPrinter::theme(&mut self, theme: impl AsRef<str>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::themes(&self) -> impl Iterator<Item = &str>,
        //     pub fn bat::PrettyPrinter::true_color(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::use_italics(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::vcs_modification_markers(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::wrapping_mode(&mut self, mode: WrappingMode) -> &mut Self,
        //     pub fn bat::SyntaxMapping::builtin() -> SyntaxMapping<'a>,
        //     pub fn bat::SyntaxMapping::clone(&self) -> SyntaxMapping<'a>,
        //     pub fn bat::SyntaxMapping::default() -> SyntaxMapping<'a>,
        //     pub fn bat::SyntaxMapping::empty() -> SyntaxMapping<'a>,
        //     pub fn bat::SyntaxMapping::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::SyntaxMapping::insert(&mut self, from: &str, to: MappingTarget<'a>) -> Result<()>,
        //     pub fn bat::SyntaxMapping::insert_ignored_suffix(&mut self, suffix: &'a str),
        //     pub fn bat::SyntaxMapping::mappings(&self) -> &[(GlobMatcher, MappingTarget<'a>)],
        //     pub fn bat::WrappingMode::clone(&self) -> WrappingMode,
        //     pub fn bat::WrappingMode::default() -> Self,
        //     pub fn bat::WrappingMode::eq(&self, other: &WrappingMode) -> bool,
        //     pub fn bat::WrappingMode::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::WrappingMode::ne(&self, other: &WrappingMode) -> bool,
        //     pub fn bat::assets::HighlightingAssets::default_theme() -> &'static str,
        //     pub fn bat::assets::HighlightingAssets::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::assets::HighlightingAssets::from_binary() -> Self,
        //     pub fn bat::assets::HighlightingAssets::from_cache(cache_path: &Path) -> Result<Self>,
        //     pub fn bat::assets::HighlightingAssets::get_syntax_for_path(&self, path: impl AsRef<Path>, mapping: &SyntaxMapping<'_>) -> Result<SyntaxReferenceInSet<'_>>,
        //     pub fn bat::assets::HighlightingAssets::get_syntax_set(&self) -> Result<&SyntaxSet>,
        //     pub fn bat::assets::HighlightingAssets::get_syntaxes(&self) -> Result<&[SyntaxReference]>,
        //     pub fn bat::assets::HighlightingAssets::get_theme(&self, theme: &str) -> &Theme,
        //     pub fn bat::assets::HighlightingAssets::set_fallback_theme(&mut self, theme: &'static str),
        //     pub fn bat::assets::HighlightingAssets::syntax_for_file_name(&self, file_name: impl AsRef<Path>, mapping: &SyntaxMapping<'_>) -> Option<&SyntaxReference>,
        //     pub fn bat::assets::HighlightingAssets::syntaxes(&self) -> &[SyntaxReference],
        //     pub fn bat::assets::HighlightingAssets::themes(&self) -> impl Iterator<Item = &str>,
        //     pub fn bat::assets::SyntaxReferenceInSet::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::assets::build(source_dir: &Path, include_integrated_assets: bool, include_acknowledgements: bool, target_dir: &Path, current_version: &str) -> Result<()>,
        //     pub fn bat::assets::get_acknowledgements() -> String,
        //     pub fn bat::assets_metadata::AssetsMetadata::default() -> AssetsMetadata,
        //     pub fn bat::assets_metadata::AssetsMetadata::deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error> where __D: _serde::Deserializer<'de>,
        //     pub fn bat::assets_metadata::AssetsMetadata::eq(&self, other: &AssetsMetadata) -> bool,
        //     pub fn bat::assets_metadata::AssetsMetadata::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::assets_metadata::AssetsMetadata::is_compatible_with(&self, current_version: &str) -> bool,
        //     pub fn bat::assets_metadata::AssetsMetadata::load_from_folder(path: &Path) -> Result<Option<Self>>,
        //     pub fn bat::assets_metadata::AssetsMetadata::ne(&self, other: &AssetsMetadata) -> bool,
        //     pub fn bat::assets_metadata::AssetsMetadata::serialize<__S>(&self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error> where __S: _serde::Serializer,
        //     pub fn bat::config::Config::clone(&self) -> Config<'a>,
        //     pub fn bat::config::Config::default() -> Config<'a>,
        //     pub fn bat::config::Config::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::config::VisibleLines::clone(&self) -> VisibleLines,
        //     pub fn bat::config::VisibleLines::default() -> Self,
        //     pub fn bat::config::VisibleLines::diff_mode(&self) -> bool,
        //     pub fn bat::config::VisibleLines::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::config::get_pager_executable(config_pager: Option<&str>) -> Option<String>,
        //     pub fn bat::controller::Controller::new<'a>(config: &'a Config<'_>, assets: &'a HighlightingAssets) -> Controller<'a>,
        //     pub fn bat::controller::Controller::run(&self, inputs: Vec<Input<'_>>) -> Result<bool>,
        //     pub fn bat::controller::Controller::run_with_error_handler(&self, inputs: Vec<Input<'_>>, handle_error: impl Fn(&Error, &mut Write)) -> Result<bool>,
        //     pub fn bat::error::Error::fmt(&self, __formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
        //     pub fn bat::error::Error::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::error::Error::from(s: &'static str) -> Self,
        //     pub fn bat::error::Error::from(s: String) -> Self,
        //     pub fn bat::error::Error::from(source: ::globset::Error) -> Self,
        //     pub fn bat::error::Error::from(source: ::serde_yaml::Error) -> Self,
        //     pub fn bat::error::Error::from(source: ::std::io::Error) -> Self,
        //     pub fn bat::error::Error::from(source: ::std::num::ParseIntError) -> Self,
        //     pub fn bat::error::Error::from(source: ::syntect::LoadingError) -> Self,
        //     pub fn bat::error::Error::source(&self) -> std::option::Option<&std::error::Error + 'static>,
        //     pub fn bat::error::default_error_handler(error: &Error, output: &mut Write),
        //     pub fn bat::input::Input::description(&self) -> &InputDescription,
        //     pub fn bat::input::Input::description_mut(&mut self) -> &mut InputDescription,
        //     pub fn bat::input::Input::from(Input<'a>) -> Self,
        //     pub fn bat::input::Input::from_reader(reader: Box<Read + 'a>) -> Self,
        //     pub fn bat::input::Input::is_stdin(&self) -> bool,
        //     pub fn bat::input::Input::ordinary_file(path: impl AsRef<Path>) -> Self,
        //     pub fn bat::input::Input::stdin() -> Self,
        //     pub fn bat::input::Input::with_name(self, provided_name: Option<impl AsRef<Path>>) -> Self,
        //     pub fn bat::input::InputDescription::clone(&self) -> InputDescription,
        //     pub fn bat::input::InputDescription::kind(&self) -> Option<&String>,
        //     pub fn bat::input::InputDescription::new(name: impl Into<String>) -> Self,
        //     pub fn bat::input::InputDescription::set_kind(&mut self, kind: Option<String>),
        //     pub fn bat::input::InputDescription::set_summary(&mut self, summary: Option<String>),
        //     pub fn bat::input::InputDescription::set_title(&mut self, title: Option<String>),
        //     pub fn bat::input::InputDescription::summary(&self) -> String,
        //     pub fn bat::input::InputDescription::title(&self) -> &String,
        //     pub fn bat::line_range::HighlightedLineRanges::clone(&self) -> HighlightedLineRanges,
        //     pub fn bat::line_range::HighlightedLineRanges::default() -> Self,
        //     pub fn bat::line_range::HighlightedLineRanges::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::line_range::LineRange::clone(&self) -> LineRange,
        //     pub fn bat::line_range::LineRange::default() -> LineRange,
        //     pub fn bat::line_range::LineRange::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::line_range::LineRange::from(range_raw: &str) -> Result<LineRange>,
        //     pub fn bat::line_range::LineRange::new(from: usize, to: usize) -> Self,
        //     pub fn bat::line_range::LineRanges::all() -> LineRanges,
        //     pub fn bat::line_range::LineRanges::clone(&self) -> LineRanges,
        //     pub fn bat::line_range::LineRanges::default() -> Self,
        //     pub fn bat::line_range::LineRanges::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::line_range::LineRanges::from(ranges: Vec<LineRange>) -> LineRanges,
        //     pub fn bat::line_range::LineRanges::none() -> LineRanges,
        //     pub fn bat::line_range::RangeCheckResult::clone(&self) -> RangeCheckResult,
        //     pub fn bat::line_range::RangeCheckResult::eq(&self, other: &RangeCheckResult) -> bool,
        //     pub fn bat::line_range::RangeCheckResult::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::style::StyleComponent::clone(&self) -> StyleComponent,
        //     pub fn bat::style::StyleComponent::components(self, interactive_terminal: bool) -> &'static [StyleComponent],
        //     pub fn bat::style::StyleComponent::eq(&self, other: &StyleComponent) -> bool,
        //     pub fn bat::style::StyleComponent::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::style::StyleComponent::from_str(s: &str) -> Result<Self>,
        //     pub fn bat::style::StyleComponent::hash<__H: $crate::hash::Hasher>(&self, state: &mut __H) -> (),
        //     pub fn bat::style::StyleComponents::changes(&self) -> bool,
        //     pub fn bat::style::StyleComponents::clone(&self) -> StyleComponents,
        //     pub fn bat::style::StyleComponents::default() -> StyleComponents,
        //     pub fn bat::style::StyleComponents::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::style::StyleComponents::grid(&self) -> bool,
        //     pub fn bat::style::StyleComponents::header(&self) -> bool,
        //     pub fn bat::style::StyleComponents::header_filename(&self) -> bool,
        //     pub fn bat::style::StyleComponents::header_filesize(&self) -> bool,
        //     pub fn bat::style::StyleComponents::new(components: &[StyleComponent]) -> StyleComponents,
        //     pub fn bat::style::StyleComponents::numbers(&self) -> bool,
        //     pub fn bat::style::StyleComponents::plain(&self) -> bool,
        //     pub fn bat::style::StyleComponents::rule(&self) -> bool,
        //     pub fn bat::style::StyleComponents::snip(&self) -> bool,
        //     pub macro bat::bat_warning!,
        //     pub mod bat,
        //     pub mod bat::assets,
        //     pub mod bat::assets_metadata,
        //     pub mod bat::config,
        //     pub mod bat::controller,
        //     pub mod bat::error,
        //     pub mod bat::input,
        //     pub mod bat::line_range,
        //     pub mod bat::style,
        //     pub struct bat::Input<'a>,
        //     pub struct bat::PrettyPrinter<'a>,
        //     pub struct bat::SyntaxMapping<'a>,
        //     pub struct bat::assets::HighlightingAssets,
        //     pub struct bat::assets::SyntaxReferenceInSet<'a>,
        //     pub struct bat::assets_metadata::AssetsMetadata,
        //     pub struct bat::config::Config<'a>,
        //     pub struct bat::controller::Controller<'a>,
        //     pub struct bat::input::Input<'a>,
        //     pub struct bat::input::InputDescription,
        //     pub struct bat::line_range::HighlightedLineRanges,
        //     pub struct bat::line_range::LineRange,
        //     pub struct bat::line_range::LineRanges,
        //     pub struct bat::style::StyleComponents,
        //     pub struct field bat::assets::SyntaxReferenceInSet::syntax: &'a SyntaxReference,
        //     pub struct field bat::assets::SyntaxReferenceInSet::syntax_set: &'a SyntaxSet,
        //     pub struct field bat::config::Config::colored_output: bool,
        //     pub struct field bat::config::Config::highlighted_lines: HighlightedLineRanges,
        //     pub struct field bat::config::Config::language: Option<&'a str>,
        //     pub struct field bat::config::Config::loop_through: bool,
        //     pub struct field bat::config::Config::pager: Option<&'a str>,
        //     pub struct field bat::config::Config::paging_mode: PagingMode,
        //     pub struct field bat::config::Config::show_nonprintable: bool,
        //     pub struct field bat::config::Config::style_components: StyleComponents,
        //     pub struct field bat::config::Config::syntax_mapping: SyntaxMapping<'a>,
        //     pub struct field bat::config::Config::tab_width: usize,
        //     pub struct field bat::config::Config::term_width: usize,
        //     pub struct field bat::config::Config::theme: String,
        //     pub struct field bat::config::Config::true_color: bool,
        //     pub struct field bat::config::Config::use_custom_assets: bool,
        //     pub struct field bat::config::Config::use_italic_text: bool,
        //     pub struct field bat::config::Config::visible_lines: VisibleLines,
        //     pub struct field bat::config::Config::wrapping_mode: WrappingMode,
        //     pub struct field bat::line_range::HighlightedLineRanges::0: LineRanges,
        //     pub struct field bat::style::StyleComponents::0: HashSet<StyleComponent>,
        //     pub type bat::error::Result<T> = std::result::Result<T, Error>,
        //     pub type bat::style::StyleComponent::Err = Error,
        // ]
        // new=[
        //     pub enum bat::MappingTarget<'a>,
        //     pub enum bat::PagingMode,
        //     pub enum bat::WrappingMode,
        //     pub enum bat::config::VisibleLines,
        //     pub enum bat::error::Error,
        //     pub enum bat::line_range::RangeCheckResult,
        //     pub enum bat::style::StyleComponent,
        //     pub enum variant bat::MappingTarget::MapExtensionToUnknown,
        //     pub enum variant bat::MappingTarget::MapTo(&'a str),
        //     pub enum variant bat::MappingTarget::MapToUnknown,
        //     pub enum variant bat::PagingMode::Always,
        //     pub enum variant bat::PagingMode::Never,
        //     pub enum variant bat::PagingMode::QuitIfOneScreen,
        //     pub enum variant bat::WrappingMode::Character,
        //     pub enum variant bat::WrappingMode::NoWrapping(bool),
        //     pub enum variant bat::config::VisibleLines::DiffContext(usize),
        //     pub enum variant bat::config::VisibleLines::Ranges(LineRanges),
        //     pub enum variant bat::error::Error::GlobParsingError(::globset::Error),
        //     pub enum variant bat::error::Error::InvalidPagerValueBat,
        //     pub enum variant bat::error::Error::Io(::std::io::Error),
        //     pub enum variant bat::error::Error::Msg(String),
        //     pub enum variant bat::error::Error::ParseIntError(::std::num::ParseIntError),
        //     pub enum variant bat::error::Error::SerdeYamlError(::serde_yaml::Error),
        //     pub enum variant bat::error::Error::SyntectError(::syntect::Error),
        //     pub enum variant bat::error::Error::SyntectLoadingError(::syntect::LoadingError),
        //     pub enum variant bat::error::Error::UndetectedSyntax(String),
        //     pub enum variant bat::error::Error::UnknownStyle(String),
        //     pub enum variant bat::error::Error::UnknownSyntax(String),
        //     pub enum variant bat::line_range::RangeCheckResult::AfterLastRange,
        //     pub enum variant bat::line_range::RangeCheckResult::BeforeOrBetweenRanges,
        //     pub enum variant bat::line_range::RangeCheckResult::InRange,
        //     pub enum variant bat::style::StyleComponent::Auto,
        //     pub enum variant bat::style::StyleComponent::Changes,
        //     pub enum variant bat::style::StyleComponent::Default,
        //     pub enum variant bat::style::StyleComponent::Full,
        //     pub enum variant bat::style::StyleComponent::Grid,
        //     pub enum variant bat::style::StyleComponent::Header,
        //     pub enum variant bat::style::StyleComponent::HeaderFilename,
        //     pub enum variant bat::style::StyleComponent::HeaderFilesize,
        //     pub enum variant bat::style::StyleComponent::LineNumbers,
        //     pub enum variant bat::style::StyleComponent::Plain,
        //     pub enum variant bat::style::StyleComponent::Rule,
        //     pub enum variant bat::style::StyleComponent::Snip,
        //     pub fn bat::Input::from(input: input::Input<'a>) -> Self,
        //     pub fn bat::Input::from_bytes(bytes: &'a [u8]) -> Self,
        //     pub fn bat::Input::from_file(path: impl AsRef<Path>) -> Self,
        //     pub fn bat::Input::from_reader<R: Read + 'a>(reader: R) -> Self,
        //     pub fn bat::Input::from_stdin() -> Self,
        //     pub fn bat::Input::kind(self, kind: impl Into<String>) -> Self,
        //     pub fn bat::Input::name(self, name: impl AsRef<Path>) -> Self,
        //     pub fn bat::Input::title(self, title: impl Into<String>) -> Self,
        //     pub fn bat::MappingTarget::clone(&self) -> MappingTarget<'a>,
        //     pub fn bat::MappingTarget::eq(&self, other: &MappingTarget<'a>) -> bool,
        //     pub fn bat::MappingTarget::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::MappingTarget::ne(&self, other: &MappingTarget<'a>) -> bool,
        //     pub fn bat::PagingMode::clone(&self) -> PagingMode,
        //     pub fn bat::PagingMode::default() -> Self,
        //     pub fn bat::PagingMode::eq(&self, other: &PagingMode) -> bool,
        //     pub fn bat::PagingMode::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::PrettyPrinter::colored_output(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::default() -> Self,
        //     pub fn bat::PrettyPrinter::grid(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::header(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::highlight(&mut self, line: usize) -> &mut Self,
        //     pub fn bat::PrettyPrinter::highlight_range(&mut self, from: usize, to: usize) -> &mut Self,
        //     pub fn bat::PrettyPrinter::input(&mut self, input: Input<'a>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::input_file(&mut self, path: impl AsRef<Path>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>,
        //     pub fn bat::PrettyPrinter::input_from_bytes(&mut self, content: &'a [u8]) -> &mut Self,
        //     pub fn bat::PrettyPrinter::input_from_reader<R: Read + 'a>(&mut self, reader: R) -> &mut Self,
        //     pub fn bat::PrettyPrinter::input_stdin(&mut self) -> &mut Self,
        //     pub fn bat::PrettyPrinter::inputs(&mut self, inputs: impl IntoIterator<Item = Input<'a>>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::language(&mut self, language: &'a str) -> &mut Self,
        //     pub fn bat::PrettyPrinter::line_numbers(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::line_ranges(&mut self, ranges: LineRanges) -> &mut Self,
        //     pub fn bat::PrettyPrinter::new() -> Self,
        //     pub fn bat::PrettyPrinter::pager(&mut self, cmd: &'a str) -> &mut Self,
        //     pub fn bat::PrettyPrinter::paging_mode(&mut self, mode: PagingMode) -> &mut Self,
        //     pub fn bat::PrettyPrinter::print(&mut self) -> Result<bool>,
        //     pub fn bat::PrettyPrinter::rule(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::show_nonprintable(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::snip(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::syntax_mapping(&mut self, mapping: SyntaxMapping<'a>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::syntaxes(&self) -> impl Iterator<Item = &SyntaxReference>,
        //     pub fn bat::PrettyPrinter::tab_width(&mut self, tab_width: Option<usize>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::term_width(&mut self, width: usize) -> &mut Self,
        //     pub fn bat::PrettyPrinter::theme(&mut self, theme: impl AsRef<str>) -> &mut Self,
        //     pub fn bat::PrettyPrinter::themes(&self) -> impl Iterator<Item = &str>,
        //     pub fn bat::PrettyPrinter::true_color(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::use_italics(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::vcs_modification_markers(&mut self, yes: bool) -> &mut Self,
        //     pub fn bat::PrettyPrinter::wrapping_mode(&mut self, mode: WrappingMode) -> &mut Self,
        //     pub fn bat::SyntaxMapping::builtin() -> SyntaxMapping<'a>,
        //     pub fn bat::SyntaxMapping::clone(&self) -> SyntaxMapping<'a>,
        //     pub fn bat::SyntaxMapping::default() -> SyntaxMapping<'a>,
        //     pub fn bat::SyntaxMapping::empty() -> SyntaxMapping<'a>,
        //     pub fn bat::SyntaxMapping::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::SyntaxMapping::insert(&mut self, from: &str, to: MappingTarget<'a>) -> Result<()>,
        //     pub fn bat::SyntaxMapping::insert_ignored_suffix(&mut self, suffix: &'a str),
        //     pub fn bat::SyntaxMapping::mappings(&self) -> &[(GlobMatcher, MappingTarget<'a>)],
        //     pub fn bat::WrappingMode::clone(&self) -> WrappingMode,
        //     pub fn bat::WrappingMode::default() -> Self,
        //     pub fn bat::WrappingMode::eq(&self, other: &WrappingMode) -> bool,
        //     pub fn bat::WrappingMode::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::WrappingMode::ne(&self, other: &WrappingMode) -> bool,
        //     pub fn bat::assets::HighlightingAssets::default_theme() -> &'static str,
        //     pub fn bat::assets::HighlightingAssets::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::assets::HighlightingAssets::from_binary() -> Self,
        //     pub fn bat::assets::HighlightingAssets::from_cache(cache_path: &Path) -> Result<Self>,
        //     pub fn bat::assets::HighlightingAssets::get_syntax_for_path(&self, path: impl AsRef<Path>, mapping: &SyntaxMapping<'_>) -> Result<SyntaxReferenceInSet<'_>>,
        //     pub fn bat::assets::HighlightingAssets::get_syntax_set(&self) -> Result<&SyntaxSet>,
        //     pub fn bat::assets::HighlightingAssets::get_syntaxes(&self) -> Result<&[SyntaxReference]>,
        //     pub fn bat::assets::HighlightingAssets::get_theme(&self, theme: &str) -> &Theme,
        //     pub fn bat::assets::HighlightingAssets::set_fallback_theme(&mut self, theme: &'static str),
        //     pub fn bat::assets::HighlightingAssets::syntax_for_file_name(&self, file_name: impl AsRef<Path>, mapping: &SyntaxMapping<'_>) -> Option<&SyntaxReference>,
        //     pub fn bat::assets::HighlightingAssets::syntaxes(&self) -> &[SyntaxReference],
        //     pub fn bat::assets::HighlightingAssets::themes(&self) -> impl Iterator<Item = &str>,
        //     pub fn bat::assets::SyntaxReferenceInSet::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::assets::build(source_dir: &Path, include_integrated_assets: bool, include_acknowledgements: bool, target_dir: &Path, current_version: &str) -> Result<()>,
        //     pub fn bat::assets::get_acknowledgements() -> String,
        //     pub fn bat::assets_metadata::AssetsMetadata::default() -> AssetsMetadata,
        //     pub fn bat::assets_metadata::AssetsMetadata::deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error> where __D: _serde::Deserializer<'de>,
        //     pub fn bat::assets_metadata::AssetsMetadata::eq(&self, other: &AssetsMetadata) -> bool,
        //     pub fn bat::assets_metadata::AssetsMetadata::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::assets_metadata::AssetsMetadata::is_compatible_with(&self, current_version: &str) -> bool,
        //     pub fn bat::assets_metadata::AssetsMetadata::load_from_folder(path: &Path) -> Result<Option<Self>>,
        //     pub fn bat::assets_metadata::AssetsMetadata::ne(&self, other: &AssetsMetadata) -> bool,
        //     pub fn bat::assets_metadata::AssetsMetadata::serialize<__S>(&self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error> where __S: _serde::Serializer,
        //     pub fn bat::config::Config::clone(&self) -> Config<'a>,
        //     pub fn bat::config::Config::default() -> Config<'a>,
        //     pub fn bat::config::Config::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::config::VisibleLines::clone(&self) -> VisibleLines,
        //     pub fn bat::config::VisibleLines::default() -> Self,
        //     pub fn bat::config::VisibleLines::diff_mode(&self) -> bool,
        //     pub fn bat::config::VisibleLines::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::config::get_pager_executable(config_pager: Option<&str>) -> Option<String>,
        //     pub fn bat::controller::Controller::new<'a>(config: &'a Config<'_>, assets: &'a HighlightingAssets) -> Controller<'a>,
        //     pub fn bat::controller::Controller::run(&self, inputs: Vec<Input<'_>>) -> Result<bool>,
        //     pub fn bat::controller::Controller::run_with_error_handler(&self, inputs: Vec<Input<'_>>, handle_error: impl Fn(&Error, &mut Write)) -> Result<bool>,
        //     pub fn bat::error::Error::fmt(&self, __formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
        //     pub fn bat::error::Error::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::error::Error::from(s: &'static str) -> Self,
        //     pub fn bat::error::Error::from(s: String) -> Self,
        //     pub fn bat::error::Error::from(source: ::globset::Error) -> Self,
        //     pub fn bat::error::Error::from(source: ::serde_yaml::Error) -> Self,
        //     pub fn bat::error::Error::from(source: ::std::io::Error) -> Self,
        //     pub fn bat::error::Error::from(source: ::std::num::ParseIntError) -> Self,
        //     pub fn bat::error::Error::from(source: ::syntect::Error) -> Self,
        //     pub fn bat::error::Error::from(source: ::syntect::LoadingError) -> Self,
        //     pub fn bat::error::Error::source(&self) -> std::option::Option<&std::error::Error + 'static>,
        //     pub fn bat::error::default_error_handler(error: &Error, output: &mut Write),
        //     pub fn bat::input::Input::description(&self) -> &InputDescription,
        //     pub fn bat::input::Input::description_mut(&mut self) -> &mut InputDescription,
        //     pub fn bat::input::Input::from(Input<'a>) -> Self,
        //     pub fn bat::input::Input::from_reader(reader: Box<Read + 'a>) -> Self,
        //     pub fn bat::input::Input::is_stdin(&self) -> bool,
        //     pub fn bat::input::Input::ordinary_file(path: impl AsRef<Path>) -> Self,
        //     pub fn bat::input::Input::stdin() -> Self,
        //     pub fn bat::input::Input::with_name(self, provided_name: Option<impl AsRef<Path>>) -> Self,
        //     pub fn bat::input::InputDescription::clone(&self) -> InputDescription,
        //     pub fn bat::input::InputDescription::kind(&self) -> Option<&String>,
        //     pub fn bat::input::InputDescription::new(name: impl Into<String>) -> Self,
        //     pub fn bat::input::InputDescription::set_kind(&mut self, kind: Option<String>),
        //     pub fn bat::input::InputDescription::set_summary(&mut self, summary: Option<String>),
        //     pub fn bat::input::InputDescription::set_title(&mut self, title: Option<String>),
        //     pub fn bat::input::InputDescription::summary(&self) -> String,
        //     pub fn bat::input::InputDescription::title(&self) -> &String,
        //     pub fn bat::line_range::HighlightedLineRanges::clone(&self) -> HighlightedLineRanges,
        //     pub fn bat::line_range::HighlightedLineRanges::default() -> Self,
        //     pub fn bat::line_range::HighlightedLineRanges::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::line_range::LineRange::clone(&self) -> LineRange,
        //     pub fn bat::line_range::LineRange::default() -> LineRange,
        //     pub fn bat::line_range::LineRange::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::line_range::LineRange::from(range_raw: &str) -> Result<LineRange>,
        //     pub fn bat::line_range::LineRange::new(from: usize, to: usize) -> Self,
        //     pub fn bat::line_range::LineRanges::all() -> LineRanges,
        //     pub fn bat::line_range::LineRanges::clone(&self) -> LineRanges,
        //     pub fn bat::line_range::LineRanges::default() -> Self,
        //     pub fn bat::line_range::LineRanges::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::line_range::LineRanges::from(ranges: Vec<LineRange>) -> LineRanges,
        //     pub fn bat::line_range::LineRanges::none() -> LineRanges,
        //     pub fn bat::line_range::RangeCheckResult::clone(&self) -> RangeCheckResult,
        //     pub fn bat::line_range::RangeCheckResult::eq(&self, other: &RangeCheckResult) -> bool,
        //     pub fn bat::line_range::RangeCheckResult::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::style::StyleComponent::clone(&self) -> StyleComponent,
        //     pub fn bat::style::StyleComponent::components(self, interactive_terminal: bool) -> &'static [StyleComponent],
        //     pub fn bat::style::StyleComponent::eq(&self, other: &StyleComponent) -> bool,
        //     pub fn bat::style::StyleComponent::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::style::StyleComponent::from_str(s: &str) -> Result<Self>,
        //     pub fn bat::style::StyleComponent::hash<__H: $crate::hash::Hasher>(&self, state: &mut __H) -> (),
        //     pub fn bat::style::StyleComponents::changes(&self) -> bool,
        //     pub fn bat::style::StyleComponents::clone(&self) -> StyleComponents,
        //     pub fn bat::style::StyleComponents::default() -> StyleComponents,
        //     pub fn bat::style::StyleComponents::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        //     pub fn bat::style::StyleComponents::grid(&self) -> bool,
        //     pub fn bat::style::StyleComponents::header(&self) -> bool,
        //     pub fn bat::style::StyleComponents::header_filename(&self) -> bool,
        //     pub fn bat::style::StyleComponents::header_filesize(&self) -> bool,
        //     pub fn bat::style::StyleComponents::new(components: &[StyleComponent]) -> StyleComponents,
        //     pub fn bat::style::StyleComponents::numbers(&self) -> bool,
        //     pub fn bat::style::StyleComponents::plain(&self) -> bool,
        //     pub fn bat::style::StyleComponents::rule(&self) -> bool,
        //     pub fn bat::style::StyleComponents::snip(&self) -> bool,
        //     pub macro bat::bat_warning!,
        //     pub mod bat,
        //     pub mod bat::assets,
        //     pub mod bat::assets_metadata,
        //     pub mod bat::config,
        //     pub mod bat::controller,
        //     pub mod bat::error,
        //     pub mod bat::input,
        //     pub mod bat::line_range,
        //     pub mod bat::style,
        //     pub struct bat::Input<'a>,
        //     pub struct bat::PrettyPrinter<'a>,
        //     pub struct bat::SyntaxMapping<'a>,
        //     pub struct bat::assets::HighlightingAssets,
        //     pub struct bat::assets::SyntaxReferenceInSet<'a>,
        //     pub struct bat::assets_metadata::AssetsMetadata,
        //     pub struct bat::config::Config<'a>,
        //     pub struct bat::controller::Controller<'a>,
        //     pub struct bat::input::Input<'a>,
        //     pub struct bat::input::InputDescription,
        //     pub struct bat::line_range::HighlightedLineRanges,
        //     pub struct bat::line_range::LineRange,
        //     pub struct bat::line_range::LineRanges,
        //     pub struct bat::style::StyleComponents,
        //     pub struct field bat::assets::SyntaxReferenceInSet::syntax: &'a SyntaxReference,
        //     pub struct field bat::assets::SyntaxReferenceInSet::syntax_set: &'a SyntaxSet,
        //     pub struct field bat::config::Config::colored_output: bool,
        //     pub struct field bat::config::Config::highlighted_lines: HighlightedLineRanges,
        //     pub struct field bat::config::Config::language: Option<&'a str>,
        //     pub struct field bat::config::Config::loop_through: bool,
        //     pub struct field bat::config::Config::pager: Option<&'a str>,
        //     pub struct field bat::config::Config::paging_mode: PagingMode,
        //     pub struct field bat::config::Config::show_nonprintable: bool,
        //     pub struct field bat::config::Config::style_components: StyleComponents,
        //     pub struct field bat::config::Config::syntax_mapping: SyntaxMapping<'a>,
        //     pub struct field bat::config::Config::tab_width: usize,
        //     pub struct field bat::config::Config::term_width: usize,
        //     pub struct field bat::config::Config::theme: String,
        //     pub struct field bat::config::Config::true_color: bool,
        //     pub struct field bat::config::Config::use_custom_assets: bool,
        //     pub struct field bat::config::Config::use_italic_text: bool,
        //     pub struct field bat::config::Config::visible_lines: VisibleLines,
        //     pub struct field bat::config::Config::wrapping_mode: WrappingMode,
        //     pub struct field bat::line_range::HighlightedLineRanges::0: LineRanges,
        //     pub struct field bat::style::StyleComponents::0: HashSet<StyleComponent>,
        //     pub type bat::error::Result<T> = std::result::Result<T, Error>,
        //     pub type bat::style::StyleComponent::Err = Error,
        // ]        
    }

    fn item_with_path(path: &str) -> PublicItem {
        PublicItem {
            path: path
                .split("::")
                .map(std::string::ToString::to_string)
                .collect(),
            tokens: vec![crate::tokens::Token::identifier(path)],
        }
    }

    fn naive_token_parser(tokens_str: &str) -> PublicItem {
        tokens_str.split(" ").map(|parts| Token)
    }
}
