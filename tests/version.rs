#![allow(missing_docs, reason = "okay in tests")]
#![allow(unused_crate_dependencies, reason = "okay in tests")]
#![cfg(not(miri))]
#![cfg(not(target_arch = "wasm32"))]

#[cfg(test)]
#[test]
fn test_readme_deps() {
    if !env!("CARGO_PKG_VERSION").contains("-dev") {
        version_sync::assert_markdown_deps_updated!("README.md");
    }
}

#[cfg(test)]
#[test]
fn test_html_root_url() {
    if !env!("CARGO_PKG_VERSION").contains("-dev") {
        version_sync::assert_html_root_url_updated!("src/lib.rs");
    }
}

#[cfg(test)]
#[test]
fn test_changelog_mentions_version() {
    if !env!("CARGO_PKG_VERSION").contains("-dev") {
        version_sync::assert_contains_regex!("CHANGELOG.md", "^## \\[{version}\\] - ");
        version_sync::assert_contains_regex!("CHANGELOG.md", "\\[{version}\\]: ");
    }
}
