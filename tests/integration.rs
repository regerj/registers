use registers::register;

#[register]
struct HIF1 {
    #[field(lsb = 0, msb = 15)]
    lower: u32,
    #[field(lsb = 16, msb = 31)]
    upper: u32,
}

#[register]
struct Signed {
    #[field(lsb = 0, msb = 15, signed = true)]
    lower: u32,
    #[field(lsb = 16, msb = 31)]
    upper: u32,
}

#[register]
struct NonStandardSigned {
    #[field(lsb = 0, msb = 3, signed = true)]
    lower_four: u32,
    #[field(lsb = 4, msb = 31, signed = false)]
    reserved: u32,
}

#[test]
pub fn test_get() {
    let reg = HIF1::from(0xDEADBEEF);
    let upper = reg.get_upper();
    let lower = reg.get_lower();

    assert_eq!(upper, 0xDEAD);
    assert_eq!(lower, 0xBEEF);
}

#[test]
fn test_set() -> registers::Result<()> {
    let mut reg = HIF1::new();
    assert_eq!(reg.get_upper(), 0);
    assert_eq!(reg.get_lower(), 0);

    reg.set_upper(0xDEAD)?;
    reg.set_lower(0xBEEF)?;

    assert_eq!(reg.get_upper(), 0xDEAD);
    assert_eq!(reg.get_lower(), 0xBEEF);

    assert_eq!(
        reg.set_upper(0x1_0000),
        Err(registers::Error::OutOfBoundsFieldWrite)
    );
    assert_eq!(
        reg.set_lower(0x1_0000),
        Err(registers::Error::OutOfBoundsFieldWrite)
    );

    assert_eq!(reg.get_upper(), 0xDEAD);
    assert_eq!(reg.get_lower(), 0xBEEF);

    Ok(())
}

#[test]
fn test_new() {
    let reg = HIF1::new();
    assert_eq!(reg, 0);
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
    let reg = HIF1::from(0xDEADBEEF);
    assert_eq!(<HIF1 as Into<u32>>::into(reg), 0xDEADBEEF);
}

#[test]
fn test_clear() {
    let mut reg = HIF1::from(0xDEADBEEF);
    reg.clear();
    assert_eq!(<HIF1 as Into<u32>>::into(reg), 0);
}

#[test]
fn test_eq() {
    let reg = HIF1::from(0xDEADBEEF);
    assert_eq!(reg, 0xDEADBEEF);
}

#[test]
fn test_clone() {
    let reg = HIF1::from(0xDEADBEEF);
    assert_eq!(reg.clone(), 0xDEADBEEF);
}

#[test]
fn test_signed() {
    let mut reg = Signed::new();
    assert!(reg.set_lower(i16::MIN.into()).is_ok());

    assert_eq!(reg.get_upper(), 0);
    assert_eq!(reg.get_lower(), i16::MIN.into());

    assert!(reg.set_upper(u16::MAX.into()).is_ok());
    assert_eq!(reg.get_upper(), u16::MAX.into());
    assert_eq!(reg.get_lower(), i16::MIN.into());
}

#[test]
fn test_nonstd_signed() {
    let mut reg = NonStandardSigned::new();

    assert!(reg.set_lower_four(-8).is_ok());
    assert_eq!(reg.get_lower_four(), -8);
    assert_eq!(reg.get_reserved(), 0);

    assert!(reg.set_lower_four(-5).is_ok());
    assert_eq!(reg.get_lower_four(), -5);
    assert_eq!(reg.get_reserved(), 0);

    assert!(reg.set_lower_four(7).is_ok());
    assert_eq!(reg.get_lower_four(), 7);
    assert_eq!(reg.get_reserved(), 0);
}

#[test]
fn test_read() {
    let some_value: u32 = 0xDEADBEEF;
    let addr = &some_value as *const u32;

    let mut reg = HIF1::new();
    unsafe {
        reg.read(addr);
    }

    assert_eq!(reg.get_upper(), 0xDEAD);
    assert_eq!(reg.get_lower(), 0xBEEF);
}

#[test]
fn test_write() {
    let mut some_value: u32 = 0;
    let addr = &mut some_value as *mut u32;

    let mut reg = HIF1::new();
    reg.set_upper(0xDEAD).expect("Could not fit into bounds");
    reg.set_lower(0xBEEF).expect("Could not fit into bounds");

    unsafe {
        reg.write(addr);
    }

    assert_eq!(some_value, 0xDEADBEEF);
}

#[test]
fn test_size() {
    assert_eq!(size_of::<HIF1>(), size_of::<u32>());
}
