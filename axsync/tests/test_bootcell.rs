use axsync::BootOnceCell;

static TEST: BootOnceCell<usize> = BootOnceCell::new();

#[test]
fn test_bootcell() {
    assert!(!TEST.is_init());
    TEST.init(101);
    assert_eq!(TEST.get(), &101);
    assert!(TEST.is_init());
}
