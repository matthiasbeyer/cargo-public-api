use std::fmt::Display;

use pretty_assertions::assert_eq;
use public_api::{public_api_from_rustdoc_json_str, Error, Options};

mod utils;
use serial_test::serial;
use utils::rustdoc_json_str_for_crate;

struct ExpectedDiff<'a> {
    removed: &'a [&'a str],
    changed: &'a [(&'a str, &'a str)],
    added: &'a [&'a str],
}

#[test]
#[serial] // Writing and reading rustdoc JSON to/from file-system; must run one test at a time
fn with_blanket_implementations() {
    assert_public_api_with_blanket_implementations(
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.2.0"),
        include_str!("./expected-output/example_api-v0.2.0-with-blanket-implementations.txt"),
    );
}

#[test]
#[serial]
fn diff_with_added_items() {
    assert_public_api_diff(
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.1.0"),
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.2.0"),
        &ExpectedDiff {
            removed: &[],
            changed: &[(
                "pub fn example_api::function(v1_param: Struct)",
                "pub fn example_api::function(v1_param: Struct, v2_param: usize)",
            )],
            added: &[
                "pub struct example_api::StructV2",
                "pub struct field example_api::Struct::v2_field: usize",
                "pub struct field example_api::StructV2::field: usize",
            ],
        },
    );
}

#[test]
#[serial]
fn no_diff() {
    // No change to the public API
    assert_public_api_diff(
        &rustdoc_json_str_for_crate("../test-apis/comprehensive_api"),
        &rustdoc_json_str_for_crate("../test-apis/comprehensive_api"),
        &ExpectedDiff {
            removed: &[],
            changed: &[],
            added: &[],
        },
    );
}

#[test]
#[serial]
fn diff_with_removed_items() {
    assert_public_api_diff(
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.2.0"),
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.1.0"),
        &ExpectedDiff {
            removed: &[
                "pub struct example_api::StructV2",
                "pub struct field example_api::Struct::v2_field: usize",
                "pub struct field example_api::StructV2::field: usize",
            ],
            changed: &[(
                "pub fn example_api::function(v1_param: Struct, v2_param: usize)",
                "pub fn example_api::function(v1_param: Struct)",
            )],
            added: &[],
        },
    );
}

#[test]
#[serial]
fn comprehensive_api() {
    assert_public_api(
        &rustdoc_json_str_for_crate("../test-apis/comprehensive_api"),
        include_str!("./expected-output/comprehensive_api.txt"),
    );
}

#[test]
#[serial]
fn comprehensive_api_proc_macro() {
    assert_public_api(
        &rustdoc_json_str_for_crate("../test-apis/comprehensive_api_proc_macro"),
        include_str!("./expected-output/comprehensive_api_proc_macro.txt"),
    );
}

/// I confess: this test is mainly to get function code coverage on Ord
#[test]
#[serial]
fn public_item_ord() {
    let public_api = public_api_from_rustdoc_json_str(
        &rustdoc_json_str_for_crate("../test-apis/comprehensive_api"),
        Options::default(),
    )
    .unwrap();

    let generic_arg = public_api
        .clone()
        .into_iter()
        .find(|x| format!("{}", x).contains("generic_arg"))
        .unwrap();

    let generic_bound = public_api
        .into_iter()
        .find(|x| format!("{}", x).contains("generic_bound"))
        .unwrap();

    assert_eq!(generic_arg.max(generic_bound.clone()), generic_bound);
}

#[test]
#[serial]
fn invalid_json() {
    let result = public_api_from_rustdoc_json_str("}}}}}}}}}", Options::default());
    ensure_impl_debug(&result);
    assert!(matches!(result, Err(Error::SerdeJsonError(_))));
}

#[test]
fn options() {
    let options = Options::default();
    ensure_impl_debug(&options);

    // If we don't do this, we will not have code coverage 100% of functions in
    // lib.rs, which is more annoying than doing this clone
    #[allow(clippy::clone_on_copy)]
    let _ = options.clone();
}

#[test]
#[serial]
fn pretty_printed_diff() {
    let options = Options::default();
    let old = public_api_from_rustdoc_json_str(
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.1.0"),
        options,
    )
    .unwrap();
    let new = public_api_from_rustdoc_json_str(
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.2.0"),
        options,
    )
    .unwrap();

    let diff = public_api::diff::PublicItemsDiff::between(old, new);
    let pretty_printed = format!("{:#?}", diff);
    assert_eq!(
        pretty_printed,
        "PublicItemsDiff {
    removed: [],
    changed: [
        ChangedPublicItem {
            old: pub fn example_api::function(v1_param: Struct),
            new: pub fn example_api::function(v1_param: Struct, v2_param: usize),
        },
    ],
    added: [
        pub struct example_api::StructV2,
        pub struct field example_api::Struct::v2_field: usize,
        pub struct field example_api::StructV2::field: usize,
    ],
}"
    );
}

fn assert_public_api_diff(old_json: &str, new_json: &str, expected: &ExpectedDiff) {
    let old = public_api_from_rustdoc_json_str(old_json, Options::default()).unwrap();
    let new = public_api_from_rustdoc_json_str(new_json, Options::default()).unwrap();

    let diff = public_api::diff::PublicItemsDiff::between(old, new);

    assert_eq!(expected.added, into_strings(diff.added));
    assert_eq!(expected.removed, into_strings(diff.removed));

    let expected_changed: Vec<_> = expected
        .changed
        .iter()
        .map(|x| (x.0.to_owned(), x.1.to_owned()))
        .collect();
    let actual_changed: Vec<_> = diff
        .changed
        .iter()
        .map(|x| (format!("{}", &x.old), format!("{}", &x.new)))
        .collect();
    assert_eq!(expected_changed, actual_changed);
}

fn assert_public_api(json: &str, expected: &str) {
    assert_public_api_impl(json, expected, false);
}

fn assert_public_api_with_blanket_implementations(json: &str, expected: &str) {
    assert_public_api_impl(json, expected, true);
}

fn assert_public_api_impl(
    rustdoc_json_str: &str,
    expected_output: &str,
    with_blanket_implementations: bool,
) {
    let mut options = Options::default();
    options.with_blanket_implementations = with_blanket_implementations;
    options.sorted = true;

    let actual = into_strings(public_api_from_rustdoc_json_str(rustdoc_json_str, options).unwrap());

    let expected = expected_output_to_string_vec(expected_output);

    assert_eq!(expected, actual);
}

fn expected_output_to_string_vec(expected_output: &str) -> Vec<String> {
    expected_output
        .split('\n')
        .map(String::from)
        .filter(|s| !s.is_empty()) // Remove empty entry caused by trailing newline in files
        .collect()
}

fn into_strings(items: Vec<impl Display>) -> Vec<String> {
    items.into_iter().map(|x| format!("{}", x)).collect()
}

/// To be honest this is mostly to get higher code coverage numbers.
/// But it is actually useful thing to test.
fn ensure_impl_debug(impl_debug: &impl std::fmt::Debug) {
    eprintln!("Yes, this can be debugged: {:?}", impl_debug);
}
