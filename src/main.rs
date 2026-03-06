#![no_std]
#![no_main]
pub mod dos;
pub mod util;

use core::arch::asm;
use core::panic::PanicInfo;
use core::ptr::addr_of;
extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

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
    static STACK_TOP: u8;
    fn user_entry_point() -> !;
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
    let msg = _info.message().as_str();
    dos::dos_print(msg.unwrap().as_bytes());
    let msg2 = b"\r\nPanic occurred at: ";
    dos::dos_print(msg2);
    if let Some(location) = _info.location() {
        let mut buf = [0u8; 140];
        let mut pos = 0;
        pos = util::emplace_str_into_buf(msg2, &mut buf, pos);
        pos = util::print_decimal_into_buf(location.file().as_ptr() as u32, &mut buf, pos);
        util::emplace_str_into_buf(b":", &mut buf, pos);
        pos += 1;
        pos = util::print_decimal_into_buf(location.line() as u32, &mut buf, pos);
        util::emplace_str_into_buf(b":", &mut buf, pos);
        pos += 1;
        pos = util::print_decimal_into_buf(location.column() as u32, &mut buf, pos);
        dos::dos_print(&buf[..pos]);
    }
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

static VERBOSE_ALLOC: bool = false;

pub fn test_allocator_capacity() {
    use alloc::vec::Vec;

    const KB_GROW_SIZE: u32 = 16;
    const BLOCK_SIZE: usize = (KB_GROW_SIZE) as usize * 1024;
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
                total_kb += &KB_GROW_SIZE;

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

static VERBOSE_STARTUP: bool = false;
static STARTUP_TEST_VGA_DRAWING: bool = false;
static STARTUP_TEST_MESSAGES: bool = true;
static STARTUP_TEST_ALLOCATOR: bool = true;

use core::ptr;
use core::ptr::NonNull;
use talc::{ErrOnOom, Span, Talc};

pub struct SingleThreadedAlloc {
    inner: core::cell::UnsafeCell<Talc<ErrOnOom>>,
}

unsafe impl Sync for SingleThreadedAlloc {}

#[global_allocator]
static ALLOCATOR: SingleThreadedAlloc = SingleThreadedAlloc {
    inner: core::cell::UnsafeCell::new(Talc::new(ErrOnOom)),
};

unsafe impl GlobalAlloc for SingleThreadedAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let talc = &mut *self.inner.get();
        talc.malloc(layout)
            .map(|ptr| ptr.as_ptr())
            .unwrap_or(core::ptr::null_mut())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let talc = &mut *self.inner.get();
        // dos::dos_print(b"Deallocating memory...\r\n");
        // let msg = b"Dealloc ptr: ";
        // let mut buf = [0u8; 140];
        // util::emplace_str_into_buf(msg, &mut buf, 0);
        // let mut pos = msg.len();
        // pos = util::print_decimal_into_buf(ptr as u32, &mut buf, pos);
        // util::emplace_str_into_buf(b", size: ", &mut buf, pos);
        // pos += 8;
        // pos = util::print_decimal_into_buf(layout.size() as u32, &mut buf, pos);
        // util::emplace_str_into_buf(b"\r\n", &mut buf, pos);
        // pos += 2;
        // dos::dos_print(&buf[..pos]);
        talc.free(core::ptr::NonNull::new_unchecked(ptr), layout);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_ptr = NonNull::new_unchecked(ptr);
        let talc = &mut *self.inner.get();
        match talc.grow(old_ptr, layout, new_size) {
            Ok(new_ptr) => new_ptr.as_ptr(),
            Err(_) => {
                let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
                let new_ptr = talc
                    .malloc(new_layout)
                    .map_or(ptr::null_mut(), |p| p.as_ptr());

                if !new_ptr.is_null() {
                    ptr::copy_nonoverlapping(ptr, new_ptr, layout.size());
                    talc.free(old_ptr, layout);
                }
                new_ptr
            }
        }
    }
}

static BUMP_POINTER: AtomicUsize = AtomicUsize::new(0x100000);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_main(load_base: u32) -> ! {
    unsafe { setup_paging(load_base) };
    unsafe {
        asm!(
            "mov fs, ax",
            // "mov byte ptr fs:[0xB8000], '!'",
            // "mov byte ptr fs:[0xB8001], 0x0E",
            in("ax") 0x28u16,
        );
    } // I genuinely don't remember why the fs segment is used for VGA text mode output
    let heap_start = BUMP_POINTER.load(Ordering::Relaxed);
    let heap_end = addr_of!(STACK_TOP) as usize - 0x10000;
    unsafe {
        // ALLOCATOR.init(heap_start, heap_end - heap_start);
        // let span = Span::from_raw_parts(heap_start, heap_end - heap_start);
        let start_ptr = heap_start as *mut u8;
        let span = Span::new(start_ptr, start_ptr.add(heap_end - heap_start));
        (&mut *ALLOCATOR.inner.get()).claim(span);
        if VERBOSE_ALLOC {
            dos::dos_print(b"Heap initialized.\r\n");
            let mut msg = [0u8; 320];
            let msg_str = b"Heap range: ";
            util::emplace_str_into_buf(msg_str, &mut msg, 0);
            let mut pos = msg_str.len();
            pos = util::print_decimal_into_buf(heap_start as u32, &mut msg, pos);
            util::emplace_str_into_buf(b" - ", &mut msg, pos);
            pos += 3;
            pos = util::print_decimal_into_buf(heap_end as u32, &mut msg, pos);
            util::emplace_str_into_buf(b"\r\n", &mut msg, pos);
            pos += 2;
            dos::dos_print(&msg[..pos]);
        }
    }
    if STARTUP_TEST_VGA_DRAWING {
        unsafe {
            asm!(
                "mov byte ptr fs:[0xB8002], '?'",
                "mov byte ptr fs:[0xB8003], 0x0C",
            );
        }
    }
    if VERBOSE_STARTUP {
        dos::dos_print(b"Paging Enabled Successfully!\r\n");
    }
    if STARTUP_TEST_VGA_DRAWING {
        unsafe {
            asm!(
                "mov byte ptr fs:[0xB8004], 'q'",
                "mov byte ptr fs:[0xB8005], 0x0C",
            );
        }
    }
    if VERBOSE_STARTUP {
        dos::dos_print(b"dosoxide runs off base:");
        print_decimal(load_base);
        dos::dos_print(b".\r\n");
        let mem_kb = unsafe { dos::get_extended_memory_kb() };
        dos::dos_print(b"Conventional memory: ");
        print_decimal(mem_kb);
        dos::dos_print(b" KB\r\n");
    }

    if STARTUP_TEST_MESSAGES {
        let mut numbers = alloc::vec::Vec::new();
        numbers.push(123);
        numbers.push(456);

        let msg = alloc::string::String::from("Hello from the Heap!\r\n");
        dos::dos_print(msg.as_bytes());
        print_decimal(*numbers.get(0).unwrap());
        print_decimal(*numbers.get(1).unwrap());

        {
            let mut msg2 = [0u8; 46];
            let some_numbers = b"\r\nSome numbers: ";
            util::emplace_str_into_buf(some_numbers, &mut msg2, 0);
            let mut pos = some_numbers.len();
            pos = util::print_decimal_into_buf(789, &mut msg2, pos);
            pos = util::print_decimal_into_buf(101112, &mut msg2, pos);
            util::emplace_str_into_buf(b"\r\n", &mut msg2, pos);
            pos += 2;
            dos::dos_print(&msg2[..pos]);
        }
        drop(numbers);
        drop(msg);
    }
    if STARTUP_TEST_ALLOCATOR {
        test_allocator_capacity();
        let should_run_pass_2 = true;
        if should_run_pass_2 {
            dos::dos_print(
                b"\r\nRunning allocator test again to check reuse of freed memory...\r\n",
            );
            test_allocator_capacity();
        }
    }

    // loop {}
    unsafe { user_entry_point() };
}

unsafe fn setup_paging(load_base: u32) {
    let current = BUMP_POINTER.load(Ordering::Relaxed);
    let current_phys = load_base + current as u32;
    let pd_phys = (current_phys + 4095) & !4095;
    let pd_linker = pd_phys - load_base;
    let pd_ptr = pd_linker as *mut u32;

    let total_memory_to_use = 32 * 1024 * 1024;
    // let total_memory_to_use = 64 * 1024 * 1024;
    // let total_memory_to_use = 128 * 1024 * 1024;
    // let total_memory_to_use = 256 * 1024 * 1024;
    let num_pages = total_memory_to_use / 4096;
    let num_pdes = num_pages / 1024;
    let num_total_pages = 1 + num_pdes;

    let end_phys = pd_phys + (num_total_pages * 4096);
    let end_linker = end_phys - load_base;
    BUMP_POINTER.store(end_linker as usize, Ordering::Relaxed);

    let pt0_phys = pd_phys + 4096;

    for i in 0..num_pdes {
        let pt_phys = pt0_phys + i * 4096;
        *pd_ptr.add(i as usize) = pt_phys | 0x03;

        let pt_linker = pt_phys - load_base;
        let pt_ptr = pt_linker as *mut u32;
        for j in 0..1024u32 {
            let page_phys = (i * 0x400000) + (j * 0x1000);
            *pt_ptr.add(j as usize) = page_phys | 0x03;
        }
    }

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
    );
}
