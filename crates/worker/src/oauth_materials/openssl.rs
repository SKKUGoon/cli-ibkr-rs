use std::path::Path;
use std::process::Command;

use crate::error::WorkerError;

pub fn ensure_available() -> Result<(), WorkerError> {
    let output = Command::new("openssl").arg("version").output()?;
    if !output.status.success() {
        return Err(WorkerError::OpenSsl(command_error(
            "openssl version",
            output.stderr,
        )));
    }
    Ok(())
}

// Result of one `openssl` invocation, in a shape we can construct in tests
// (unlike `std::process::Output` whose `ExitStatus` has no public ctor).
pub struct CommandResult {
    pub success: bool,
    pub stderr: Vec<u8>,
}

// Indirection over `Command::new("openssl")` so the keygen sequence can be
// driven by a fake in tests. Real callers use `OpensslRunner`; tests use a
// recording stub that returns canned `CommandResult`s.
pub trait CommandRunner {
    fn run(&self, args: &[&str], cwd: &Path) -> std::io::Result<CommandResult>;
}

pub struct OpensslRunner;

impl CommandRunner for OpensslRunner {
    fn run(&self, args: &[&str], cwd: &Path) -> std::io::Result<CommandResult> {
        let output = Command::new("openssl")
            .args(args)
            .current_dir(cwd)
            .output()?;
        Ok(CommandResult {
            success: output.status.success(),
            stderr: output.stderr,
        })
    }
}

// One named step in the keygen sequence. The `label` is what surfaces in
// error messages, so it should describe *what* the step is producing, not
// just echo the argv.
pub struct KeygenStep {
    pub label: &'static str,
    pub args: &'static [&'static str],
}

// Sequence run by `oauth-materials generate`. Order matters: the `rsa
// -pubout` and `pkcs8 -topk8` steps consume files written by the earlier
// `genrsa` steps in the same directory.
pub const KEYGEN_STEPS: &[KeygenStep] = &[
    KeygenStep {
        label: "generate DH parameters (2048-bit)",
        args: &["dhparam", "-out", "dhparam.pem", "-outform", "PEM", "2048"],
    },
    KeygenStep {
        label: "generate signature private key",
        args: &["genrsa", "-out", "private_signature.pem", "2048"],
    },
    KeygenStep {
        label: "generate encryption private key",
        args: &["genrsa", "-out", "private_encryption.pem", "2048"],
    },
    KeygenStep {
        label: "extract signature public key",
        args: &[
            "rsa",
            "-in",
            "private_signature.pem",
            "-outform",
            "PEM",
            "-pubout",
            "-out",
            "public_signature.pem",
        ],
    },
    KeygenStep {
        label: "extract encryption public key",
        args: &[
            "rsa",
            "-in",
            "private_encryption.pem",
            "-outform",
            "PEM",
            "-pubout",
            "-out",
            "public_encryption.pem",
        ],
    },
    KeygenStep {
        label: "convert encryption key to PKCS#8",
        args: &[
            "pkcs8",
            "-topk8",
            "-inform",
            "PEM",
            "-outform",
            "PEM",
            "-in",
            "private_encryption.pem",
            "-out",
            "private_encryption.pk8",
            "-nocrypt",
        ],
    },
    KeygenStep {
        label: "convert signature key to PKCS#8",
        args: &[
            "pkcs8",
            "-topk8",
            "-inform",
            "PEM",
            "-outform",
            "PEM",
            "-in",
            "private_signature.pem",
            "-out",
            "private_signature.pk8",
            "-nocrypt",
        ],
    },
];

pub fn run_all(out_dir: &Path) -> Result<(), WorkerError> {
    run_all_with(&OpensslRunner, out_dir)
}

pub(crate) fn run_all_with<R: CommandRunner>(
    runner: &R,
    out_dir: &Path,
) -> Result<(), WorkerError> {
    for step in KEYGEN_STEPS {
        let result = runner.run(step.args, out_dir)?;
        if !result.success {
            return Err(WorkerError::OpenSsl(command_error(
                &format!("{} (openssl {})", step.label, step.args.join(" ")),
                result.stderr,
            )));
        }
    }
    Ok(())
}

fn command_error(command: &str, stderr: Vec<u8>) -> String {
    let message = String::from_utf8_lossy(&stderr).trim().to_string();
    if message.is_empty() {
        format!("{command} failed without stderr")
    } else {
        format!("{command} failed: {message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct RecordingRunner {
        calls: RefCell<Vec<Vec<String>>>,
        // Index into KEYGEN_STEPS at which to inject a failure; None means
        // every call succeeds.
        fail_at: Option<usize>,
        fail_stderr: Vec<u8>,
    }

    impl RecordingRunner {
        fn success() -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                fail_at: None,
                fail_stderr: Vec::new(),
            }
        }

        fn failing_at(index: usize, stderr: &[u8]) -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                fail_at: Some(index),
                fail_stderr: stderr.to_vec(),
            }
        }
    }

    impl CommandRunner for RecordingRunner {
        fn run(&self, args: &[&str], _cwd: &Path) -> std::io::Result<CommandResult> {
            let recorded: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
            let mut calls = self.calls.borrow_mut();
            let index = calls.len();
            calls.push(recorded);
            if self.fail_at == Some(index) {
                Ok(CommandResult {
                    success: false,
                    stderr: self.fail_stderr.clone(),
                })
            } else {
                Ok(CommandResult {
                    success: true,
                    stderr: Vec::new(),
                })
            }
        }
    }

    #[test]
    fn run_all_invokes_every_step_in_order() {
        let runner = RecordingRunner::success();

        run_all_with(&runner, Path::new("/tmp/ignored")).expect("should succeed");

        let calls = runner.calls.borrow();
        assert_eq!(calls.len(), KEYGEN_STEPS.len());
        for (call, step) in calls.iter().zip(KEYGEN_STEPS.iter()) {
            let expected: Vec<String> = step.args.iter().map(|s| (*s).to_string()).collect();
            assert_eq!(*call, expected);
        }
    }

    #[test]
    fn run_all_stops_at_first_failure_and_includes_step_label() {
        // Fail on the third step (encryption private key generation).
        let runner = RecordingRunner::failing_at(2, b"disk full");

        let err =
            run_all_with(&runner, Path::new("/tmp/ignored")).expect_err("third step should fail");

        // No further calls should have been attempted.
        assert_eq!(runner.calls.borrow().len(), 3);
        let message = err.to_string();
        assert!(
            message.contains(KEYGEN_STEPS[2].label),
            "error should name the failing step: {message}"
        );
        assert!(
            message.contains("disk full"),
            "error should surface stderr: {message}"
        );
    }
}
