//! The pinned toolchain must be present in every development and CI environment
//! (run scripts/setup.sh). These tests fail rather than skip when it is not.

#[test]
fn z3_is_pinned_version() {
    assert_eq!(
        bmb::toolchain::check_z3(),
        Ok(bmb::toolchain::z3_version().to_string())
    );
}

#[test]
fn llvm_is_pinned_major() {
    assert_eq!(
        bmb::toolchain::check_llvm(),
        Ok(bmb::toolchain::llvm_major().to_string())
    );
}
