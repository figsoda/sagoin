# sagoin

[![release](https://img.shields.io/github/v/release/figsoda/sagoin?logo=github&style=flat-square)](https://github.com/figsoda/sagoin/releases)
[![version](https://img.shields.io/crates/v/sagoin?logo=rust&style=flat-square)][crate]
[![deps](https://deps.rs/repo/github/figsoda/sagoin/status.svg?style=flat-square&compact=true)](https://deps.rs/repo/github/figsoda/sagoin)
[![license](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue?style=flat-square)](https://www.mozilla.org/en-US/MPL/2.0)
[![ci](https://img.shields.io/github/actions/workflow/status/figsoda/sagoin/ci.yml?label=ci&logo=github-actions&style=flat-square)](https://github.com/figsoda/sagoin/actions?query=workflow:ci)

`sagoin` is a command-line submission tool for the UMD CS Submit Server.
This project is largely inspired by and aims to be the spiritual successor of the [CommandLineSubmission] program from the [marmoset](https://marmoset.cs.umd.edu) project by [Jaime Spacco](https://www.knox.edu/academics/faculty/spacco-jaime), [William Pugh](https://www.cs.umd.edu/~pugh), and others,
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

## Quick start

Submit the project in the current directory:
```sh
sagoin
```

Submit the project under the `ProjectName` directory:
```sh
sagoin ProjectName
```

Submit the project under the `ProjectName` directory and open the project page in a web browser:
```sh
sagoin ProjectName -o
```


## Usage

```
Usage: sagoin [OPTIONS] [DIRECTORY]

Arguments:
  [DIRECTORY]  Set the working directory, all commands will be run under this directory

Options:
  -n, --no-submit                   Don't submit the project
  -l, --list-files                  List files without submitting them
  -i, --info                        Show information about the project and exit
  -o, --open                        Open the project page in a web browser
  -f, --field <KEY> <VALUE>         Additional key-value pairs to send to the submit server, this will not affect authentication
      --color <WHEN>                Controls when to use color [default: auto] [possible values: auto, always, never]
  -c, --config <FILE>               Specify the path to the config file, looks for sagoin/config.toml under XDG configuration directories on unix-like systems, and defaults to {FOLDERID_RoamingAppData}\sagoin\config.toml on windows when unspecified [env: SAGOIN_CONFIG=]
  -t, --time-format <FORMAT>        Specify how to format the due date, ignored without the --info flag, defaults to "[month repr:short] [day padding:none], [hour]:[minute]" when unspecified [env: SAGOIN_TIME_FORMAT=]
  -u, --username <USERNAME>         Specify the username for authentication, see --username-type for more information [env: SAGOIN_USERNAME=]
  -U, --username-type <TYPE>        Specify the type for the username, defaults to text when unspecified [env: SAGOIN_USERNAME_TYPE=] [possible values: command, file, text]
  -p, --password <PASSWORD>         Specify the password for authentication, see --password-type for more information [env: SAGOIN_PASSWORD=]
  -P, --password-type <TYPE>        Specify the type for the password, defaults to text when unspecified [env: SAGOIN_PASSWORD_TYPE=] [possible values: command, file, text]
  -s, --pre-submit-hook <COMMAND>   Command to run before submission [env: SAGOIN_PRE_SUBMIT_HOOK=]
  -S, --post-submit-hook <COMMAND>  Command to run after successful submissions [env: SAGOIN_POST_SUBMIT_HOOK=]
      --client-name <NAME>          Change the client name used to submit the project [env: SAGOIN_CLIENT_NAME=]
      --client-version <VERSION>    Change the client version used to submit the project [env: SAGOIN_CLIENT_VERSION=]
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
Submit hooks | ✓ | ✗ | ✗
Show project information | ✓ | ✗ | ✗


## Configuration

File resolution:
- for unix-like systems: looks for `sagoin/config.toml` under XDG configuration directories, e.g. `/home/<user>/.config/sagoin/config.toml`
- for windows: defaults to `{FOLDERID_RoamingAppData}\sagoin\config.toml`, e.g. `C:\Users\<user>\AppData\Roaming\sagoin\config.toml`

The configuration file is written in [TOML](https://toml.io). Run `sagoin --help` for more information.

```toml
# all possible fields in config.toml
time_format = "..."
username = "..."
username_type = "command | file | text"
password = "..."
password_type = "command | file | text"
pre_submit_hook = "..."
post_submit_hook = "..."
client_name = "..."
client_version = "..."
```


## Changelog
See [CHANGELOG.md](CHANGELOG.md)


## License

This repository is licensed under [GNU Affero General Public License v3.0 or later][AGPL-3.0-or-later].


[AGPL-3.0-or-later]: https://spdx.org/licenses/AGPL-3.0-or-later.html
[CommandLineSubmission]: https://github.com/billpugh/marmoset/tree/master/CommandLineSubmission
[crate]: https://crates.io/crates/sagoin
