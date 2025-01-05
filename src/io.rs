

// The offsets for reach register.
static GPIO_BASE: usize = 0x200000;

// Controls actuation of pull up/down to ALL GPIO pins.
static GPPUD: usize = GPIO_BASE + 0x94;

// Controls actuation of pull up/down for specific GPIO pin.
static GPPUDCLK0: usize = GPIO_BASE + 0x98;

pub mod mmio {
    use core::sync::atomic::{AtomicUsize, Ordering};
    static MMIO_BASE: AtomicUsize = AtomicUsize::new(0);
    
    pub fn init(raspi: u32) {
        match raspi {
            2 | 3 => {
                MMIO_BASE.store(0x3F000000, Ordering::Relaxed);
            }
            4 => {
                MMIO_BASE.store(0xFE000000, Ordering::Relaxed);
            }
            _ => {
                MMIO_BASE.store(0x20000000, Ordering::Relaxed);
            }
        }
    }
    
    pub unsafe fn write(reg: usize, data: u32) {
        let mmio_base = MMIO_BASE.load(Ordering::Relaxed);
        if mmio_base == 0 {
            return;
        }
        ((mmio_base + reg) as *mut u32).write_volatile(data);
    }
    
    pub unsafe fn read(reg: usize) -> u32 {
        let mmio_base = MMIO_BASE.load(Ordering::Relaxed);
        if mmio_base == 0 {
            return 0;
        }
        ((mmio_base + reg) as *mut u32).read_volatile()
    }

}

pub mod uart {
    use core::arch::asm;

    use super::{GPIO_BASE, GPPUD, GPPUDCLK0, mmio};
    
    // The base address for UART.
    static UART0_BASE: usize = GPIO_BASE + 0x1000; // for raspi4 0xFE201000, raspi2 & 3 0x3F201000, and 0x20201000 for raspi1
    
    // The offsets for reach register for the UART.
    static UART0_DR: usize = UART0_BASE + 0x00;
    static UART0_RSRECR: usize = UART0_BASE + 0x04;
    static UART0_FR: usize = UART0_BASE + 0x18;
    static UART0_ILPR: usize = UART0_BASE + 0x20;
    static UART0_IBRD: usize = UART0_BASE + 0x24;
    static UART0_FBRD: usize = UART0_BASE + 0x28;
    static UART0_LCRH: usize = UART0_BASE + 0x2C;
    static UART0_CR: usize = UART0_BASE + 0x30;
    static UART0_IFLS: usize = UART0_BASE + 0x34;
    static UART0_IMSC: usize = UART0_BASE + 0x38;
    static UART0_RIS: usize = UART0_BASE + 0x3C;
    static UART0_MIS: usize = UART0_BASE + 0x40;
    static UART0_ICR: usize = UART0_BASE + 0x44;
    static UART0_DMACR: usize = UART0_BASE + 0x48;
    static UART0_ITCR: usize = UART0_BASE + 0x80;
    static UART0_ITIP: usize = UART0_BASE + 0x84;
    static UART0_ITOP: usize = UART0_BASE + 0x88;
    static UART0_TDR: usize = UART0_BASE + 0x8C;
    
    // The offsets for Mailbox registers
    static MBOX_BASE: usize = 0xB880;
    static MBOX_READ: usize = MBOX_BASE + 0x00;
    static MBOX_STATUS: usize = MBOX_BASE + 0x18;
    static MBOX_WRITE: usize = MBOX_BASE + 0x20;
    
    /// Loop `count` times in a way that the compiler won't optimize away
    pub fn delay(mut _count: usize) {
        unsafe {
            asm!(
                "
                1:
                subs {count}, {count}, #1
                bne 1b
                ",
                count = inout(reg) _count,
                options(nostack, preserves_flags)
            );
        }
    }
    
    #[repr(align(16))]
    struct MboxMessage([u32; 9]);
    
    // A Mailbox message with set clock rate of PL011 to 3MHz tag
    static MBOX: MboxMessage = MboxMessage([
        9*4, 0, 0x38002, 12, 8, 2, 3000000, 0 ,0
    ]);
    
    pub fn init(raspi: u32) {
        unsafe {
            mmio::init(raspi);
        
            // Disable UART0.
            mmio::write(UART0_CR, 0x00000000);
            // Setup the GPIO pin 14 && 15.
        
            // Disable pull up/down for all GPIO pins & delay for 150 cycles.
            mmio::write(GPPUD, 0x00000000);
            delay(150);
        
            // Disable pull up/down for pin 14,15 & delay for 150 cycles.
            mmio::write(GPPUDCLK0, (1 << 14) | (1 << 15));
            delay(150);
        
            // Write 0 to GPPUDCLK0 to make it take effect.
            mmio::write(GPPUDCLK0, 0x00000000);
        
            // Clear pending interrupts.
            mmio::write(UART0_ICR, 0x7FF);
        
            // Set integer & fractional part of baud rate.
            // Divider = CLOCK/(16 * Baud)
            // Fraction part register = (Fractional part * 64) + 0.5
            // Baud = 115200.
        
            // For Raspi3 and 4 the CLOCK is system-clock dependent by default.
            // Set it to 3Mhz so that we can consistently set the baud rate
            if raspi >= 3 {
                // CLOCK = 30000000;
                let r = ((&MBOX as *const MboxMessage as u32) & !0xF) | 8;
                // wait until we can talk to the VC
                while mmio::read(MBOX_STATUS) & 0x80000000 != 0 { }
                // send our message to property channel and wait for the response
                mmio::write(MBOX_WRITE, r);
                while mmio::read(MBOX_STATUS) & 0x40000000 != 0 || mmio::read(MBOX_READ) != r  { }
            }
        
            // Divider = 3000000 / (16 * 115200) = 1.627 = ~1.
            mmio::write(UART0_IBRD, 1);
            // Fractional part register = (.627 * 64) + 0.5 = 40.6 = ~40.
            mmio::write(UART0_FBRD, 40);
        
            // Enable FIFO & 8 bit data transmission (1 stop bit, no parity).
            mmio::write(UART0_LCRH, (1 << 4) | (1 << 5) | (1 << 6));
        
            // Mask all interrupts.
            mmio::write(UART0_IMSC, (1 << 1) | (1 << 4) | (1 << 5) | (1 << 6) |
                                (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10));
        
            // Enable UART0, receive & transfer part of UART.
            mmio::write(UART0_CR, (1 << 0) | (1 << 8) | (1 << 9));
        }
    }
    
    pub fn putb(byte: u8) {
        while unsafe { mmio::read(UART0_FR) } & (1 << 5) != 0 { }
        unsafe {
            mmio::write(UART0_DR, byte as u32);
        }
    }
    
    pub fn getb() -> u8 {
        while unsafe { mmio::read(UART0_FR) } & (1 << 4) != 0 { }
        unsafe {
            mmio::read(UART0_DR) as u8
        }
    }
    
    pub fn puts(string: &str) {
        for byte in string.bytes() {
            putb(byte);
        }
    }
    
    pub fn gets(buf: &mut [u8]) -> usize {
        for len in 0..buf.len() {
            let b = getb();
            match b {
                b'\n' | b'\r' => {
                    puts("\r\n");
                    return len;
                }
                b'\x08' => puts("\x08 \x08"),
                _ => putb(b),
            }
            buf[len] = b;
        }
        return buf.len();
    }

    pub fn put_u8(num: u8) {
        static HEX_CHART: [u8; 16] = [
            b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
            b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F'
        ];
        let lower_nibble = num & 0x0F;
        let upper_nibble = (num & 0xF0) >> 4;
        putb(HEX_CHART[upper_nibble as usize]);
        putb(HEX_CHART[lower_nibble as usize]);
    }
}
