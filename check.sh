#!/usr/bin/env bash

set -Eeuo pipefail

crate_docs_link="https://docs\.rs/bitflags-lite/latest/bitflags_lite"

tmp_dir=""
bold_red=$'\033[1;31m'
bold_green=$'\033[1;32m'
no_color=$'\033[0m'

cleanup() {
    local return_code=$?
    if [ -n "$tmp_dir" ]; then
        rm -rf -- "$tmp_dir"
    fi
    exit $return_code
}

trap cleanup EXIT

tmp_dir="$( mktemp -d )"

exec() { "$@"; }
passthru() { tee; }
comment() { tee; }
ok() { echo "${bold_green}OK${no_color}: $@" 1>&2; }
fail() { echo "${bold_red}ERROR${no_color}: $@" 1>&2; exit 1; }
echo_and_run() { echo "$ ${*@Q}"; "$@"; }

echo_and_try_run() {
    set +eo pipefail
    echo "$ ${*@Q}"
    "$@" 2> >( tee "$tmp_dir/error.txt" )
    echo $? > "$tmp_dir/return_code.txt"
    set -eo pipefail
}

expect_failure() {
    if [ "$(cat "$tmp_dir/return_code.txt")" -ne "0" ]; then
        ok "Command failed as expected."
        if ! cat "$tmp_dir/error.txt" | grep -q "$@"; then
            fail "Unexpected error message, expected regex: ${*@Q}."
        fi
    else
        fail "Command did not fail as expected."
    fi
}

# Apply some transformations to lib.rs and README.md, to get the documentation
# in a single intermediate format, which is expected to be the same for both:
# lib.rs and README.md.
normalize_docs() {
    source=$1
    local libdoc=false
    local readme=false
    if [[ $source == "lib.rs" ]]; then
        libdoc=exec
        readme=passthru
    elif [[ $source == "README.md" ]]; then
        libdoc=passthru
        readme=exec
    else
        fail "Internal script error: invalid docs source."
    fi

    set +eo pipefail

    cat \
        | comment "Remove title and badges from README.md" \
        | $readme awk '
            BEGIN { is_content=0 }
            is_content==1 { print; next }
            /^(# .*||!\[.*|\[!\[.*)$/ { next } # Ignore title, empty line and images.
            is_content==0 { is_content=1; print; next }
        ' \
        | comment "Remove '//!' lines from lib.rs" \
        | $libdoc rg '^//!' \
        | $libdoc rg '^//! ?' -r '' \
        | comment "Add "C " prefix for markdown code lines." \
        | comment "Add "T " prefix for other markdown lines." \
        | awk '
            BEGIN { is_code=0 }
            /^```.*$/ { print "C " $0; is_code=!is_code; next }
            is_code==0 { print "T " $0; next }
            is_code==1 { print "C " $0; next }
        ' \
        | comment "Remove hidden portions from lib.rs doc examples." \
        | $libdoc rg -v $'^C #( |$)' \
        | comment "Decrease all headers level in README.md." \
        | $readme rg --passthru $'^T #(#+) (.*)' -r $'T $1 $2' \
        | comment "Convert rust stdlib links to rust paths in README.md." \
        | comment "Use "L " prefix for markdown lines with link references." \
        | $readme rg --passthru \
            $'^T \[(.*)\]: https://doc\.rust-lang\.org/(std|core)/(?:(?:(?:([^/]*)/)?([^/]*)/)?([^/]*)/)?([^/]*\.html)(#| |$)' \
            -r $'L [$1]: $2::$3::$4::$5::$6$7' \
        | comment "Convert rust stdlib module links to rust paths in README.md." \
        | $readme rg --passthru \
            $'^T \[(.*)\]: https://doc\.rust-lang\.org/(std|core)/(?:(?:(?:([^/]*)/)?([^/]*)/)?([^/]*)/)?index\.html(#| |$)' \
            -r $'L [$1]: $2::$3::$4::$5$6' \
        | comment "Convert crate links to rust paths in README.md." \
        | $readme rg --passthru \
            $'^T \[(.*)\]: '"$crate_docs_link"$'/(?:(?:(?:([^/]*)/)?([^/]*)/)?([^/]*)/)?([^/]*\.html)(#| |$)' \
            -r $'L [$1]: $2::$3::$4::$5$6' \
        | comment "Convert crate module links to rust paths in README.md." \
        | $readme rg --passthru \
            $'^T \[(.*)\]: '"$crate_docs_link"$'/(?:(?:(?:([^/]*)/)?([^/]*)/)?([^/]*)/)?index\.html(#| |$)' \
            -r $'L [$1]: $2::$3::$4$5' \
        | comment "Remove artifact "::::" and starting "::" in paths after the previous link conversions." \
        | $readme rg --passthru \
            $'^L \[(.*)\]: ([^:]*)::(?:::)+' -r $'L [$1]: $2::' \
        | $readme rg --passthru \
            $'^L \[(.*)\]: ::' -r $'L [$1]: ' \
        | comment "Convert link methods to rust paths in README.md." \
        | $readme rg --passthru \
            $'^L \[(.*)\]: (.*)#method\.' -r $'L [$1]: $2$3::' \
        | comment "Remove link titles that now duplicates rust path in README.md." \
        | $readme rg --pcre2 --passthru \
            $'^L \[(.*)\]: ([^ ]*) "\\2"' -r $'L [$1]: $2' \
        | comment "Remove link references that now duplicates rust path in README.md" \
        | $readme rg --pcre2 -v $'^L \[`(.*)`\]: \\1$' \
        | $readme rg -U --pcre2 --passthru $'^(L .*)\nT \n(?:T \n)+(L .*)$' -r $'$1\n\n$2' \
        | comment "Remove temporary prefixes." \
        | rg --passthru $'T ' -r '' \
        | rg --passthru $'C ' -r '' \
        | rg --passthru $'L ' -r '' \
        | tee

    set -eo pipefail
}

echo_and_run cargo +nightly fmt --all -- --check
echo_and_run cargo outdated --exit-code 1

readme="$( cat README.md | normalize_docs "README.md" )"
libdoc="$( cat src/lib.rs | normalize_docs "lib.rs" )"
echo_and_run git diff <(echo "$readme") <(echo "$libdoc")

params_sets=(
    'stable,stable,"",""'
    'beta,beta,"",""'
    'nightly,nightly,"",""'
    '1.97.0,1.97.0,"",""'
    'stable release,stable,"","--release"'
    'nightly release,nightly,"","--release"'
    'no_std x86_64,stable,x86_64-unknown-none,""'
    'no_std thumbv7em,stable,thumbv7em-none-eabihf,""'
);


for params in "${params_sets[@]}"; do
    IFS=: read -r name toolchain target flags <<< "$params"
    if [[ $target == "" ]]; then
        echo_and_run cargo hack test --all-targets --feature-powerset $flags
        echo_and_run cargo test --all-targets --all-features $flags
    else
        echo_and_run cargo hack check --lib \
            --target $target \
            --feature-powerset \
            --exclude-features std
    fi
done


echo_and_run cargo deny --workspace --all-features check

echo_and_run cargo +nightly llvm-cov --doctests --all-features --html \
    --fail-under-functions 99 \
    --fail-under-lines 95 \
    --fail-under-regions 95

ok "All checks succeeded." 1>&2
