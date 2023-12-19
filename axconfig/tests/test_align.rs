use axconfig::{align_up, align_down, PAGE_SIZE};

#[test]
fn test_align_up() {
    assert_eq!(align_up(23, 16), 32);
    assert_eq!(align_up(4095, PAGE_SIZE), PAGE_SIZE);
    assert_eq!(align_up(4096, PAGE_SIZE), PAGE_SIZE);
    assert_eq!(align_up(4097, PAGE_SIZE), 2*PAGE_SIZE);
}

#[test]
fn test_align_down() {
    assert_eq!(align_down(23, 16), 16);
    assert_eq!(align_down(4095, PAGE_SIZE), 0);
    assert_eq!(align_down(4096, PAGE_SIZE), PAGE_SIZE);
    assert_eq!(align_down(4097, PAGE_SIZE), PAGE_SIZE);
}
