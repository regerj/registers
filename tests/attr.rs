use anyhow::Result;
use registers::register;

#[register]
struct HIF1 {
    #[field(signed = false, lsb = 0, msb = 15, write = false)]
    lower: u32,
    #[field(signed = false, lsb = 16, msb = 31)]
    upper: u32,
}

#[test]
pub fn test_attr() -> Result<()> {
    let mut reg = HIF1::new(0xDEADBEEF);
    let upper = reg.get_upper();
    let lower = reg.get_lower();

    assert_eq!(upper, 0xDEAD);
    assert_eq!(lower, 0xBEEF);

    assert!(reg.set_upper(0xFFFF).is_ok());

    assert_eq!(reg.get_upper(), 0xFFFF);

    assert!(reg.set_upper(0xDEADF).is_err());

    assert_eq!(reg.get_upper(), 0xFFFF);

    assert_eq!(reg.get_lower(), 0xBEEF);

    Ok(())
}

#[test]
fn test_from() {
    let reg: HIF1 = 0xDEADBEEF.into();

    assert_eq!(reg.get_upper(), 0xDEAD);
    assert_eq!(reg.get_lower(), 0xBEEF);

    let reg = HIF1::from(0xDEADBEEF);

    assert_eq!(reg.get_upper(), 0xDEAD);
    assert_eq!(reg.get_lower(), 0xBEEF);
}

#[test]
fn test_into() {
    let reg = HIF1::new(0xDEADBEEF);
    assert_eq!(<HIF1 as Into<u32>>::into(reg), 0xDEADBEEF);
}

#[test]
fn test_clear() {
    let mut reg = HIF1::new(0xDEADBEEF);
    reg.clear();
    assert_eq!(<HIF1 as Into<u32>>::into(reg), 0);
}
