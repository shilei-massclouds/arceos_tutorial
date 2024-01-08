use std::str;
use std::io::Read;
use axdtb::SliceRead;

#[test]
fn test_dtb() {
    let mut input = std::fs::File::open("tests/sample.dtb").unwrap();
    let mut buf = Vec::new();
    input.read_to_end(&mut buf).unwrap();

    let mut cb = |name: String, addr_cells: usize, size_cells: usize, props: Vec<(String, Vec<u8>)>| {
        match name.as_str() {
            "" => {
                assert_eq!(addr_cells, 2);
                assert_eq!(size_cells, 2);
                for prop in &props {
                    if prop.0.as_str() == "compatible" {
                        assert_eq!(str::from_utf8(&(prop.1)), Ok("riscv-virtio\0"));
                    }
                }
            },
            "soc" => {
                assert_eq!(addr_cells, 2);
                assert_eq!(size_cells, 2);
                for prop in &props {
                    if prop.0.as_str() == "compatible" {
                        assert_eq!(str::from_utf8(&(prop.1)), Ok("simple-bus\0"));
                    }
                }
            },
            "virtio_mmio@10001000" => {
                assert_eq!(addr_cells, 2);
                assert_eq!(size_cells, 2);
                for prop in &props {
                    if prop.0.as_str() == "compatible" {
                        assert_eq!(str::from_utf8(&(prop.1)), Ok("virtio,mmio\0"));
                    } else if prop.0.as_str() == "reg" {
                        let reg = &(prop.1);
                        assert_eq!(reg.as_slice().read_be_u64(0).unwrap(), 0x10001000);
                        assert_eq!(reg.as_slice().read_be_u64(8).unwrap(), 0x1000);
                    }
                }
            },
            _ => {}
        }
    };

    let dt = axdtb::DeviceTree::init(buf.as_slice().as_ptr() as usize).unwrap();
    assert_eq!(dt.parse(dt.off_struct, 0, 0, &mut cb).unwrap(), 396);
}
