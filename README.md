# sagoin

[![release](https://img.shields.io/github/v/release/figsoda/sagoin?logo=github&style=flat-square)](https://github.com/figsoda/sagoin/releases)
[![version](https://img.shields.io/crates/v/sagoin?logo=rust&style=flat-square)][crate]
[![dependencies](https://img.shields.io/librariesio/release/cargo/sagoin?style=flat-square)](https://libraries.io/cargo/sagoin)
[![license](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue?style=flat-square)](https://www.mozilla.org/en-US/MPL/2.0)
[![ci](https://img.shields.io/github/workflow/status/figsoda/sagoin/ci?label=ci&logo=github-actions&style=flat-square)](https://github.com/figsoda/sagoin/actions?query=workflow:ci)

`sagoin` is a command-line submission tool for the UMD CS Submission Server.
This project is largely inspired by and aims to be the spiritual successor of the [CommandLineSubmission] program from the [marmoset](https://marmoset.cs.umd.edu) project by [Professor William Pugh](https://www.cs.umd.edu/~pugh),
hence the [AGPL-3.0-or-later] license and the name [sagoin](https://en.wiktionary.org/wiki/sagoin),
which refers to a [marmoset](https://en.wikipedia.org/wiki/Marmoset).


## Installation

The latest precompiled binaries are available on [github](https://github.com/figsoda/sagoin/releases/latest).

Alternatively you can install sagoin from [crates.io][crate] with cargo.

```sh
cargo install sagoin
```


## Building from source

```sh
cargo build --release
```


## Usage

```
Usage: sagoin [OPTIONS] [directory]

Arguments:
  [directory]  Set the working directory

Options:
  -n, --no-submit                   Don't submit the project
  -o, --open                        Open the project page in a web browser
  -u, --username <username>         Specify the username for authentication, see --username-type for more information [env: SAGOIN_USERNAME=]
  -U, --username-type <type>        Specify the type for the username [env: SAGOIN_USERNAME_TYPE=] [default: text] [possible values: command, file, text]
  -p, --password <password>         Specify the password for authentication, see --password-type for more information [env: SAGOIN_PASSWORD=]
  -P, --password-type <type>        Specify the type for the password [env: SAGOIN_PASSWORD_TYPE=] [default: text] [possible values: command, file, text]
  -s, --pre-submit-hook <command>   Command to run before submission [env: SAGOIN_PRE_SUBMIT_HOOK=]
  -S, --post-submit-hook <command>  Command to run after successful submissions [env: SAGOIN_POST_SUBMIT_HOOK=]
  -h, --help                        Print help information (use `-h` for a summary, use `--help` for more detail)
  -V, --version                     Print version information
```

## Feature comparison

Freature | Sagoin | [CommandLineSubmission] | [CourseProjectManager Eclipse plugin](https://www.cs.umd.edu/~pugh/eclipse)
-|-|-|-
Submission | ✓ | ✓ | ✓
Work without Eclipse | ✓ | ✓ | ✗
LDAP authentication | ✓ | ✓ | ✓
OpenID authentication | ✗ | ✓ | ✓
CVS integration | ✗ | ✗ | ✓
CVS ignore | ✗ | ✓ | ✓
Git ignore | ✓ | ✗ | ✗
Custom credential input | ✓ | ✗ | ✗
Open project page | ✓ | ✗ | ✗


## Changelog
See [CHANGELOG.md](CHANGELOG.md)


## License

This repository is licensed under [GNU Affero General Public License v3.0 or later][AGPL-3.0-or-later].


[AGPL-3.0-or-later]: https://spdx.org/licenses/AGPL-3.0-or-later.html
[CommandLineSubmission]: https://github.com/billpugh/marmoset/tree/master/CommandLineSubmission
[crate]: https://crates.io/crates/sagoin
