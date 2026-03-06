#![no_std]
#![no_main]
mod dos;
use core::arch::asm;
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

struct BumpAllocator {
    next: AtomicUsize,
}

fn print_decimal_into_buf(mut value: u32, buf: &mut [u8], pos: usize) -> usize {
    if value == 0 {
        buf[pos] = b'0';
        return pos + 1;
    }
    let mut i = pos;
    while value > 0 && i < buf.len() {
        buf[i] = b'0' + (value % 10) as u8;
        value /= 10;
        i += 1;
    }
    let mut start = pos;
    let mut end = i - 1;
    while start < end {
        buf.swap(start, end);
        start += 1;
        end -= 1;
    }
    i
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let current = self.next.load(Ordering::Relaxed);
        let aligned = (current + layout.align() - 1) & !(layout.align() - 1);
        let new_end = aligned + layout.size();

        let esp: u32;
        asm!("mov {}, esp", out(reg) esp);

        dos::dos_print(b"Alloc at: ");
        print_decimal(aligned as u32);
        dos::dos_print(b"\tESP: ");
        print_decimal(esp);
        dos::dos_print(b"\r\n");
        let mut slow_down_every_next = false;

        if aligned >= 42004401 { // actually the program crashes on the 4200400 allocation, which is the 262nd 16KB block, which means we have allocated 4192 KB, But why?
            let eax: u32;
            let ebx: u32;
            let ecx: u32;
            let edx: u32;
            let esi: u32;
            let edi: u32;
            let eflags: u32;
            asm!(
                "mov {}, eax",
                "mov {}, ebx",
                "mov {}, ecx",
                "mov {}, edx",
                "mov {}, esi",
                "mov {}, edi",
                "pushf",
                "pop {}",
                out(reg) eax,
                out(reg) ebx,
                out(reg) ecx,
                out(reg) edx,
                out(reg) esi,
                out(reg) edi,
                out(reg) eflags,
            );
            unsafe {
                let vga_base = 0xB8000 as *mut u8;
                let mut msg = [0u8; 200];
                let mut pos = 0;

                fn advance_and_print_label(label: &[u8], buf: &mut [u8], pos: usize) -> usize {
                    let mut new_pos = pos;
                    for &c in label {
                        buf[new_pos] = c;
                        new_pos += 1;
                    }
                    new_pos
                }

                fn print_into_buf(value: u32, buf: &mut [u8], pos: usize) -> usize {
                    let mut new_pos = pos;
                    new_pos = print_decimal_into_buf(value, buf, new_pos);
                    buf[new_pos] = b' ';
                    new_pos + 1
                }

                pos = advance_and_print_label(b"OOM at: ", &mut msg, pos);
                pos = print_into_buf(aligned as u32, &mut msg, pos);
                pos = advance_and_print_label(b"ESP: ", &mut msg, pos);
                pos = print_into_buf(esp, &mut msg, pos);
                pos = advance_and_print_label(b"EAX: ", &mut msg, pos);
                pos = print_into_buf(eax, &mut msg, pos);
                pos = advance_and_print_label(b"EBX: ", &mut msg, pos);
                pos = print_into_buf(ebx, &mut msg, pos);
                pos = advance_and_print_label(b"ECX: ", &mut msg, pos);
                pos = print_into_buf(ecx, &mut msg, pos);
                pos = advance_and_print_label(b"EDX: ", &mut msg, pos);
                pos = print_into_buf(edx, &mut msg, pos);
                pos = advance_and_print_label(b"ESI: ", &mut msg, pos);
                pos = print_into_buf(esi, &mut msg, pos);
                pos = advance_and_print_label(b"EDI: ", &mut msg, pos);
                pos = print_into_buf(edi, &mut msg, pos);
                pos = advance_and_print_label(b"EFLAGS: ", &mut msg, pos);

                pos = print_decimal_into_buf(eflags, &mut msg, pos);
                for (i, &c) in msg.iter().enumerate() {
                    asm!(
                        "mov byte ptr fs:[ebx], al",
                        "mov byte ptr fs:[ebx+1], 0x4F",
                        in("ebx") vga_base.add(i * 2),
                        in("al") c,
                    );
                }
            }
            slow_down_every_next = true;
        }
        if slow_down_every_next {
            for _ in 0..1000000 {
                asm!("nop");
            }
            let msg = b"Slowing down due to OOM...";
            unsafe {
                let vga_base = 0xB8000 as *mut u8;
                for (i, &c) in msg.iter().enumerate() {
                    asm!(
                        "mov byte ptr fs:[ebx], al",
                        "mov byte ptr fs:[ebx+1], 0x4F",
                        in("ebx") vga_base.add(i * 2).add(100 * 2),
                        in("al") c,
                    );
                }
            }
            for _ in 0..1000000 {
                asm!("nop");
            }
            unsafe {
                let vga_base = 0xB8000 as *mut u8;
                for i in 0..msg.iter().len() {
                    asm!(
                        "mov byte ptr fs:[ebx], al",
                        "mov byte ptr fs:[ebx+1], 0x4F",
                        in("ebx") vga_base.add(i * 2).add(100 * 2),
                        in("al") b' ',
                    );
                }
            }
        }

        if new_end >= 0x1F00000 {
            return core::ptr::null_mut();
        }
        self.next.store(new_end, Ordering::Relaxed);
        aligned as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator {
    next: AtomicUsize::new(0x100000),
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = c as u8;
        i += 1;
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    dest
}

unsafe extern "C" {
    // static mut PAGING_BUFFER: [u8; 12288];
    static mut PAGING_BUFFER: [u8; 40960];
    // static mut PAGING_BUFFER: [u8; 81920];
}
fn print_decimal(mut value: u32) {
    if value == 0 {
        dos::dos_print(b"0");
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    while value > 0 && i > 0 {
        i -= 1;
        buf[i] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    dos::dos_print(&buf[i..]);
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        let msg = b"PANIC!";
        for (i, &c) in msg.iter().enumerate() {
            asm!(
                "mov byte ptr fs:[ebx], al",
                "mov byte ptr fs:[ebx+1], 0x4F",
                in("ebx") 0xB8000 + (i * 2),
                in("al") c,
            );
        }
    }
    loop {}
}

pub fn test_allocator_capacity() {
    use alloc::vec::Vec;

    const BLOCK_SIZE: usize = 16 * 1024;
    let mut blocks: Vec<Vec<u8>> = Vec::new();
    let mut total_kb = 0;

    dos::dos_print(b"Starting Memory Stress Test (16KB blocks)...\r\n");

    loop {
        let mut block = Vec::new();
        match block.try_reserve_exact(BLOCK_SIZE) {
            Ok(_) => {
                unsafe {
                    let ptr = block.as_mut_ptr();
                    core::ptr::write_bytes(ptr, 0, BLOCK_SIZE);
                    block.set_len(BLOCK_SIZE);
                }

                blocks.push(block);
                total_kb += 16;

                if total_kb % 1024 == 0 {
                    dos::dos_print(b".");
                }
            }
            Err(_) => {
                dos::dos_print(b"\r\n\r\n[!] OUT OF MEMORY\r\n");
                break;
            }
        }
    }

    dos::dos_print(b"Max Allocated: ");
    print_decimal(total_kb);
    dos::dos_print(b" KB\r\n");

    let total_blocks = blocks.len();
    dos::dos_print(b"Total Blocks:  ");
    print_decimal(total_blocks as u32);
    dos::dos_print(b"\r\n");

    drop(blocks);
    dos::dos_print(b"Memory Freed.\r\n");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_main(load_base: u32) -> ! {
    unsafe { setup_paging(load_base) };
    unsafe {
        asm!(
            "mov fs, ax",
            "mov byte ptr fs:[0xB8000], '!'",
            "mov byte ptr fs:[0xB8001], 0x0E",
            in("ax") 0x28u16,
        );
    }
    unsafe {
        asm!(
            "mov byte ptr fs:[0xB8002], '?'",
            "mov byte ptr fs:[0xB8003], 0x0C",
        );
    }
    dos::dos_print(b"Paging Enabled Successfully!\r\n");
    unsafe {
        asm!(
            "mov byte ptr fs:[0xB8004], 'q'",
            "mov byte ptr fs:[0xB8005], 0x0C",
        );
    }
    dos::dos_print(b"DOS Extender runs off base:");
    print_decimal(load_base);
    dos::dos_print(b".\r\n");
    let mem_kb = unsafe { dos::get_extended_memory_kb() };
    dos::dos_print(b"Conventional memory: ");
    print_decimal(mem_kb);
    dos::dos_print(b" KB\r\n");

    /* // uncommenting this makes the OOM at 4 MB, without this, the OOM happens at 8 MB!
    let mut numbers = alloc::vec::Vec::new();
    numbers.push(123);
    numbers.push(456);

    let msg = alloc::string::String::from("Hello from the Heap!\r\n");
    dos::dos_print(msg.as_bytes());
    print_decimal(*numbers.get(0).unwrap());
    print_decimal(*numbers.get(1).unwrap());

    dos::dos_print(b"Test?\r\n");
    */
    // print anything 100 times
    // for i in 0..10000 {
    //     print_decimal(i);
    //     dos::dos_print(b"Testing allocator capacity...\r\n");
    // }

    test_allocator_capacity();

    loop {}
}

unsafe fn setup_paging(load_base: u32) {
    let buffer_ptr = addr_of_mut!(PAGING_BUFFER) as *mut u8;
    let buffer_phys_start = load_base + (buffer_ptr as u32);
    let pd_phys = (buffer_phys_start + 4095) & !4095;
    let alignment_padding = pd_phys - buffer_phys_start;
    let pd_ptr = buffer_ptr.add(alignment_padding as usize) as *mut u32;

    let pt0_phys = pd_phys + 4096;

    for i in 0..8 {
        let pt_phys = pt0_phys + (i * 4096);

        *pd_ptr.add(i as usize) = pt_phys | 0x03;

        let pt_ptr = pd_ptr.add(1024 + (i as usize * 1024));

        for j in 0..1024 {
            let page_phys = (i as u32 * 0x400000) + (j as u32 * 0x1000);
            *pt_ptr.add(j) = page_phys | 0x03;
        }
    }

    unsafe {
        asm!(
            "mov cr3, {0}",
            "mov eax, cr0",
            "or eax, 0x80000000",
            "mov cr0, eax",
            "jmp 2f",
            "2:",
            in(reg) pd_phys,
            out("eax") _,
            options(nostack)
        )
    };
}
