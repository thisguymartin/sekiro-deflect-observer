use std::ffi::OsStr;
use std::io::{self, Read};

use sha2::{Digest, Sha256};

/// Checks the host name only. It does not establish compatible game offsets.
pub fn is_sekiro_host(file_name: &OsStr) -> bool {
    file_name
        .to_str()
        .is_some_and(|name| name.eq_ignore_ascii_case("sekiro.exe"))
}

pub fn sha256(reader: &mut impl Read) -> io::Result<String> {
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => digest.update(&buffer[..count]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(format!("{:x}", digest.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_game_name_without_case_sensitivity() {
        assert!(is_sekiro_host(OsStr::new("sekiro.exe")));
        assert!(is_sekiro_host(OsStr::new("SEKIRO.EXE")));
    }

    #[test]
    fn rejects_unrelated_executables_and_similar_names() {
        for name in [
            "notepad.exe",
            "sekiro.exe.bak",
            "sekiro",
            "",
            "my-sekiro.exe",
        ] {
            assert!(!is_sekiro_host(OsStr::new(name)), "accepted {name}");
        }
    }

    #[test]
    fn computes_a_known_sha256() {
        assert_eq!(
            sha256(&mut &b"abc"[..]).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn preserves_read_failure_instead_of_returning_a_partial_hash() {
        struct FailingReader(bool);
        impl Read for FailingReader {
            fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
                if self.0 {
                    return Err(io::Error::new(io::ErrorKind::PermissionDenied, "fixture"));
                }
                self.0 = true;
                buffer[0] = b'a';
                Ok(1)
            }
        }
        assert_eq!(
            sha256(&mut FailingReader(false)).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}
