//! Top level application management: Open/Close/Create/etc. documents
//!
//! [`OcApplication`] owns a `TDocStd_Application` Handle and is the factory
//! for [`OcDocument`] instances. At the application development level, it can be considered
//! the prinvcipal means of overall data management. If you have multiple tabs, one for
//! each project, [`OcApplication`] is what manages the tabs.

use std::marker::PhantomData;

use occt_sys::ffi;

use crate::error::OcctError;
use crate::ocaf::document::OcDocument;

/// An OCAF application — the factory and registry for [`OcDocument`] instances.
///
/// Wraps `Handle(TDocStd_Application)`.  Each call to [`new`] produces an
/// independent application; sharing between Rust owners is the caller's
/// concern.
///
/// # Thread safety
///
/// OCCT Handle ref-counting is not atomic.  `OcApplication` must not be sent
/// across thread boundaries.
///
/// [`new`]: OcApplication::new
pub struct OcApplication {
    inner: cxx::UniquePtr<ffi::ApplicationHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcApplication {
    /// Creates a new `TDocStd_Application` instance.
    pub fn new() -> Self {
        Self {
            inner: ffi::new_application(),
            _not_send: PhantomData,
        }
    }

    /// Creates a new in-memory document under this application.
    ///
    /// `format` is a label string stored on the document (e.g. `"BinXCAF"`).
    /// Persistence drivers for that format need not be registered for
    /// in-memory use; they are only required when saving or loading.
    ///
    /// # Errors
    ///
    /// Returns `Err` if OCCT raises or returns a null document handle.
    pub fn new_document(&mut self, format: &str) -> Result<OcDocument, OcctError> {
        let inner =
            ffi::application_new_document(self.inner.pin_mut(), format).map_err(OcctError::from)?;
        Ok(OcDocument::from_ffi(inner))
    }

    /// Returns the number of documents currently registered with this application.
    ///
    /// Useful as a test hook to confirm that dropping an [`OcDocument`] deregisters it.
    pub fn nb_documents(&self) -> i32 {
        ffi::application_nb_documents(&self.inner)
    }
}

impl Default for OcApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for OcApplication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcApplication").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_application_succeeds() {
        let _app = OcApplication::new();
    }

    #[test]
    fn new_document_succeeds() {
        let mut app = OcApplication::new();
        assert!(app.new_document("BinXCAF").is_ok());
    }

    #[test]
    fn multiple_documents_independent() {
        let mut app = OcApplication::new();
        let doc_a = app.new_document("BinXCAF").unwrap();
        let doc_b = app.new_document("BinXCAF").unwrap();
        // Both documents have valid (non-null) main labels.
        assert!(!doc_a.main().inner.is_null());
        assert!(!doc_b.main().inner.is_null());
    }

    #[test]
    fn dropping_document_deregisters_it_from_application() {
        let mut app = OcApplication::new();
        assert_eq!(app.nb_documents(), 0);
        {
            let _doc = app.new_document("BinXCAF").unwrap();
            assert_eq!(app.nb_documents(), 1);
        } // _doc dropped here → Drop calls document_close → deregisters
        assert_eq!(
            app.nb_documents(),
            0,
            "document leaked: still registered after drop"
        );
    }

    #[test]
    fn explicit_close_deregisters_and_drop_is_safe() {
        let mut app = OcApplication::new();
        let doc = app.new_document("BinXCAF").unwrap();
        assert_eq!(app.nb_documents(), 1);
        doc.close().unwrap();
        // After explicit close, document is gone from the registry.
        assert_eq!(app.nb_documents(), 0);
        // Drop of the now-moved `doc` value already happened in close(); no double-close.
    }
}
