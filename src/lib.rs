use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, RwLock};

use mimetype_detector::detect as internal_detect;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};

enum Matcher {
    Magic { bytes: Vec<u8>, offset: usize },
    Callable(Py<PyAny>),
}

impl Matcher {
    fn clone_ref(&self, py: Python<'_>) -> Self {
        match self {
            Matcher::Magic { bytes, offset } => Matcher::Magic {
                bytes: bytes.clone(),
                offset: *offset,
            },
            Matcher::Callable(f) => Matcher::Callable(f.clone_ref(py)),
        }
    }
}

struct CustomType {
    matcher: Matcher,
    mime: Py<PyString>,
    extension: Py<PyString>,
}

impl CustomType {
    fn clone_ref(&self, py: Python<'_>) -> Self {
        Self {
            matcher: self.matcher.clone_ref(py),
            mime: self.mime.clone_ref(py),
            extension: self.extension.clone_ref(py),
        }
    }

    fn matches(&self, py: Python<'_>, data: &[u8]) -> PyResult<bool> {
        let (bytes, offset) = match &self.matcher {
            Matcher::Callable(f) => {
                return f.call1(py, (PyBytes::new(py, data),))?.bind(py).is_truthy()
            }
            Matcher::Magic { bytes, offset } => (bytes, *offset),
        };
        let Some(end) = offset.checked_add(bytes.len()) else {
            return Ok(false);
        };
        if data.len() < end {
            return Ok(false);
        }
        Ok(&data[offset..end] == bytes.as_slice())
    }
}

/// Registered types, swapped wholesale so the hot path never holds the lock
/// across a call back into Python.
static CUSTOM: LazyLock<RwLock<Arc<Vec<CustomType>>>> =
    LazyLock::new(|| RwLock::new(Arc::new(Vec::new())));

/// Lets the common case skip the lock entirely.
static HAS_CUSTOM: AtomicBool = AtomicBool::new(false);

fn custom_hit<'py>(
    py: Python<'py>,
    data: &[u8],
    pick: fn(&CustomType) -> &Py<PyString>,
) -> PyResult<Option<Bound<'py, PyString>>> {
    if !HAS_CUSTOM.load(Ordering::Relaxed) {
        return Ok(None);
    }
    let snapshot = {
        let guard = CUSTOM.read().unwrap();
        Arc::clone(&guard)
    };
    for entry in snapshot.iter() {
        if !entry.matches(py, data)? {
            continue;
        }
        return Ok(Some(pick(entry).bind(py).clone()));
    }
    Ok(None)
}

/// Detects the mime type of a byte array.
#[pyfunction]
fn detect_mime<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyString>> {
    if let Some(hit) = custom_hit(py, data, |entry| &entry.mime)? {
        return Ok(hit);
    }
    Ok(PyString::new(py, internal_detect(data).mime()))
}

/// Detects the extension of a byte array.
#[pyfunction]
fn detect_type<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyString>> {
    if let Some(hit) = custom_hit(py, data, |entry| &entry.extension)? {
        return Ok(hit);
    }
    Ok(PyString::new(py, internal_detect(data).extension()))
}

fn build_matcher(
    magic: Option<Vec<u8>>,
    offset: usize,
    matcher: Option<Py<PyAny>>,
) -> PyResult<Matcher> {
    if magic.is_some() && matcher.is_some() {
        return Err(PyValueError::new_err(
            "pass either magic or matcher, not both",
        ));
    }
    if let Some(f) = matcher {
        return Ok(Matcher::Callable(f));
    }
    let Some(bytes) = magic else {
        return Err(PyValueError::new_err("pass either magic or matcher"));
    };
    if bytes.is_empty() {
        return Err(PyValueError::new_err("magic must not be empty"));
    }
    Ok(Matcher::Magic { bytes, offset })
}

/// Registers a custom type, matched by magic bytes or by a Python callable.
///
/// Registered types are checked before the built-in table, in registration
/// order, so they can add new formats or override a built-in verdict.
#[pyfunction]
#[pyo3(signature = (mime, extension, *, magic=None, offset=0, matcher=None))]
fn register(
    py: Python<'_>,
    mime: &str,
    extension: &str,
    magic: Option<Vec<u8>>,
    offset: usize,
    matcher: Option<Py<PyAny>>,
) -> PyResult<()> {
    let entry = CustomType {
        matcher: build_matcher(magic, offset, matcher)?,
        mime: PyString::new(py, mime).unbind(),
        extension: PyString::new(py, extension).unbind(),
    };
    let mut guard = CUSTOM.write().unwrap();
    let mut next: Vec<CustomType> = guard.iter().map(|e| e.clone_ref(py)).collect();
    next.push(entry);
    *guard = Arc::new(next);
    HAS_CUSTOM.store(true, Ordering::Relaxed);
    Ok(())
}

/// Returns the registered (mime, extension) pairs in registration order.
#[pyfunction]
fn registered(py: Python<'_>) -> Vec<(Py<PyString>, Py<PyString>)> {
    CUSTOM
        .read()
        .unwrap()
        .iter()
        .map(|e| (e.mime.clone_ref(py), e.extension.clone_ref(py)))
        .collect()
}

/// Drops every registered type.
#[pyfunction]
fn clear_registrations() {
    let mut guard = CUSTOM.write().unwrap();
    *guard = Arc::new(Vec::new());
    HAS_CUSTOM.store(false, Ordering::Relaxed);
}

/// A Python module implemented in Rust.
#[pymodule(gil_used = false)]
fn mimey(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(detect_mime, m)?)?;
    m.add_function(wrap_pyfunction!(detect_type, m)?)?;
    m.add_function(wrap_pyfunction!(register, m)?)?;
    m.add_function(wrap_pyfunction!(registered, m)?)?;
    m.add_function(wrap_pyfunction!(clear_registrations, m)?)?;
    Ok(())
}
