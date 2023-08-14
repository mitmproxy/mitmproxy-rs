#[allow(unused_imports)]
use anyhow::{anyhow, Result};
use data_encoding::BASE64;
#[cfg(target_os = "macos")]
use mitmproxy::macos;
use pyo3::exceptions::PyOSError;
use pyo3::types::{PyString, PyTuple};
use pyo3::{exceptions::PyValueError, prelude::*};
use rand_core::OsRng;
//#[cfg(any(test, target_os = "macos"))]
use std::fs;
use std::net::{IpAddr, SocketAddr};
//#[cfg(any(test, target_os = "macos"))]
use std::path::Path;
use std::str::FromStr;
use tokio::sync::mpsc;
use x25519_dalek::{PublicKey, StaticSecret};

pub fn string_to_key<T>(data: String) -> PyResult<T>
where
    T: From<[u8; 32]>,
{
    BASE64
        .decode(data.as_bytes())
        .ok()
        .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
        .map(T::from)
        .ok_or_else(|| PyValueError::new_err("Invalid key."))
}

pub fn socketaddr_to_py(py: Python, s: SocketAddr) -> PyObject {
    match s {
        SocketAddr::V4(addr) => (addr.ip().to_string(), addr.port()).into_py(py),
        SocketAddr::V6(addr) => {
            log::debug!(
                "Converting IPv6 address/port to Python equivalent (not sure if this is correct): {:?}",
                (addr.ip().to_string(), addr.port())
            );
            (addr.ip().to_string(), addr.port()).into_py(py)
        }
    }
}

//#[cfg(any(test, target_os = "macos"))]
pub fn copy_dir(src: &Path, dst: &Path) -> PyResult<()> {
    for entry in src.read_dir()? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            fs::create_dir_all(&dst_path).expect("Failed to create directory");
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).expect("Failed to copy {src_path} to {dst_path}");
        }
    }
    Ok(())
}

pub fn py_to_socketaddr(t: &PyTuple) -> PyResult<SocketAddr> {
    if t.len() == 2 {
        let host = t.get_item(0)?.downcast::<PyString>()?;
        let port: u16 = t.get_item(1)?.extract()?;

        let addr = IpAddr::from_str(host.to_str()?)?;
        Ok(SocketAddr::from((addr, port)))
    } else {
        Err(PyValueError::new_err("not a socket address"))
    }
}

pub fn event_queue_unavailable<T>(_: mpsc::error::SendError<T>) -> PyErr {
    PyOSError::new_err("Server has been shut down.")
}

/// Generate a WireGuard private key, analogous to the `wg genkey` command.
#[pyfunction]
pub fn genkey() -> String {
    BASE64.encode(&StaticSecret::random_from_rng(OsRng).to_bytes())
}

/// Derive a WireGuard public key from a private key, analogous to the `wg pubkey` command.
#[pyfunction]
pub fn pubkey(private_key: String) -> PyResult<String> {
    let private_key: StaticSecret = string_to_key(private_key)?;
    Ok(BASE64.encode(PublicKey::from(&private_key).as_bytes()))
}

/// Convert pem certificate to der certificate and add it to macos keychain.
#[pyfunction]
#[allow(unused_variables)]
pub fn add_cert(py: Python<'_>, pem: String) -> PyResult<()> {
    #[cfg(target_os = "macos")]
    {
        let pem_body = pem
            .lines()
            .skip(1)
            .take_while(|&line| line != "-----END CERTIFICATE-----")
            .collect::<String>();

        let filename = py.import("mitmproxy_rs")?.filename()?;
        let executable_path = std::path::Path::new(filename)
            .parent()
            .ok_or_else(|| anyhow!("invalid path"))?
            .join("macos-certificate-truster.app");
        if !executable_path.exists() {
            return Err(anyhow!("{} does not exist", executable_path.display()).into());
        }
        let der = BASE64.decode(pem_body.as_bytes()).unwrap();
        match macos::add_cert(der, executable_path.to_str().unwrap()) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyErr::new::<PyOSError, _>(format!(
                "Failed to add certificate: {:?}",
                e
            ))),
        }
    }
    #[cfg(not(target_os = "macos"))]
    Err(pyo3::exceptions::PyNotImplementedError::new_err(
        "OS proxy mode is only available on macos",
    ))
}

/// Delete mitmproxy certificate from the keychain.
#[pyfunction]
pub fn remove_cert() -> PyResult<()> {
    #[cfg(target_os = "macos")]
    {
        match macos::remove_cert() {
            Ok(_) => Ok(()),
            Err(e) => Err(PyErr::new::<PyOSError, _>(format!(
                "Failed to remove certificate: {:?}",
                e
            ))),
        }
    }
    #[cfg(not(target_os = "macos"))]
    Err(pyo3::exceptions::PyNotImplementedError::new_err(
        "OS proxy mode is only available on macos",
    ))
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_copy_dir() -> Result<()> {
        let a = tempdir()?;
        let b = tempdir()?;

        fs::create_dir_all(a.path().join("foo/bar"))?;
        File::create(a.path().join("foo/bar/baz.txt"))?;

        copy_dir(a.path(), b.path())?;

        fs::metadata(b.path().join("foo/bar/baz.txt"))?;

        Ok(())
    }
}
*/