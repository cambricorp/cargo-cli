//! `cargo-cli` template files

use curl::easy::Easy;
use error::Result;
use mustache::{self, Data, MapBuilder};
use serde_json;
use std::collections::BTreeMap;
use std::fmt;
use std::io::Cursor;
use std::time::Duration;

/// Template Type
pub enum TemplateType {
    /// main.rs
    Main,
    /// run.rs
    Run,
    /// error.rs
    Error,
    /// LICENSE-MIT
    Mit,
    /// LICENSE-APACHE
    Apache,
    /// README.md
    Readme,
}

/// json
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrateInfo {
    /// Crate data.
    #[serde(rename = "crate")]
    krate: Crate,
}

impl fmt::Display for CrateInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "crate: {}", self.krate)
    }
}

/// Crate data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Crate {
    /// Maximum version field.
    max_version: String,
}

impl fmt::Display for Crate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "max_version: {}", self.max_version)
    }
}

/// Contaier for file templates for various auto-generated files.
pub struct Templates {
    /// clap or docopt?
    clap: bool,
    /// mustache `Data`.
    kvs: Data,
    /// The `main.rs` replacement.
    main: &'static str,
    /// The `run.rs` file.
    run: &'static str,
    /// The `error.rs` file.
    error: &'static str,
    /// The license prefix.
    prefix: &'static str,
    /// The `LICENSE-MIT` file.
    mit: Option<&'static str>,
    /// The `LICENSE-APACHE` file.
    apache: Option<&'static str>,
    /// The README.md file.
    readme: Option<&'static str>,
    /// Should we query for the latest version of the dependencies?
    query: bool,
}


impl Templates {
    /// Create a new template use for file creation.
    pub fn new(
        name: &str,
        clap: bool,
        mit: bool,
        apache: bool,
        readme: bool,
        query: bool,
    ) -> Templates {
        let mut template = Templates {
            clap: clap,
            kvs: MapBuilder::new().insert_str("name", name).build(),
            main: "",
            run: "",
            error: "",
            prefix: "",
            mit: None,
            apache: None,
            readme: None,
            query: query,
        };

        if mit && apache {
            template.prefix = PREFIX_BOTH;
        }
        if mit {
            template.mit = Some(LICENSE_MIT);
            if !apache {
                template.prefix = PREFIX_MIT;
            }
        }
        if apache {
            template.apache = Some(LICENSE_APACHE);
            if !mit {
                template.prefix = PREFIX_APACHE;
            }
        }

        if readme {
            template.readme = Some(README);
        }

        if clap {
            // Setup clap templates
            template.main = CLAP_MAIN_RS;
            template.run = CLAP_RUN_RS;
            template.error = CLAP_ERROR_RS;
        } else {
            // Setup docopt templates
            template.main = DOCOPT_MAIN_RS;
            template.run = DOCOPT_RUN_RS;
            template.error = DOCOPT_ERROR_RS;
        }
        template
    }

    /// Get the `main` value.
    pub fn main(&self) -> Result<String> {
        self.render(self.main)
    }

    /// Get the `run` value.
    pub fn run(&self) -> Result<String> {
        self.render(self.run)
    }

    /// Get the `error` value.
    pub fn error(&self) -> Result<String> {
        self.render(self.error)
    }

    /// Get the `prefix` value.
    pub fn prefix(&self) -> Result<String> {
        self.render(self.prefix)
    }

    /// Get the `mit` value.
    pub fn mit(&self) -> Option<&str> {
        self.mit
    }

    /// Get the `apache` value.
    pub fn apache(&self) -> Option<&str> {
        self.apache
    }

    /// Get the `readme` value.
    pub fn readme(&self) -> Option<Result<String>> {
        if let Some(readme) = self.readme {
            Some(self.render(readme))
        } else {
            None
        }
    }

    /// Does this set of templates include license information?
    pub fn has_license(&self) -> bool {
        self.mit.is_some() || self.apache.is_some()
    }

    /// Get the readme value.
    pub fn cargo_toml_readme(&self) -> &str {
        CARGO_TOML_README
    }

    /// Get the license value for both.
    pub fn cargo_toml_both(&self) -> &str {
        CARGO_TOML_BOTH
    }

    /// Get the license value for MIT only.
    pub fn cargo_toml_mit(&self) -> &str {
        CARGO_TOML_MIT
    }

    /// Get the license value for APACHE-2.0 only.
    pub fn cargo_toml_apache(&self) -> &str {
        CARGO_TOML_APACHE
    }

    /// Add the appropriate deps to the deps `BTreeMap`.
    pub fn add_deps(&self, deps: &mut BTreeMap<String, String>) {
        if self.clap {
            let (error_chain_latest, clap_latest) = if self.query {
                (
                    get_latest("error-chain").unwrap_or_else(|_| "0.10.0".to_string()),
                    get_latest("clap").unwrap_or_else(|_| "2.25.0".to_string()),
                )
            } else {
                ("0.10.0".to_string(), "2.25.0".to_string())
            };
            deps.insert("error-chain".to_string(), error_chain_latest);
            deps.insert("clap".to_string(), clap_latest);
        } else {
            let (ec_latest, docopt_latest, sd_latest, s_latest) = if self.query {
                (
                    get_latest("error-chain").unwrap_or_else(|_| "0.10.0".to_string()),
                    get_latest("docopt").unwrap_or_else(|_| "0.8.1".to_string()),
                    get_latest("serde_derive").unwrap_or_else(|_| "1.0.9".to_string()),
                    get_latest("serde").unwrap_or_else(|_| "1.0.9".to_string()),
                )
            } else {
                (
                    "0.10.0".to_string(),
                    "0.8.1".to_string(),
                    "1.0.9".to_string(),
                    "1.0.9".to_string(),
                )
            };
            deps.insert("serde_derive".to_string(), sd_latest);
            deps.insert("serde".to_string(), s_latest);
            deps.insert("error-chain".to_string(), ec_latest);
            deps.insert("docopt".to_string(), docopt_latest);
        }
    }

    /// Render the given mustache template with the key/value pairs in `kvs`.
    fn render(&self, template_str: &str) -> Result<String> {
        let template = mustache::compile_str(template_str)?;
        let mut out = Cursor::new(Vec::new());
        template.render_data(&mut out, &self.kvs)?;
        Ok(String::from_utf8(out.into_inner())?)
    }
}

/// Get the latest version from crates.io.
fn get_latest(name: &str) -> Result<String> {
    let crate_json = fetch_cratesio(name)?;
    let crate_info: CrateInfo = serde_json::from_str(&crate_json)?;
    Ok(crate_info.krate.max_version)
}

/// Fetch crate data from crates.io.
fn fetch_cratesio(path: &str) -> Result<String> {
    let mut easy = Easy::new();
    easy.url(&format!("{}/api/v1/crates/{}", REGISTRY_HOST, path))?;
    easy.timeout(Duration::from_secs(5))?;
    easy.get(true)?;
    easy.accept_encoding("application/json")?;

    let mut html = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            html.extend_from_slice(data);
            Ok(data.len())
        })?;


        transfer.perform()?;
    }

    Ok(String::from_utf8(html)?)
}

/// crates.io Cargo Registry
const REGISTRY_HOST: &str = "https://crates.io";

/// Cargo.toml package readme entry.
const CARGO_TOML_README: &str = r#"README.md"#;

/// clap version of `main.rs`
const CLAP_MAIN_RS: &str = r#"//! `{{ name }}` 0.1.0
#![deny(missing_docs)]
#[macro_use]
extern crate error_chain;
extern crate clap;

mod error;
mod run;

use std::io::{self, Write};
use std::process;

/// CLI Entry Point
fn main() {
    match run::run() {
        Ok(i) => process::exit(i),
        Err(e) => {
            writeln!(io::stderr(), "{}", e).expect("Unable to write to stderr!");
            process::exit(1)
        }
    }
}"#;

/// clap version of `run.rs`
const CLAP_RUN_RS: &str = r#"//! `{{ name }}` runtime
use clap::App;
use error::Result;
use std::io::{self, Write};

/// CLI Runtime
pub fn run() -> Result<i32> {
    let _matches = App::new(env!("CARGO_PKG_NAME"))
                      .version(env!("CARGO_PKG_VERSION"))
                      .author(env!("CARGO_PKG_AUTHORS"))
                      .about("Prints 'Hello, Rustaceans!' to stdout")
                      .get_matches();
    writeln!(io::stdout(), "Hello, Rustaceans!")?;
    Ok(0)
}"#;

/// clap version of `error.rs`
const CLAP_ERROR_RS: &str = r#"//! `{{ name }}` errors
error_chain!{
    foreign_links {
        Io(::std::io::Error);
    }
}"#;

/// docopt version of `main.rs`
const DOCOPT_MAIN_RS: &str = r#"//! `{{ name }}` 0.1.0
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate docopt;

mod error;
mod run;

use std::io::{self, Write};
use std::process;

/// CLI Entry Point
fn main() {
    match run::run() {
        Ok(i) => process::exit(i),
        Err(e) => {
            writeln!(io::stderr(), "{}", e).expect("Unable to write to stderr!");
            process::exit(1)
        }
    }
}"#;

/// docopt version of `run.rs`
const DOCOPT_RUN_RS: &str = r#"//! `{{ name }}` runtime
use docopt::Docopt;
use error::Result;
use std::io::{self, Write};

/// Write the Docopt usage string.
const USAGE: &str = "
Usage: {{ name }} ( -h | --help )
       {{ name }} ( -V | --version )

Options:
    -h --help     Show this screen.
    -v --version  Show version.
";

/// Command line arguments
#[derive(Debug, Deserialize)]
struct Args;

/// CLI Runtime
pub fn run() -> Result<i32> {
    let _args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize())?;
    writeln!(io::stdout(), "Hello, Rustaceans!")?;
    Ok(0)
}"#;

/// docopt version of `error.rs`
const DOCOPT_ERROR_RS: &str = r#"//! `{{ name }}` errors
error_chain!{
    foreign_links {
        Docopt(::docopt::Error);
        Io(::std::io::Error);
    }
}"#;

/// MIT/Apache-2.0 license entry for Cargo.toml.
const CARGO_TOML_BOTH: &str = r#"MIT/Apache-2.0"#;

/// .rs file prefix when both licenses are used.
const PREFIX_BOTH: &str = r#"// Copyright (c) 2017 {{ name }} developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

"#;

/// MIT license entry for Cargo.toml.
const CARGO_TOML_MIT: &str = r#"MIT"#;

/// .rs file prefix when MIT is the only license.
const PREFIX_MIT: &str = r#"// Copyright (c) 2017 {{ name }} developers
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

"#;

/// Apache-2.0 license entry for Cargo.toml.
const CARGO_TOML_APACHE: &str = r#"Apache-2.0"#;

/// .rs file prefix when Apache-2.0 is the only license.
const PREFIX_APACHE: &str = r#"// Copyright (c) 2017 {{ name }} developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>.
// All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

"#;

/// MIT License template
const LICENSE_MIT: &str = r#"Copyright (c) 2016 The Rust Project Developers

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.

"#;

/// Apache-2.0 License template
const LICENSE_APACHE: &str = r#"                              Apache License
                        Version 2.0, January 2004
                     http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

   "Licensor" shall mean the copyright owner or entity authorized by
   the copyright owner that is granting the License.

   "Legal Entity" shall mean the union of the acting entity and all
   other entities that control, are controlled by, or are under common
   control with that entity. For the purposes of this definition,
   "control" means (i) the power, direct or indirect, to cause the
   direction or management of such entity, whether by contract or
   otherwise, or (ii) ownership of fifty percent (50%) or more of the
   outstanding shares, or (iii) beneficial ownership of such entity.

   "You" (or "Your") shall mean an individual or Legal Entity
   exercising permissions granted by this License.

   "Source" form shall mean the preferred form for making modifications,
   including but not limited to software source code, documentation
   source, and configuration files.

   "Object" form shall mean any form resulting from mechanical
   transformation or translation of a Source form, including but
   not limited to compiled object code, generated documentation,
   and conversions to other media types.

   "Work" shall mean the work of authorship, whether in Source or
   Object form, made available under the License, as indicated by a
   copyright notice that is included in or attached to the work
   (an example is provided in the Appendix below).

   "Derivative Works" shall mean any work, whether in Source or Object
   form, that is based on (or derived from) the Work and for which the
   editorial revisions, annotations, elaborations, or other modifications
   represent, as a whole, an original work of authorship. For the purposes
   of this License, Derivative Works shall not include works that remain
   separable from, or merely link (or bind by name) to the interfaces of,
   the Work and Derivative Works thereof.

   "Contribution" shall mean any work of authorship, including
   the original version of the Work and any modifications or additions
   to that Work or Derivative Works thereof, that is intentionally
   submitted to Licensor for inclusion in the Work by the copyright owner
   or by an individual or Legal Entity authorized to submit on behalf of
   the copyright owner. For the purposes of this definition, "submitted"
   means any form of electronic, verbal, or written communication sent
   to the Licensor or its representatives, including but not limited to
   communication on electronic mailing lists, source code control systems,
   and issue tracking systems that are managed by, or on behalf of, the
   Licensor for the purpose of discussing and improving the Work, but
   excluding communication that is conspicuously marked or otherwise
   designated in writing by the copyright owner as "Not a Contribution."

   "Contributor" shall mean Licensor and any individual or Legal Entity
   on behalf of whom a Contribution has been received by Licensor and
   subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone or by combination of their Contribution(s)
   with the Work to which such Contribution(s) was submitted. If You
   institute patent litigation against any entity (including a
   cross-claim or counterclaim in a lawsuit) alleging that the Work
   or a Contribution incorporated within the Work constitutes direct
   or contributory patent infringement, then any patent licenses
   granted to You under this License for that Work shall terminate
   as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:

   (a) You must give any other recipients of the Work or
       Derivative Works a copy of this License; and

   (b) You must cause any modified files to carry prominent notices
       stating that You changed the files; and

   (c) You must retain, in the Source form of any Derivative Works
       that You distribute, all copyright, patent, trademark, and
       attribution notices from the Source form of the Work,
       excluding those notices that do not pertain to any part of
       the Derivative Works; and

   (d) If the Work includes a "NOTICE" text file as part of its
       distribution, then any Derivative Works that You distribute must
       include a readable copy of the attribution notices contained
       within such NOTICE file, excluding those notices that do not
       pertain to any part of the Derivative Works, in at least one
       of the following places: within a NOTICE text file distributed
       as part of the Derivative Works; within the Source form or
       documentation, if provided along with the Derivative Works; or,
       within a display generated by the Derivative Works, if and
       wherever such third-party notices normally appear. The contents
       of the NOTICE file are for informational purposes only and
       do not modify the License. You may add Your own attribution
       notices within Derivative Works that You distribute, alongside
       or as an addendum to the NOTICE text from the Work, provided
       that such additional attribution notices cannot be construed
       as modifying the License.

   You may add Your own copyright statement to Your modifications and
   may provide additional or different license terms and conditions
   for use, reproduction, or distribution of Your modifications, or
   for any such Derivative Works as a whole, provided Your use,
   reproduction, and distribution of the Work otherwise complies with
   the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.
   Notwithstanding the above, nothing herein shall supersede or modify
   the terms of any separate license agreement you may have executed
   with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or
   agreed to in writing, Licensor provides the Work (and each
   Contributor provides its Contributions) on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied, including, without limitation, any warranties or conditions
   of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
   PARTICULAR PURPOSE. You are solely responsible for determining the
   appropriateness of using or redistributing the Work and assume any
   risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   unless required by applicable law (such as deliberate and grossly
   negligent acts) or agreed to in writing, shall any Contributor be
   liable to You for damages, including any direct, indirect, special,
   incidental, or consequential damages of any character arising as a
   result of this License or out of the use or inability to use the
   Work (including but not limited to damages for loss of goodwill,
   work stoppage, computer failure or malfunction, or any and all
   other commercial damages or losses), even if such Contributor
   has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations and/or rights consistent with this
   License. However, in accepting such obligations, You may act only
   on Your own behalf and on Your sole responsibility, not on behalf
   of any other Contributor, and only if You agree to indemnify,
   defend, and hold each Contributor harmless for any liability
   incurred by, or claims asserted against, such Contributor by reason
   of your accepting any such warranty or additional liability.

END OF TERMS AND CONDITIONS

APPENDIX: How to apply the Apache License to your work.

   To apply the Apache License to your work, attach the following
   boilerplate notice, with the fields enclosed by brackets "[]"
   replaced with your own identifying information. (Don't include
   the brackets!)  The text should be enclosed in the appropriate
   comment syntax for the file format. We also recommend that a
   file or class name and description of purpose be included on the
   same "printed page" as the copyright notice for easier
   identification within third-party archives.

Copyright [yyyy] [name of copyright owner]

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

	http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
"#;

/// README.md template
const README: &str = r#"# {{ name }}
A Rust command line interface generated by `cargo-cli`.
"#;
