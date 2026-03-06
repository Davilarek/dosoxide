#[repr(C, packed)]
#[derive(Default, Debug)]
pub struct RmRegs {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    pub eflags: u32,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
}

unsafe extern "C" {
    pub fn call_rm_int(int_no: u8, regs: *mut RmRegs);
}

pub fn dos_print(s: &[u8]) {
    let mut regs = RmRegs::default();
    for &c in s {
        regs.eax = 0x0200;
        regs.edx = c as u32;
        unsafe {
            call_rm_int(0x21, &mut regs);
        }
    }
}

pub fn get_extended_memory_kb() -> u32 {
    let mut regs = RmRegs::default();
    // regs.eax = 0xE801;
    regs.eax = 0xE820;
    unsafe {
        call_rm_int(0x15, &mut regs);
    }

    if (regs.eflags & 1) != 0 {
        regs.eax = 0x8800;
        unsafe {
            call_rm_int(0x15, &mut regs);
        }
        return (regs.eax & 0xFFFF) as u32;
    }
    let low = (regs.eax & 0xFFFF) as u32;
    let high = (regs.ebx & 0xFFFF) as u32 * 64;
    low + high
}
