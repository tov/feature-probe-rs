#![doc(html_root_url = "https://docs.rs/feature-probe/0.1.1")]
//! To support multiple versions of Rust, it's often necessary to conditionally
//! compile parts of our libraries or programs. It's possible to allow users to
//! specify what features to enable, but detection is better, because users get
//! all the features that their version of Rust supports. And while we could check
//! the rustc version, it's better to probe for individual features.
//!
//! ## Usage
//!
//! It’s [on crates.io](https://crates.io/crates/feature-probe), so you can add
//!
//! ```toml
//! [build-dependencies]
//! feature-probe = "0.1.1"
//! ```
//!
//! Then add to your `build.rs`:
//!
//! ```no_compile
//! extern crate feature_probe;
//!
//! use feature_probe::Probe;
//! ```
//!
//! Then you can probe for features such as types or expressions. For example:
//!
//! ```no_compile
//! fn main () {
//!     let probe = Probe::new();
//!
//!     if probe.probe_type("i128") {
//!         println!("cargo:rustc-cfg=int_128");
//!     }
//!
//!     if probe.probe_type("::std::ops::RangeInclusive<u64>") {
//!         println!("cargo:rustc-cfg=inclusive_range");
//!     }
//! }
//! ```
//!
//! This crate supports Rust version 1.16.0 and newer.

#[macro_use]
extern crate lazy_static;

use std::env;
use std::path::PathBuf;
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::Mutex;

/// A probe object, which is used for probing for features.
///
/// Create this with [`Probe::new`](#method.new), and then probe with
/// one of the probing methods.
#[derive(Clone, Debug)]
pub struct Probe {
    debug: bool,
    emit_type: &'static str,
    retries: usize,
    rustc: PathBuf,
    rustc_args: Vec<OsString>,
}


lazy_static! {
    static ref RUSTC_MUTEX: Mutex<()> = Mutex::new(());
}

#[cfg(target_os = "windows")]
const NULL_DEVICE: &'static str = "NUL";

#[cfg(not(target_os = "windows"))]
const NULL_DEVICE: &'static str = "/dev/null";


impl Probe {
    /// Creates a new [`Probe`](struct.Probe.html) object with a default
    /// configuration.
    ///
    /// In particular, it consults the environment variable `"RUSTC"` to determine
    /// what Rust compiler to use. If this are not set it defaults to `"rustc"`.
    ///
    /// # Panics
    ///
    /// If the child `rustc` cannot be started or communicated with.
    ///
    /// # Examples
    ///
    /// ```
    /// use feature_probe::Probe;
    ///
    /// let probe = Probe::new();
    /// assert!( probe.probe_type("u32") );
    /// ```
    pub fn new() -> Self {
        Probe {
            debug: false,
            emit_type: "obj",
            retries: 2,
            rustc: PathBuf::from(env_var_or("RUSTC", "rustc")),
            rustc_args: vec![],
        }
    }

    /// Adds an argument to the list of arguments to pass to
    /// `rustc`.
    pub fn arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: Into<OsString> {

        self.rustc_args.push(arg.into());
        self
    }

    /// Adds multiple arguments to the list of arguments to pass
    /// to `rustc`.
    pub fn args<S, I>(&mut self, args: I) -> &mut Self
    where
        S: Into<OsString>,
        I: IntoIterator<Item=S> {

        self.rustc_args.extend(args.into_iter().map(S::into));
        self
    }

    /// Configures the probe to show the programs that it
    /// attempts to compile.
    ///
    /// Default is `false`.
    pub fn debug(&mut self, debug: bool) -> &mut Self {
        self.debug = debug;
        self
    }

    /// Configures the probe to ask `rustc` to emit a different
    /// output type.
    ///
    /// Default is `obj`.
    pub fn emit(&mut self, emit_type: &'static str) -> &mut Self {
        self.emit_type = emit_type;
        self
    }

    /// Configures the probe to retry this many times if starting
    /// or communicating with `rustc` fails.
    ///
    /// Default is `2`.
    pub fn retries(&mut self, retries: usize) -> &mut Self {
        self.retries = retries;
        self
    }

    /// Sets the name or path to use for running `rustc`.
    ///
    /// Default is value of environment `RUSTC` if set, `"rustc"`
    /// otherwise.
    pub fn rustc<P: Into<PathBuf>>(&mut self, rustc: P) -> &mut Self {
        self.rustc = rustc.into();
        self
    }

    /// Probes for the existence of the given type by name.
    ///
    /// # Panics
    ///
    /// If the child `rustc` cannot be started or communicated with.
    ///
    /// # Examples
    ///
    /// ```
    /// use feature_probe::Probe;
    ///
    /// let probe = Probe::new();
    /// assert!(   probe.probe_type("u32") );
    /// assert!( ! probe.probe_type("u512") );
    /// ```
    pub fn probe_type(&self, type_name: &str) -> bool {
        self.probe(&format!("fn probe_fun(_: Box<{}>) {{}} fn main() {{}} ", type_name))
    }

    /// Probes whether the given expression can be compiled.
    ///
    /// # Examples
    ///
    /// ```
    /// use feature_probe::Probe;
    ///
    /// let probe = Probe::new();
    /// assert!(   probe.probe_expression("3 + 4") );
    /// assert!( ! probe.probe_expression("3 + true") );
    pub fn probe_expression(&self, expression: &str) -> bool {
        self.probe(&format!("fn main() {{ let _ = {}; }}", expression))
    }

    /// Probes whether the given expression can be compiled.
    ///
    /// # Examples
    ///
    /// ```
    /// use feature_probe::Probe;
    ///
    /// let probe = Probe::new();
    /// assert!( ! probe.probe_expression(      "Vec::new()") );
    /// assert!(   probe.probe_typed_expression("Vec::new()", "Vec<u16>") );
    /// assert!(   probe.probe_typed_expression("Vec::new()", "Vec<u64>") );
    /// ```
    ///
    /// assert!(   probe.probe_typed_expression("3  + 4", "u32") );
    /// assert!(   probe.probe_typed_expression("3  + 4", "f32") );
    /// assert!( ! probe.probe_typed_expression("3. + 4", "u32") );
    /// assert!(   probe.probe_typed_expression("3. + 4", "f32") );
    pub fn probe_typed_expression(&self, expression: &str, type_name: &str) -> bool {
        self.probe(&format!("fn main() {{ let _: {} = {}; }}", type_name, expression))
    }

    /// Probes for whether a whole program can be compiled.
    ///
    /// # Panics
    ///
    /// If the child `rustc` cannot be started or communicated with.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate feature_probe;
    /// # fn main() {
    /// use feature_probe::Probe;
    ///
    /// let probe = Probe::new();
    /// assert!(   probe.probe("fn main() { }") );
    /// assert!( ! probe.probe("fn main(args: Vec<String>) { }") );
    /// # }
    /// ```
    pub fn probe(&self, code: &str) -> bool {
        self.probe_result(code).expect("Probe::probe")
    }

    /// Probes for whether a whole program can be compiled.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate feature_probe;
    /// # fn main() {
    /// use feature_probe::Probe;
    ///
    /// let probe = Probe::new();
    /// assert_eq!( probe.probe_result("fn main() { }").unwrap(),                  true );
    /// assert_eq!( probe.probe_result("fn main(args: Vec<String>) { }").unwrap(), false );
    /// # }
    /// ```
    pub fn probe_result(&self, code: &str) -> io::Result<bool> {
        let mut cmd = Command::new(&self.rustc);

        if self.debug {
            eprintln!("probing: {}", code);
            cmd.env("RUST_BACKTRACE", "full");
            cmd.arg("--verbose");
        }

        cmd.arg("--emit")
           .arg(&self.build_emit())
           .arg("-")
           .args(&self.rustc_args)
           .stdin(Stdio::piped())
           .stdout(Stdio::null())
           .stderr(Stdio::null());

        retry_n_times(self.retries, || {
            let _guard = RUSTC_MUTEX.lock().unwrap();
            let mut child = cmd.spawn()?;
            child.stdin.as_mut().unwrap().write_all(code.as_bytes())?;
            Ok(child.wait()?.success())
        })
    }

    fn build_emit(&self) -> String {
        format!("{}={}", self.emit_type, NULL_DEVICE)
    }
}

fn retry_n_times<T, E, F>(mut n: usize, mut f: F) -> Result<T, E>
where F: FnMut() -> Result<T, E> {
    let mut result = f();

    while result.is_err() && n > 0 {
        result = f();
        n -= 1;
    }

    result
}

impl Default for Probe {
    fn default() -> Self {
        Probe::new()
    }
}

fn env_var_or(var: &str, default: &str) -> OsString {
    env::var_os(var).unwrap_or_else(|| default.into())
}
