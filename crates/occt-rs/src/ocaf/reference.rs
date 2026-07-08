//! Reference: https://dev.opencascade.org/doc/refman/html/class_t_d_f___reference.html

use occt_sys::ffi;

use crate::ocaf::OcLabel;

/// Zero-size marker — `TDF_Reference::Set`/`Get` are static OCCT operations;
/// no instance state is needed on the Rust side.
pub struct OcReference;

impl OcReference {
    /// Stores a reference from `at` to `target`.
    ///
    /// # Panics
    /// Panics if called outside an open command — `TDF_Reference::Set`
    /// requires a live transaction, same as `TDataStd_Name::Set` etc.
    /// TODO: consider returning `Result` instead, to match how command-scope
    /// errors are surfaced elsewhere in this crate, rather than panicking —
    /// check the convention used by `TDataStd_Name::set` before finalizing.
    pub fn set(at: &OcLabel, target: &OcLabel) {
        ffi::tdf_reference_set(&at.inner, &target.inner)
            .expect("TDF_Reference::Set requires an open command");
    }

    /// Returns the label `at` points to, or `None` if `at` has no
    /// `TDF_Reference` attribute, OR if the attribute exists but its stored
    /// origin is itself a null label (uses `from_ffi`, not `_unchecked`,
    /// specifically to fold that second case into `None` rather than
    /// panicking or returning a garbage `OcLabel` — same reasoning as
    /// `OcLabel::father()` returning `None` on the root).
    pub fn get(at: &OcLabel) -> Option<OcLabel> {
        let handle = ffi::tdf_reference_find(&at.inner);
        if handle.is_null() {
            return None;
        }
        let label = ffi::tdf_reference_get(&handle);
        OcLabel::from_ffi(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocaf::OcApplication;

    #[test]
    fn reference_round_trips_through_get() {
        let mut app = OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();
        let main = doc.main();

        doc.begin_command().unwrap();
        let source = main.get_or_create_child(1);
        let at = main.get_or_create_child(2);
        OcReference::set(&at, &source);
        doc.commit().unwrap();

        let found = OcReference::get(&at).expect("reference was just set");
        assert_eq!(found.entry(), source.entry());
    }

    #[test]
    fn reference_get_returns_none_when_absent() {
        let mut app = OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();
        let main = doc.main();

        doc.begin_command().unwrap();
        let empty = main.get_or_create_child(99);
        doc.commit().unwrap();

        assert!(OcReference::get(&empty).is_none());
    }
}
